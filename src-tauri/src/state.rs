//! Shared app state, locking, and the background auto-lock monitor.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant, SystemTime};

use tauri::{AppHandle, Emitter, Manager};

use crate::error::{Error, Result};
use crate::portable::Paths;
use crate::vault::Session;
use crate::{backup, system, ui};

/// A vault with security off is backed up at most this often while it is open (and only if
/// something changed since the last hourly backup), because it never locks.
const OPEN_BACKUP_EVERY: Duration = Duration::from_secs(60 * 60);
/// After a failed hourly backup, the next try comes after this long, then 3 times and 12 times
/// as long if it keeps failing (5 minutes, 15 minutes, then hourly). Each try copies the whole
/// vault, so a drive that stays full or broken must not be hammered.
const OPEN_BACKUP_RETRY: Duration = Duration::from_secs(5 * 60);
/// The hourly backup holds the vault for as long as the copy takes, which on a slow USB stick
/// can be many seconds, so it starts only after the keyboard and mouse have been still this
/// long (so it does not freeze the window mid-typing) ...
const OPEN_BACKUP_IDLE_MS: u64 = 30_000;
/// ... unless it is this much overdue, so a PC that is never idle (or whose idle time can't be
/// read) is still backed up.
const OPEN_BACKUP_OVERDUE: Duration = Duration::from_secs(90 * 60);

/// How often the monitor makes the hourly backup of an open vault, how soon it retries after a
/// failure, and how long the keyboard and mouse must have been still before it starts. The
/// real-exe test scripts set `NOTAYA_OPEN_BACKUP_SECS` (until V1.3.0 `CIPHERLEAF_OPEN_BACKUP_SECS`, still
/// read) to shorten both waits and to drop the pause, so the hourly backup can be watched without
/// waiting an hour (like `NOTAYA_HOME`, it is read in release builds too).
fn open_backup_timing() -> (Duration, Duration, u64) {
    let secs = crate::portable::env_var("NOTAYA_OPEN_BACKUP_SECS", "CIPHERLEAF_OPEN_BACKUP_SECS")
        .and_then(|v| v.to_str().and_then(|v| v.trim().parse::<u64>().ok()));
    match secs.filter(|n| *n > 0) {
        Some(n) => (Duration::from_secs(n), Duration::from_secs(n), 0),
        None => (OPEN_BACKUP_EVERY, OPEN_BACKUP_RETRY, OPEN_BACKUP_IDLE_MS),
    }
}

/// How long a lock waits for the windows to save their pending edits.
const FLUSH_TIMEOUT: Duration = Duration::from_secs(2);

struct Attempts {
    failures: u32,
    wait_until: Option<Instant>,
}

/// Progress of one "save your edits before the vault locks" round. Each round
/// has a number, and only an answer carrying the current number from a window
/// that was asked counts, so a late answer can't finish a later round early.
#[derive(Default)]
struct Flush {
    /// Number of the latest round (0 before the first).
    round: u64,
    /// Labels of the windows that were asked and haven't answered yet.
    pending: HashSet<String>,
}

impl Flush {
    /// Starts a new round for `expected` and returns its number.
    fn begin(&mut self, expected: HashSet<String>) -> u64 {
        self.round += 1;
        self.pending = expected;
        self.round
    }

    /// Records an answer. Returns true if it counted: the round is the current
    /// one and `label` was still pending.
    fn ack(&mut self, label: &str, round: u64) -> bool {
        round == self.round && self.pending.remove(label)
    }

    fn is_done(&self) -> bool {
        self.pending.is_empty()
    }

    /// Ends the round; answers arriving later are ignored.
    fn finish(&mut self) {
        self.pending.clear();
    }
}

/// The shell supplies runtime operations; core state and its tests never own an AppHandle.
#[derive(Clone)]
struct ChangeSink {
    changed: Arc<dyn Fn(crate::changes::ItemsChanged) + Send + Sync>,
    close_sticky: Arc<dyn Fn(&str) + Send + Sync>,
}

pub struct AppState {
    pub paths: Paths,
    change_sink: Mutex<Option<ChangeSink>>,
    session: Mutex<Option<Session>>,
    attempts: Mutex<Attempts>,
    /// Labels of windows that have called `window_ready` (they listen for `vault-locking`).
    ready: Mutex<HashSet<String>>,
    flush: Mutex<Flush>,
    flush_cv: Condvar,
    /// Lets only one flush round run at a time.
    flush_gate: Mutex<()>,
    /// The backup file the user most recently chose in the file dialog (canonical
    /// path). `restore_vault` accepts an absolute path only if it is this one.
    picked_backup: Mutex<Option<PathBuf>>,
    /// A message for the owner that the UI shows once (for example after a backup
    /// failed). Kept here as well as sent as an event, because a lock
    /// reloads the window and a live event could be missed.
    notice: Mutex<Option<String>>,
    /// Held while the app opens or creates a vault that has security off, and while a
    /// restore swaps the vault file, so two windows asking at once can't both create one
    /// and nothing starts a new vault in the middle of a restore.
    open_gate: Mutex<()>,
    /// A quit is in progress: nothing may reopen a vault that has security off meanwhile.
    quitting: AtomicBool,
    /// The tray icon exists. Without it nothing may hide the window: there would be no way back.
    tray_ready: AtomicBool,
    /// The main window is hidden in the tray (hidden by the close button, or started hidden by
    /// Windows). While this is set, the window does not show itself when it reports ready (a lock
    /// reloads it, and the reload must not bring it back); opening it from the tray clears it.
    tray_hidden: AtomicBool,
    /// Held while the close button decides whether the vault needs locking, and while an unlock
    /// stores its session, so an unlock that finishes after the window was hidden cannot leave
    /// the vault open in the tray.
    tray_gate: Mutex<()>,
}

fn guard<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    // A panic while holding the lock must not brick the app; recover the data.
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl AppState {
    pub fn new(paths: Paths) -> Self {
        AppState {
            paths,
            change_sink: Mutex::new(None),
            session: Mutex::new(None),
            attempts: Mutex::new(Attempts { failures: 0, wait_until: None }),
            ready: Mutex::new(HashSet::new()),
            flush: Mutex::new(Flush::default()),
            flush_cv: Condvar::new(),
            flush_gate: Mutex::new(()),
            picked_backup: Mutex::new(None),
            notice: Mutex::new(None),
            open_gate: Mutex::new(()),
            quitting: AtomicBool::new(false),
            tray_ready: AtomicBool::new(false),
            tray_hidden: AtomicBool::new(false),
            tray_gate: Mutex::new(()),
        }
    }

    pub fn tray_ready(&self) -> bool {
        self.tray_ready.load(Ordering::SeqCst)
    }

    pub fn set_tray_ready(&self, on: bool) {
        self.tray_ready.store(on, Ordering::SeqCst);
    }

    pub fn set_tray_hidden(&self, on: bool) {
        self.tray_hidden.store(on, Ordering::SeqCst);
    }

    pub fn tray_hidden(&self) -> bool {
        self.tray_hidden.load(Ordering::SeqCst)
    }

    pub fn open_gate(&self) -> MutexGuard<'_, ()> {
        guard(&self.open_gate)
    }

    pub fn is_quitting(&self) -> bool {
        self.quitting.load(Ordering::SeqCst)
    }

    fn set_quitting(&self, on: bool) {
        self.quitting.store(on, Ordering::SeqCst);
    }

    /// Keeps `text` for the UI to show once.
    pub fn set_notice(&self, text: String) {
        *guard(&self.notice) = Some(text);
    }

    /// The pending notice, if any. It is returned once and then forgotten.
    pub fn take_notice(&self) -> Option<String> {
        guard(&self.notice).take()
    }

    /// Remembers the file the user just chose in the backup dialog.
    pub fn set_picked_backup(&self, path: PathBuf) {
        *guard(&self.picked_backup) = Some(path);
    }

    pub fn picked_backup(&self) -> Option<PathBuf> {
        guard(&self.picked_backup).clone()
    }

    /// Forgets the chosen file, so a restore attempt can't be repeated without a new pick.
    pub fn clear_picked_backup(&self) {
        *guard(&self.picked_backup) = None;
    }

    /// Remembers that the window `label` is ready and will flush when asked.
    pub fn mark_ready(&self, label: &str) {
        guard(&self.ready).insert(label.to_string());
    }

    /// Forgets a window that no longer exists, so a lock doesn't wait for it.
    pub fn unmark_ready(&self, label: &str) {
        if let Some(s) = guard(&self.session).as_mut() { s.coordination.forget_window(label); }
        guard(&self.ready).remove(label);
        let mut f = guard(&self.flush);
        if f.pending.remove(label) {
            self.flush_cv.notify_all();
        }
    }

    /// The window `label` finished saving its pending edits (answer to the
    /// `vault-locking` event of round `round`). Ignored if the round is not the
    /// current one or the window wasn't asked.
    pub fn flush_done(&self, label: &str, round: u64) {
        if guard(&self.flush).ack(label, round) {
            self.flush_cv.notify_all();
        }
    }

    pub fn is_unlocked(&self) -> bool {
        guard(&self.session).is_some()
    }

    pub fn set_session(&self, mut session: Session) {
        session.drain_changes(); // creation/unlock is followed by a complete window load
        *guard(&self.session) = Some(session);
        crate::logging::unlocked_event();
    }

    /// Stores the session of a finished unlock, unless the main window is hidden in the tray
    /// meanwhile (the person closed the lock screen while the password was being checked): then
    /// the session is dropped (wiped) and the vault stays locked.
    pub fn set_session_unless_hidden(&self, session: Session) -> Result<()> {
        let _gate = guard(&self.tray_gate);
        if self.tray_hidden() {
            return Err(Error::Locked);
        }
        self.set_session(session);
        Ok(())
    }

    pub fn take_session(&self) -> Option<Session> {
        // Messages of automations hold row titles: they do not outlive the unlocked session.
        crate::commands::automation::clear_notices();
        guard(&self.session).take()
    }

    pub fn with<T>(&self, f: impl FnOnce(&Session) -> Result<T>) -> Result<T> {
        match guard(&self.session).as_ref() {
            Some(s) => f(s),
            None => Err(Error::Locked),
        }
    }

    pub fn with_mut<T>(&self, f: impl FnOnce(&mut Session) -> Result<T>) -> Result<T> {
        let mut session = guard(&self.session);
        let s = session.as_mut().ok_or(Error::Locked)?;
        let result = f(s);
        let mut close = Vec::new();
        let sink = guard(&self.change_sink).clone();
        // Some old cores perform several transactions. Publish durable partial writes even
        // if a later step failed; they must never leak into the next command's signal.
        if let Some(changes) = s.drain_changes() {
            if let Some(sink) = sink.as_ref() {
                for id in &changes.ids {
                    if matches!(s.get(id), Some(crate::model::Item::Sticky(st)) if st.trashed_at.is_some()) {
                        close.push(id.clone());
                    }
                }
                (sink.changed)(changes);
            }
        }
        drop(session);
        // Destroyed windows call unmark_ready, which also takes the session lock.
        if let Some(sink) = sink {
            for id in close {
                (sink.close_sticky)(&id);
            }
        }
        result
    }

    pub fn bind_events(
        &self,
        changed: impl Fn(crate::changes::ItemsChanged) + Send + Sync + 'static,
        close_sticky: impl Fn(&str) + Send + Sync + 'static,
    ) {
        *guard(&self.change_sink) = Some(ChangeSink {
            changed: Arc::new(changed),
            close_sticky: Arc::new(close_sticky),
        });
    }

    /// Slows down repeated guessing: after 3 failures, waits 2, 4, 8 … up to 60 seconds.
    pub fn check_rate_limit(&self) -> Result<()> {
        let a = guard(&self.attempts);
        if let Some(until) = a.wait_until {
            let now = Instant::now();
            if now < until {
                return Err(Error::RateLimited((until - now).as_secs() + 1));
            }
        }
        Ok(())
    }

    pub fn record_failure(&self) {
        let mut a = guard(&self.attempts);
        a.failures += 1;
        if a.failures >= 3 {
            let secs = 2u64.saturating_pow(a.failures - 2).min(60);
            a.wait_until = Some(Instant::now() + Duration::from_secs(secs));
        }
    }

    pub fn record_success(&self) {
        let mut a = guard(&self.attempts);
        a.failures = 0;
        a.wait_until = None;
    }
}

/// Emits `vault-locking` (payload: the round number, a `u64`) and waits until
/// every ready window that still exists has answered with `flush_done` for that
/// round, or until `timeout` passes. Does nothing when no window is expected to
/// answer.
///
/// Never call this on the main (event-loop) thread: the windows' `flush_done`
/// calls need that thread to be free.
pub fn request_flush(app: &AppHandle, timeout: Duration) {
    let state = app.state::<AppState>();
    let _gate = guard(&state.flush_gate);
    if !state.is_unlocked() {
        return; // another lock already finished while we waited for the gate
    }
    let expected: HashSet<String> = {
        let open = app.webview_windows();
        let mut ready = guard(&state.ready);
        ready.retain(|label| open.contains_key(label));
        ready.clone()
    };
    if expected.is_empty() {
        return;
    }
    let round = guard(&state.flush).begin(expected);
    let _ = app.emit("vault-locking", round);
    let deadline = Instant::now() + timeout;
    let mut f = guard(&state.flush);
    while !f.is_done() {
        let Some(left) = deadline.checked_duration_since(Instant::now()) else {
            break;
        };
        f = match state.flush_cv.wait_timeout(f, left) {
            Ok((g, _)) => g,
            Err(e) => e.into_inner().0,
        };
    }
    if !f.is_done() {
        let mut missing: Vec<&str> = f.pending.iter().map(String::as_str).collect();
        missing.sort_unstable();
        eprintln!("flush round {round}: no answer from {}", missing.join(", "));
    }
    f.finish();
}

/// Whether a lock tells the UI with `vault-locked`. A restore finishes first and
/// the UI reloads itself, so it must not reload in the middle of the restore.
fn emits_vault_locked(reason: &str, was_open: bool) -> bool {
    // A quit of a vault with security off needs no reload: there is no lock screen to show.
    reason != "restore" && !(reason == "quit" && was_open)
}

/// Asks every window to write its pending edits, then waits (up to the usual lock time). Used before
/// closing sticky notes without locking ("Close all"). Same rule as `request_flush`: not on the main thread.
pub fn flush_windows(app: &AppHandle) {
    request_flush(app, FLUSH_TIMEOUT);
}

/// Locks the vault: first lets every window save its pending edits (unless the
/// drive was removed, when saving can't work), then wipes keys and decrypted
/// data from memory, closes sticky notes and calculators, tells the UI to reset itself, and only
/// then does the slow work (the backup).
///
/// Waits for the windows, so call it only from a thread other than the main
/// one (an `async` command, the monitor thread, the close handler's thread).
///
/// Returns the failure notice if the backup made on locking failed (it has been reported
/// already as a notice and an event); a quit uses it, because the window is gone before
/// anyone could read the notice. `None` when nothing failed.
pub fn lock(app: &AppHandle, reason: &str) -> Option<String> {
    let state = app.state::<AppState>();
    // A vault with security off never locks by itself or by a button. Only a quit and a
    // restore close it (they have to, to save and to swap the file).
    if !matches!(reason, "quit" | "restore") && state.with(|s| Ok(s.is_open())).unwrap_or(false) {
        return None;
    }
    // An import in progress stops at once: it must not go on reading and converting files for
    // minutes after the notes were locked (it ends as a failure and adds nothing).
    crate::commands::import_notes::cancel_running();
    if reason != "drive-removed" && state.is_unlocked() {
        request_flush(app, FLUSH_TIMEOUT);
    }
    let session = state.take_session();
    let was_open = session.as_ref().is_some_and(|s| s.is_open());
    if session.is_some() {
        crate::logging::locked_event(reason);
    }
    // The UI comes first: close stickies, reset the windows.
    ui::close_stickies(app);
    ui::close_calculators(app);
    if emits_vault_locked(reason, was_open) {
        let _ = app.emit("vault-locked", reason.to_string());
    }
    // Slow parts after that.
    let made = session.as_ref().filter(|s| s.dirty).map(|s| backup::make_backup(&state.paths, s));
    let mut backup_failure = None;
    if let Some(Err(e)) = made {
        backup_failure = Some(report_backup_failure(app, &state, &e));
    }
    drop(session); // zeroizes the data key and all items
    backup_failure
}

/// Quits Notaya: locks (saving the windows' edits and making the backup) and exits.
///
/// Waits for the windows, so call it only from a thread other than the main one.
pub fn quit(app: &AppHandle) -> bool {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    let state = app.state::<AppState>();
    // While quitting, nothing may reopen a vault that has security off.
    state.set_quitting(true);
    // Whatever fails or panics here, the exit must still happen.
    let backup_failure = catch_unwind(AssertUnwindSafe(|| lock(app, "quit"))).ok().flatten();
    // A failed backup at quit would go unseen (the window is destroyed right away), so say it
    // in a message box first. Not when the vault file is gone (the drive was removed): the
    // notice then is about the drive, and nothing may hold up the exit.
    if let Some(text) = backup_failure.filter(|_| state.paths.vault.exists()) {
        let _ = catch_unwind(AssertUnwindSafe(|| show_backup_failure(app, &text)));
    }
    app.exit(0);
    true
}

/// The close button with "close to tray" on: the window goes away but Notaya keeps running.
/// With a master password the vault locks first (every window saves, sticky notes and
/// calculators close, the lock screen is shown, the backup is made), so the notes are not left
/// open in the tray. With security off nothing is locked, but the windows still save their pending
/// edits and a backup is made if anything changed (a vault with security off otherwise gets its
/// ordinary backup only at quit, which may now be rare). Only then is the main window hidden. A
/// failed backup is returned for the caller to tell in a message box (the window is hidden, so a toast would
/// go unseen).
///
/// Waits for the windows, so call it only from a thread other than the main one.
pub fn hide_to_tray(app: &AppHandle) -> Option<String> {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    let state = app.state::<AppState>();
    // Set first, under the gate: the lock below reloads the window and its "ready" must not show
    // it again, and an unlock still running must not store its session after this decision.
    let protected = {
        let _gate = guard(&state.tray_gate);
        state.set_tray_hidden(true);
        state.with(|s| Ok(!s.is_open())).unwrap_or(false)
    };
    let backup_failure = catch_unwind(AssertUnwindSafe(|| {
        if protected {
            lock(app, "tray")
        } else {
            request_flush(app, FLUSH_TIMEOUT);
            backup_open_vault(app, &state)
        }
    }))
    .ok()
    .flatten();
    // Hide only if nobody opened the window meanwhile (the tray's Open, or a second start). If hiding
    // fails, the window stays and the flag is cleared, so it is not left half hidden.
    if state.tray_hidden() {
        if let Some(w) = app.get_webview_window("main") {
            if w.hide().is_err() {
                state.set_tray_hidden(false);
            }
        }
    }
    backup_failure.filter(|_| state.paths.vault.exists())
}

/// The ordinary backup of an open vault (security off) when something changed since the last
/// one, as a lock or quit would make it. Returns the failure notice if it failed.
fn backup_open_vault(app: &AppHandle, state: &AppState) -> Option<String> {
    let result = state.with_mut(|s| {
        if !s.dirty {
            return Ok(());
        }
        backup::make_backup(&state.paths, s)?;
        s.dirty = false;
        Ok(())
    });
    match result {
        Err(Error::Locked) | Ok(()) => None,
        Err(e) => Some(report_backup_failure(app, state, &e)),
    }
}

/// A native message box that waits until it is closed. Call it from a thread other than the
/// main one (as `quit` is).
pub(crate) fn show_backup_failure(app: &AppHandle, text: &str) {
    use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
    let _ = app
        .dialog()
        .message(text.to_string())
        .title("Notaya could not make a backup")
        .kind(MessageDialogKind::Warning)
        .blocking_show();
}

/// Tells the owner that an automatic backup could not be made (the release exe has no
/// console, so a message on stderr would never be seen). The notice is kept as well as
/// sent as an event, because a lock reloads the window and a live event could be
/// missed. The vault file itself is not affected. Returns the text.
pub(crate) fn report_backup_failure(app: &AppHandle, state: &AppState, e: &Error) -> String {
    eprintln!("backup failed: {e}");
    // What the notice says about the notes depends on the cause (and on whether the vault
    // file is still there: it is not when the drive was removed).
    let text = backup::failure_notice(e, state.paths.vault.exists());
    state.set_notice(text.clone());
    let _ = app.emit("app-notice", text.clone());
    text
}

/// A backup of an open vault (security off) while it is running: it never locks, so
/// without this it would be backed up only when the app quits. It is an hourly copy
/// (`vault-hourly-<ms>.clf`, the newest 3 kept) and does nothing when nothing changed since
/// the last one. It does not stand in for the backup made at lock or quit, so the older
/// backups keep a copy from the end of each session. Holds the session for the length of the
/// backup, like the safety backup before a delete does.
///
/// Returns `Some(true)` when it worked or had nothing to do, `Some(false)` when it failed, and
/// `None` when the session was taken in the meantime. A failure is shown to the person only
/// when `notify` is true, so a drive that keeps failing is not announced every five minutes.
fn periodic_backup(app: &AppHandle, state: &AppState, notify: bool) -> Option<bool> {
    let result = state.with_mut(|s| {
        if !s.hourly_due {
            return Ok(());
        }
        backup::make_hourly_backup(&state.paths, s)?;
        s.hourly_due = false;
        Ok(())
    });
    match result {
        Ok(()) => Some(true),
        // A quit, a lock or a restore took the session while this was starting: nothing failed.
        Err(Error::Locked) => None,
        Err(e) => {
            if notify {
                report_backup_failure(app, state, &e);
            } else {
                eprintln!("backup failed again: {e}");
            }
            Some(false)
        }
    }
}

/// Checks once a second for reasons to lock: idle time, Win+L, sleep/hibernate,
/// the USB drive being removed, or the window being minimized.
pub fn spawn_monitor(app: AppHandle) {
    std::thread::spawn(move || {
        let mut last_wall = SystemTime::now();
        let mut desktop_seen_unlocked = false;
        let mut last_open_backup = Instant::now();
        let mut open_backup_failures: u32 = 0;
        let (open_backup_every, open_backup_retry, open_backup_idle_ms) = open_backup_timing();
        loop {
            std::thread::sleep(Duration::from_secs(1));
            let state = app.state::<AppState>();

            // A big jump in wall-clock time means the PC was asleep or hibernating.
            let now = SystemTime::now();
            let slept = now.duration_since(last_wall).map(|d| d > Duration::from_secs(20)).unwrap_or(true);
            last_wall = now;

            let Ok((settings, security_off)) = state.with(|s| Ok((s.settings(), s.is_open()))) else {
                desktop_seen_unlocked = false;
                continue;
            };
            // Security off: nothing locks the vault (A21), but it is backed up now and then.
            if security_off {
                let due = match open_backup_failures {
                    0 => open_backup_every,
                    1 => open_backup_retry,
                    2 => open_backup_retry * 3,
                    _ => open_backup_retry * 12,
                };
                let since = last_open_backup.elapsed();
                let quiet = system::idle_ms() >= open_backup_idle_ms || since >= due + OPEN_BACKUP_OVERDUE;
                if since >= due && quiet && !state.is_quitting() {
                    // The first failure is shown, the second (soon after) is not, and from the third
                    // on each one is shown again, at the slow pace, so a drive that stays broken
                    // is not forgotten and not announced every few minutes.
                    let notify = open_backup_failures == 0 || open_backup_failures >= 2;
                    if let Some(ok) = periodic_backup(&app, &state, notify) {
                        open_backup_failures = if ok { 0 } else { open_backup_failures.saturating_add(1) };
                    }
                    // The wait starts when the copy ends (a slow copy must not make the next one
                    // due at once), and the clock jump the copy caused is not a sleep.
                    last_open_backup = Instant::now();
                    last_wall = SystemTime::now();
                }
                continue;
            }
            open_backup_failures = 0;

            let screen_locked = system::workstation_locked();
            if !screen_locked {
                desktop_seen_unlocked = true;
            }

            let reason = if slept {
                Some("sleep")
            } else if !state.paths.vault.exists() {
                Some("drive-removed")
            } else if system::idle_ms() >= settings.idle_lock_minutes as u64 * 60_000 {
                Some("idle")
            } else if screen_locked && desktop_seen_unlocked {
                Some("screen-locked")
            } else if settings.lock_on_minimize
                && app
                    .get_webview_window("main")
                    .and_then(|w| w.is_minimized().ok())
                    .unwrap_or(false)
            {
                Some("minimized")
            } else if state.tray_hidden() {
                // A protected vault must not stay unlocked behind a hidden window, whatever opened it
                // (a restore or turning security on that finished after the window was hidden).
                Some("tray")
            } else {
                None
            };

            if let Some(reason) = reason {
                lock(&app, reason);
                desktop_seen_unlocked = false;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels(l: &[&str]) -> HashSet<String> {
        l.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn right_answers_complete_the_round() {
        let mut f = Flush::default();
        let r = f.begin(labels(&["main", "sticky-1"]));
        assert!(!f.is_done());
        assert!(f.ack("main", r));
        assert!(!f.is_done());
        assert!(f.ack("sticky-1", r));
        assert!(f.is_done());
    }

    #[test]
    fn wrong_round_or_unexpected_label_is_ignored() {
        let mut f = Flush::default();
        let old = f.begin(labels(&["main"]));
        f.finish();
        let r = f.begin(labels(&["main", "sticky-1"]));
        assert_ne!(old, r);
        assert!(!f.ack("main", old), "late answer from an earlier round");
        assert!(!f.ack("other", r), "window that wasn't asked");
        assert!(!f.is_done());
        assert!(f.ack("main", r));
        assert!(!f.ack("main", r), "a second answer doesn't count twice");
        assert!(!f.is_done());
        assert!(f.ack("sticky-1", r));
        assert!(f.is_done());
    }

    #[test]
    fn finished_round_accepts_nothing() {
        let mut f = Flush::default();
        let r = f.begin(labels(&["main"]));
        f.finish();
        assert!(f.is_done());
        assert!(!f.ack("main", r));
    }

    #[test]
    fn only_a_restore_lock_skips_the_vault_locked_event() {
        for reason in ["manual", "idle", "sleep", "screen-locked", "minimized", "drive-removed", "quit", "tray"] {
            assert!(emits_vault_locked(reason, false), "{reason}");
        }
        assert!(!emits_vault_locked("restore", false));
        assert!(!emits_vault_locked("restore", true));
        // Quitting a vault with security off needs no reload; every other lock does.
        assert!(!emits_vault_locked("quit", true));
        assert!(emits_vault_locked("idle", true));
    }

    #[test]
    fn the_picked_backup_is_remembered_replaced_and_cleared() {
        let root = std::env::temp_dir().join(format!("notaya-state-{}", crate::model::new_id()));
        let state = AppState::new(Paths::at(&root));
        assert_eq!(state.picked_backup(), None);
        state.set_picked_backup(PathBuf::from("a.clf"));
        assert_eq!(state.picked_backup(), Some(PathBuf::from("a.clf")));
        state.set_picked_backup(PathBuf::from("b.clf"));
        assert_eq!(state.picked_backup(), Some(PathBuf::from("b.clf")));
        state.clear_picked_backup();
        assert_eq!(state.picked_backup(), None);
    }

    #[test]
    fn a_notice_is_returned_once() {
        let root = std::env::temp_dir().join(format!("notaya-state-{}", crate::model::new_id()));
        let state = AppState::new(Paths::at(&root));
        assert_eq!(state.take_notice(), None);
        state.set_notice("first".into());
        state.set_notice("second".into());
        assert_eq!(state.take_notice().as_deref(), Some("second"));
        assert_eq!(state.take_notice(), None, "gone after the first read");
    }

    #[test]
    fn change_sink_publishes_partial_writes_then_closes_sticky_outside_session_lock() {
        let root = PathBuf::from(
            std::env::var_os("NOTAYA_HOME").unwrap_or_else(|| "D:\\Notaya Tests\\v1-6-C2".into()),
        )
        .join(format!("event-sink-{}", crate::model::new_id()));
        let paths = Paths::at(&root);
        std::fs::create_dir_all(&paths.data).unwrap();
        let mut session = Session::create_open(&paths.vault).unwrap();
        let id = crate::model::new_id();
        session
            .put(serde_json::from_value(serde_json::json!({
                "kind": "sticky", "id": id, "content": "", "text": "", "color": "yellow",
                "x": 0, "y": 0, "w": 320, "h": 320, "onTop": true, "open": true,
                "created": 1, "updated": 1
            })).unwrap())
            .unwrap();
        session.drain_changes();
        let state = Arc::new(AppState::new(paths));
        state.set_session(session);
        let calls = Arc::new(Mutex::new(Vec::new()));
        let changed_calls = calls.clone();
        let closed_calls = calls.clone();
        let emitting_state = Arc::downgrade(&state);
        let closing_state = Arc::downgrade(&state);
        let changed_id = id.clone();
        state.bind_events(
            move |changes| {
                assert_eq!(changes.ids, vec![changed_id.clone()]);
                assert!(
                    matches!(emitting_state.upgrade().unwrap().session.try_lock(),
                        Err(std::sync::TryLockError::WouldBlock)),
                    "signals preserve write order"
                );
                changed_calls.lock().unwrap().push("changed");
            },
            move |id| {
                let state = closing_state.upgrade().unwrap();
                let session = state.session.try_lock()
                    .expect("window destruction can re-enter state");
                assert!(session.as_ref().unwrap().in_bin(id));
                closed_calls.lock().unwrap().push("closed");
            },
        );
        let result: Result<()> = state.with_mut(|s| {
            crate::commands::stickies::delete_sticky_core(s, &crate::actions::Actor::Person, id)?;
            Err(Error::invalid("later step failed"))
        });
        assert!(result.is_err());
        assert_eq!(*calls.lock().unwrap(), vec!["changed", "closed"]);
        state.with_mut(|_| Ok(())).unwrap();
        assert_eq!(
            calls.lock().unwrap().len(), 2,
            "no stale signal in the next command"
        );
    }

    #[test]
    fn empty_round_is_done_at_once() {
        let mut f = Flush::default();
        f.begin(HashSet::new());
        assert!(f.is_done());
    }
}
