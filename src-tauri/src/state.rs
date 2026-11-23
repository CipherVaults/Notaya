//! Shared app state, locking, and the background auto-lock monitor.

use std::collections::{HashSet, VecDeque};
use std::panic::{catch_unwind, AssertUnwindSafe};
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

/// How long a lock waits for window acknowledgements once the session is ready.
const FLUSH_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum SessionPhase {
    #[default]
    Running,
    Draining,
    Flushing,
    Sealed,
}

#[derive(Default)]
struct SessionAdmission {
    phase: SessionPhase,
    prior_saves: usize,
    flush_saves: usize,
}

struct PendingSave<'a> {
    state: &'a AppState,
    during_flush: bool,
}

impl Drop for PendingSave<'_> {
    fn drop(&mut self) {
        let mut admission = guard(&self.state.admission);
        let count = if self.during_flush {
            &mut admission.flush_saves
        } else {
            &mut admission.prior_saves
        };
        *count -= 1;
        if *count == 0 {
            self.state.saves_cv.notify_all();
        }
    }
}

/// The lock thread owns this until it has detached the session.
struct SessionDrain<'a>(&'a AppState);

impl Drop for SessionDrain<'_> {
    fn drop(&mut self) {
        guard(&self.0.admission).phase = SessionPhase::Running;
    }
}

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
    changed: Arc<dyn Fn(crate::changes::ItemsChanged) -> Result<()> + Send + Sync>,
    close_sticky: Arc<dyn Fn(&str) + Send + Sync>,
}

struct ChangeDelivery {
    generation: u64,
    changes: Option<crate::changes::ItemsChanged>,
    close: Vec<String>,
}

#[derive(Default)]
struct ChangeQueue {
    generation: u64,
    pending: VecDeque<ChangeDelivery>,
    dispatching: bool,
}

impl ChangeQueue {
    fn new_session(&mut self) {
        self.generation += 1;
        self.pending.clear();
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct AppNotice {
    pub id: String,
    pub text: String,
}

pub struct AppState {
    pub ai: crate::ai::Runtime,
    pub paths: Paths,
    change_sink: Mutex<Option<ChangeSink>>,
    changes: Mutex<ChangeQueue>,
    session: Mutex<Option<Session>>,
    admission: Mutex<SessionAdmission>,
    saves_cv: Condvar,
    /// Serializes graceful drain, flush and session removal.
    close_gate: Mutex<()>,
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
    /// failed). Keep these in display order until the main window acknowledges them.
    notices: Mutex<VecDeque<AppNotice>>,
    /// Serializes live-vault open, unlock, creation, and restore. Held from the
    /// read-only upgrade preflight through publishing the unlocked session.
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
            ai: crate::ai::Runtime::default(),
            paths,
            change_sink: Mutex::new(None),
            changes: Mutex::new(ChangeQueue::default()),
            session: Mutex::new(None),
            admission: Mutex::new(SessionAdmission::default()),
            saves_cv: Condvar::new(),
            close_gate: Mutex::new(()),
            attempts: Mutex::new(Attempts { failures: 0, wait_until: None }),
            ready: Mutex::new(HashSet::new()),
            flush: Mutex::new(Flush::default()),
            flush_cv: Condvar::new(),
            flush_gate: Mutex::new(()),
            picked_backup: Mutex::new(None),
            notices: Mutex::new(VecDeque::new()),
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

    fn begin_quit(&self) -> bool {
        !self.quitting.swap(true, Ordering::SeqCst)
    }

    /// Repeated reports of a pending notice share its place in the queue.
    pub fn set_notice(&self, text: String) {
        let mut notices = guard(&self.notices);
        if !notices.iter().any(|notice| notice.text == text) {
            notices.push_back(AppNotice { id: crate::model::new_id(), text });
        }
    }

    /// Reading does not consume the head: the window can reload before rendering it.
    pub fn pending_notice(&self) -> Option<AppNotice> {
        guard(&self.notices).front().cloned()
    }

    /// A late or repeated acknowledgement must never consume the next notice.
    pub fn acknowledge_notice(&self, id: &str) {
        let mut notices = guard(&self.notices);
        if notices.front().is_some_and(|notice| notice.id == id) {
            notices.pop_front();
        }
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
        guard(&self.ready).remove(label);
        let mut f = guard(&self.flush);
        if f.pending.remove(label) {
            self.flush_cv.notify_all();
        }
    }

    /// This may wait behind an off-thread vault operation. Window events call it on a worker.
    pub fn forget_window(&self, label: &str) {
        if let Some(session) = guard(&self.session).as_mut() {
            if !guard(&self.ready).contains(label) {
                session.coordination.forget_window(label);
            }
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
        // Stop lasts until Resume AI or a new app process, not across restarts.
        if !self.ai.started.swap(true, Ordering::SeqCst) {
            let mut settings=session.settings();
            if settings.ai.paused {
                settings.ai.paused=false;
                // Fail closed if the transient Stop flag cannot be reset.
                if session.put(crate::model::Item::Settings(settings)).is_err() {
                    self.ai.started.store(false, Ordering::SeqCst);
                }
            }
        }
        session.drain_changes(); // creation/unlock is followed by a complete window load
        session.clear_query_cache();
        let mut current = guard(&self.session);
        guard(&self.changes).new_session();
        *current = Some(session);
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
        let mut session = guard(&self.session);
        // Invalidate while owning this session. No new prompt can bind the new epoch to it.
        self.ai.invalidate();
        // Messages of automations hold row titles: they do not outlive the unlocked session.
        crate::commands::automation::clear_notices();
        if let Some(s) = session.as_mut() {
            self.ai.flush_reads(s, None);
            s.clear_query_cache();
        }
        guard(&self.changes).new_session();
        session.take()
    }

    fn accepts_background(&self) -> bool {
        guard(&self.admission).phase == SessionPhase::Running
    }

    fn background_session(&self) -> Result<MutexGuard<'_, Option<Session>>> {
        if !self.accepts_background() {
            return Err(Error::Locked);
        }
        let session = guard(&self.session);
        // An operation already queued when shutdown began must also yield.
        if !self.accepts_background() {
            return Err(Error::Locked);
        }
        Ok(session)
    }

    fn register_save(&self) -> Result<PendingSave<'_>> {
        let mut admission = guard(&self.admission);
        if admission.phase == SessionPhase::Sealed {
            return Err(Error::Locked);
        }
        let during_flush = admission.phase == SessionPhase::Flushing;
        if during_flush {
            admission.flush_saves += 1;
        } else {
            admission.prior_saves += 1;
        }
        Ok(PendingSave { state: self, during_flush })
    }

    fn advance_drain(&self, phase: SessionPhase) {
        let mut admission = guard(&self.admission);
        admission.phase = phase;
        // New saves during the window round get a separate count. They can't
        // keep postponing its start, and sealing admission still drains them.
        while admission.prior_saves != 0
            || (phase == SessionPhase::Sealed && admission.flush_saves != 0)
        {
            admission = self.saves_cv.wait(admission).unwrap_or_else(|e| e.into_inner());
        }
    }

    /// Stops new background operations before waiting for the current session holder.
    /// All admitted saves complete before the session is detached, even if a window doesn't ack.
    fn close_after_flushing(&self, allow_open: bool, flush: impl FnOnce()) -> Option<Option<Session>> {
        let _close = guard(&self.close_gate);
        guard(&self.admission).phase = SessionPhase::Draining;
        let _drain = SessionDrain(self);
        {
            let session = guard(&self.session);
            if !allow_open && session.as_ref().is_some_and(Session::is_open) {
                return None;
            }
        }
        self.advance_drain(SessionPhase::Flushing);
        flush();
        // The window deadline never preempts a write Rust already admitted.
        self.advance_drain(SessionPhase::Sealed);
        Some(self.take_session())
    }

    pub fn with<T>(&self, f: impl FnOnce(&Session) -> Result<T>) -> Result<T> {
        match self.background_session()?.as_ref() {
            Some(s) => f(s),
            None => Err(Error::Locked),
        }
    }

    pub fn with_mut<T>(&self, f: impl FnOnce(&mut Session) -> Result<T>) -> Result<T> {
        self.with_actor(&crate::actions::Actor::Person, f)
    }

    pub fn with_actor<T>(&self, actor: &crate::actions::Actor, f: impl FnOnce(&mut Session) -> Result<T>) -> Result<T> {
        self.write_session(actor, None, f)
    }

    /// The editor and its flush hooks can save during a graceful drain.
    pub(crate) fn with_save<T>(&self, f: impl FnOnce(&mut Session) -> Result<T>) -> Result<T> {
        let pending = self.register_save()?;
        self.write_session(&crate::actions::Actor::Person, Some(pending), f)
    }

    fn write_session<T>(
        &self,
        actor: &crate::actions::Actor,
        pending: Option<PendingSave<'_>>,
        f: impl FnOnce(&mut Session) -> Result<T>,
    ) -> Result<T> {
        let mut session = if pending.is_some() {
            guard(&self.session)
        } else {
            self.background_session()?
        };
        let s = session.as_mut().ok_or(Error::Locked)?;
        let result = s.run_as(actor, f);
        // Some old cores perform several transactions. Publish durable partial writes even
        // if a later step failed; they must never leak into the next command's signal.
        if let Some(changes) = s.drain_changes() {
            let close = changes.ids.iter().filter(|id| {
                matches!(s.get(id), Some(crate::model::Item::Sticky(st)) if st.trashed_at.is_some())
            }).cloned().collect();
            // Enqueue under the session lock to preserve commit order. Delivery
            // holds neither lock, so callbacks may read, write or lock the vault.
            let mut queue = guard(&self.changes);
            let generation = queue.generation;
            queue.pending.push_back(ChangeDelivery { generation, changes: Some(changes), close });
        }
        drop(session);
        // Event callbacks may themselves access or lock the vault.
        drop(pending);
        self.dispatch_changes();
        result
    }

    /// One ordered, reentrant pump. Failures cannot change a durable command's
    /// result or skip sticky cleanup; the next command/bind retries the head.
    fn dispatch_changes(&self) {
        {
            let mut queue = guard(&self.changes);
            if queue.dispatching { return; }
            queue.dispatching = true;
        }
        loop {
            let sink = guard(&self.change_sink).clone();
            let mut delivery = {
                let mut queue = guard(&self.changes);
                if sink.is_none() || queue.pending.is_empty() {
                    queue.dispatching = false;
                    return;
                }
                queue.pending.pop_front().unwrap()
            };
            let sink = sink.unwrap();
            if let Some(changes) = delivery.changes.take() {
                let sent = catch_unwind(AssertUnwindSafe(|| (sink.changed)(changes.clone())));
                if !matches!(sent, Ok(Ok(()))) { delivery.changes = Some(changes); }
            }
            let mut failed_close = Vec::new();
            for id in delivery.close {
                // A callback may have locked or replaced the session. Old work
                // must never close a new session's windows or be retried there.
                if guard(&self.changes).generation != delivery.generation { break; }
                if catch_unwind(AssertUnwindSafe(|| (sink.close_sticky)(&id))).is_err() {
                    failed_close.push(id);
                }
            }
            delivery.close = failed_close;
            if delivery.changes.is_some() || !delivery.close.is_empty() {
                let mut queue = guard(&self.changes);
                if queue.generation == delivery.generation {
                    queue.pending.push_front(delivery);
                    queue.dispatching = false;
                    return;
                }
            }
        }
    }

    pub fn bind_events(
        &self,
        changed: impl Fn(crate::changes::ItemsChanged) -> Result<()> + Send + Sync + 'static,
        close_sticky: impl Fn(&str) + Send + Sync + 'static,
    ) {
        *guard(&self.change_sink) = Some(ChangeSink {
            changed: Arc::new(changed),
            close_sticky: Arc::new(close_sticky),
        });
        self.dispatch_changes();
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
/// round, or until the deadline passes. Does nothing when no window is expected to
/// answer.
///
/// Never call this on the main (event-loop) thread: the windows' `flush_done`
/// calls need that thread to be free.
fn request_flush(app: &AppHandle, for_close: bool) {
    let state = app.state::<AppState>();
    if !for_close && !state.accepts_background() {
        return;
    }
    let _gate = guard(&state.flush_gate);
    if !for_close && !state.accepts_background() {
        return; // the closing thread will flush the windows
    }
    if !state.is_unlocked() {
        return; // another lock already finished while we waited for the gate
    }
    let round = {
        let open = app.webview_windows();
        let mut ready = guard(&state.ready);
        ready.retain(|label| open.contains_key(label));
        if ready.is_empty() {
            return;
        }
        // Destruction either removes the label before this snapshot or drops
        // it from this round in unmark_ready.
        guard(&state.flush).begin(ready.clone())
    };
    let _ = app.emit("vault-locking", round);
    let deadline = Instant::now() + FLUSH_TIMEOUT;
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

/// Restore reloads the UI after the file swap. Quit is ending the UI session.
fn emits_vault_locked(reason: &str) -> bool {
    !matches!(reason, "restore" | "quit")
}

fn hide_main(app: &AppHandle, state: &AppState, quit: bool) -> bool {
    if !quit && !state.tray_hidden() {
        return false;
    }
    let hidden = app.get_webview_window("main").is_none_or(|window| window.hide().is_ok());
    if !hidden && !quit {
        state.set_tray_hidden(false);
    }
    hidden
}

/// Asks every window to write its pending edits, then waits (up to the usual lock time). Used before
/// closing sticky notes without locking ("Close all"). Same rule as `request_flush`: not on the main thread.
pub fn flush_windows(app: &AppHandle) {
    request_flush(app, false);
}

/// Locks the vault: first lets every window save its pending edits (unless the
/// drive was removed, when saving can't work), then wipes keys and decrypted
/// data from memory, closes sticky notes and calculators, tells the UI to reset itself, and only
/// then does the slow work (the backup).
///
/// Waits for the windows, so call it only from a thread other than the main
/// one (an `async` command, the monitor thread, the close handler's thread).
///
pub fn lock(app: &AppHandle, reason: &str) {
    let _ = lock_with_notice(app, reason);
}

fn lock_with_notice(app: &AppHandle, reason: &str) -> Option<crate::native_confirm::NativePrompt> {
    let state = app.state::<AppState>();
    // A vault with security off never locks by itself or by a button. Only a quit and a
    // restore close it (they have to, to save and to swap the file).
    let session = if reason == "drive-removed" {
        if guard(&state.session).as_ref().is_some_and(Session::is_open) {
            return None;
        }
        crate::commands::import_notes::cancel_running();
        state.take_session()
    } else {
        let allow_open = matches!(reason, "quit" | "restore");
        if allow_open {
            crate::commands::import_notes::cancel_running();
        }
        state.close_after_flushing(allow_open, || {
            // An import in progress must stop reading and converting files as the vault closes.
            crate::commands::import_notes::cancel_running();
            request_flush(app, true);
        })?
    };
    if session.is_some() {
        crate::logging::locked_event(reason);
    }
    // Protect the locked screen and hide while a checked backup is made. A
    // clean, open vault can exit directly after flushing.
    let hide_for_quit = reason == "quit" && session.as_ref().is_some_and(|s| s.dirty || !s.is_open());
    let hidden = (reason == "tray" || hide_for_quit) && hide_main(app, &state, reason == "quit");
    ui::close_stickies(app);
    ui::close_calculators(app);
    if emits_vault_locked(reason) || (hide_for_quit && !hidden) {
        let _ = app.emit("vault-locked", reason.to_string());
    }
    // Slow parts after that.
    let made = session.as_ref().filter(|s| s.dirty).map(|s| backup::make_backup(&state.paths, s));
    let backup_failure = if let Some(Err(e)) = made {
        if reason == "quit" {
            close_backup_failure(app, &state, &e).map(|text| {
                let appearance = session.as_ref().map(|s| s.settings().into());
                crate::native_confirm::NativePrompt::backup_failure(text, appearance)
            })
        } else {
            report_backup_failure(app, &state, &e);
            None
        }
    } else {
        None
    };
    drop(session); // zeroizes the data key and all items
    backup_failure
}

/// Quits Notaya: locks (saving the windows' edits and making the backup) and exits.
///
/// Waits for the windows, so call it only from a thread other than the main one.
pub fn quit(app: &AppHandle) -> bool {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    let state = app.state::<AppState>();
    // Repeated window, app-menu and tray quit requests share this shutdown.
    if !state.begin_quit() {
        return true;
    }
    // Whatever fails or panics here, the exit must still happen.
    let backup_failure = catch_unwind(AssertUnwindSafe(|| lock_with_notice(app, "quit"))).ok().flatten();
    if let Some(prompt) = backup_failure.filter(|_| state.paths.vault.exists()) {
        let _ = catch_unwind(AssertUnwindSafe(|| state.ai.native.show_notice(prompt)));
    }
    app.exit(0);
    true
}

/// Hide the main window once pending edits have been saved. A protected vault locks and
/// closes its other windows too. Finish any due backup on this worker after the window hides.
pub fn hide_to_tray(app: &AppHandle) {
    let state = app.state::<AppState>();
    // Set before locking so a reload or an in-flight unlock cannot reopen the hidden vault.
    let protected = {
        let _gate = guard(&state.tray_gate);
        state.set_tray_hidden(true);
        state.with(|s| Ok(!s.is_open())).unwrap_or(false)
    };
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if protected {
            lock(app, "tray");
        } else {
            flush_windows(app);
            hide_main(app, &state, false);
            backup_open_vault(app, &state);
        }
    }));
}

/// Keep the ordinary end-of-session backup behavior for changed open vaults.
fn backup_open_vault(app: &AppHandle, state: &AppState) {
    let result = state.with_mut(|s| {
        if s.dirty {
            backup::make_backup(&state.paths, s)?;
            s.dirty = false;
        }
        Ok(())
    });
    if let Err(error) = result {
        if !matches!(error, Error::Locked) {
            report_backup_failure(app, state, &error);
        }
    }
}

/// Tells the owner that an automatic backup could not be made (the release exe has no
/// console, so a message on stderr would never be seen). The event wakes the
/// main window, and the queue covers a missed event during reload.
pub(crate) fn report_backup_failure(app: &AppHandle, state: &AppState, e: &Error) {
    let text = backup_failure_text(e, state.paths.vault.exists());
    state.set_notice(text);
    let _ = app.emit_to("main", "app-notice", ());
}

fn close_backup_failure(app: &AppHandle, state: &AppState, e: &Error) -> Option<String> {
    if state.paths.vault.exists() {
        Some(backup_failure_text(e, true))
    } else {
        report_backup_failure(app, state, e);
        None
    }
}

fn backup_failure_text(e: &Error, vault_exists: bool) -> String {
    eprintln!("backup failed: {e}");
    backup::failure_notice(e, vault_exists)
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

    fn wait_for_admission(state: &AppState, reached: impl Fn(&SessionAdmission) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !reached(&guard(&state.admission)) {
            assert!(Instant::now() < deadline, "session admission did not reach the expected state");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn shutdown_drains_a_long_vault_operation_before_flushing_a_pending_editor_save() {
        use crate::{actions::Actor, model::Item};
        use std::sync::mpsc;

        let (session, paths) = crate::changes::tests::fixture();
        let page_id = session.items().find(|item| matches!(item, Item::Page(_))).unwrap().id().to_owned();
        let base = session.rev(&page_id);
        let state = AppState::new(paths);
        state.set_session(session);
        let (holding, held) = mpsc::channel();
        let (release, released) = mpsc::channel();
        let (emitting, emitted) = mpsc::channel();
        let limit = Duration::from_secs(5);

        std::thread::scope(|threads| {
            let state_ref = &state;
            let page = &page_id;
            let long_op = threads.spawn(move || state_ref.with_mut(|_| {
                holding.send(()).unwrap();
                released.recv_timeout(limit).unwrap();
                Ok(())
            }));
            held.recv_timeout(limit).unwrap();

            let save = threads.spawn(move || state_ref.with_save(|s| s.person_save(page, Some(base), |s| {
                crate::commands::notes::save_page_core(
                    s, &Actor::Person, page.to_owned(), "Saved draft".into(),
                    String::new(), "Edited while busy".into(),
                )?;
                Ok(())
            })));
            wait_for_admission(&state, |a| a.prior_saves == 1);
            let close = threads.spawn(move || state_ref.close_after_flushing(false, || {
                let session = guard(&state_ref.session);
                let Some(Item::Page(saved)) = session.as_ref().unwrap().get(page) else {
                    panic!("missing saved page");
                };
                assert_eq!(saved.text, "Edited while busy", "save committed before the UI round");
                emitting.send(()).unwrap();
            }).unwrap().unwrap());
            wait_for_admission(&state, |a| a.phase == SessionPhase::Draining);
            assert!(matches!(state.with(|_| Ok(())), Err(Error::Locked)));
            assert!(matches!(state.with_mut(|_| Ok(())), Err(Error::Locked)));
            assert!(matches!(emitted.try_recv(), Err(mpsc::TryRecvError::Empty)));

            release.send(()).unwrap();
            emitted.recv_timeout(limit).unwrap();
            long_op.join().unwrap().unwrap();
            let detached = close.join().unwrap();
            save.join().unwrap().unwrap();
            let Some(Item::Page(saved)) = detached.get(&page_id) else { panic!("missing saved page") };
            assert_eq!(saved.text, "Edited while busy");
            drop(detached);
            assert!(!state.is_unlocked());

            let reopened = Session::unlock(&state.paths.vault, "throwaway", None).unwrap();
            let Some(Item::Page(saved)) = reopened.get(&page_id) else { panic!("missing reopened page") };
            assert_eq!(saved.text, "Edited while busy");
        });
    }

    #[test]
    fn a_save_admitted_during_flush_finishes_without_a_window_acknowledgement() {
        use crate::{actions::Actor, model::Item};
        use std::sync::mpsc;

        let (session, paths) = crate::changes::tests::fixture();
        let page_id = session.items().find(|item| matches!(item, Item::Page(_))).unwrap().id().to_owned();
        let state = AppState::new(paths);
        state.set_session(session);
        let (run_save, may_save) = mpsc::channel();
        let (admitted, registered) = mpsc::channel();
        let (release, released) = mpsc::channel();
        let limit = Duration::from_secs(5);

        std::thread::scope(|threads| {
            let state_ref = &state;
            let page = &page_id;
            let save = threads.spawn(move || {
                may_save.recv_timeout(limit).unwrap();
                let pending = state_ref.register_save().unwrap();
                admitted.send(()).unwrap();
                released.recv_timeout(limit).unwrap();
                state_ref.write_session(&Actor::Person, Some(pending), |s| {
                    let Item::Page(mut updated) = s.get(page).unwrap().clone() else { panic!("missing page") };
                    updated.text = "Saved before the vault closed".into();
                    s.put(Item::Page(updated))
                })
            });
            let close = threads.spawn(move || state_ref.close_after_flushing(false, || {
                run_save.send(()).unwrap();
                registered.recv_timeout(limit).unwrap();
                // The window stopped responding after Rust admitted its write.
            }).unwrap().unwrap());
            wait_for_admission(&state, |a| a.phase == SessionPhase::Sealed);
            assert!(matches!(state.with_save(|_| Ok(())), Err(Error::Locked)));

            release.send(()).unwrap();
            let detached = close.join().unwrap();
            save.join().unwrap().unwrap();
            let Some(Item::Page(saved)) = detached.get(&page_id) else { panic!("missing saved page") };
            assert_eq!(saved.text, "Saved before the vault closed");
        });
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
    fn restore_and_quit_skip_the_vault_locked_event() {
        for reason in ["manual", "idle", "sleep", "screen-locked", "minimized", "drive-removed", "tray"] {
            assert!(emits_vault_locked(reason), "{reason}");
        }
        assert!(!emits_vault_locked("restore"));
        assert!(!emits_vault_locked("quit"));
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
    fn upgrade_and_backup_notices_survive_reads_and_drain_in_order() {
        let root = std::env::temp_dir().join(format!("notaya-state-{}", crate::model::new_id()));
        let state = AppState::new(Paths::at(&root));
        assert_eq!(state.pending_notice(), None);
        let upgrade = "Notaya saved vault-before-1.6.nota in your data folder before upgrading this vault to 1.6.";
        let failure = backup::failure_notice(&Error::Corrupt, true);
        state.set_notice(upgrade.into());
        let first = state.pending_notice().unwrap();
        assert_eq!(first.text, upgrade);

        // A backup can fail while the window is preparing the first toast.
        state.set_notice(failure.clone());
        state.set_notice(upgrade.into());
        state.set_notice(failure.clone());
        assert_eq!(state.pending_notice(), Some(first.clone()), "still available after reload");
        state.acknowledge_notice(&first.id);
        let second = state.pending_notice().unwrap();
        assert_eq!(second.text, failure);
        state.acknowledge_notice(&first.id);
        assert_eq!(state.pending_notice(), Some(second.clone()), "stale ack leaves the next notice");
        state.acknowledge_notice(&second.id);
        assert_eq!(state.pending_notice(), None, "both notices were drained exactly once");
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
                let state = emitting_state.upgrade().unwrap();
                let session = state.session.try_lock().expect("delivery releases the session lock");
                assert!(session.as_ref().unwrap().in_bin(&changed_id));
                drop(session);
                assert!(state.is_unlocked());
                state.with(|s| { assert!(s.in_bin(&changed_id)); Ok(()) }).unwrap();
                state.with_mut(|_| Ok(())).unwrap();
                changed_calls.lock().unwrap().push("changed");
                Ok(())
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
    fn with_mut_groups_nested_journals_and_publishes_partial_commits_once() {
        use crate::{actions::Actor, model::Item};
        let (session, paths) = crate::changes::tests::fixture();
        let id = session.items().find(|i| matches!(i, Item::Page(_))).unwrap().id().to_string();
        let base = session.rev(&id);
        let state = AppState::new(paths);
        state.set_session(session);
        let received = Arc::new(Mutex::new(Vec::new()));
        let events = received.clone();
        state.bind_events(move |event| { events.lock().unwrap().push(event); Ok(()) }, |_| {});
        let result: Result<()> = state.with_mut(|s| {
            for title in ["first durable step", "second durable step"] {
                s.run_as(&Actor::Person, |s| {
                    let Item::Page(mut page) = s.get(&id).unwrap().clone() else { panic!() };
                    page.title = title.into();
                    page.favorite = !page.favorite; // structural writes are separate activity entries
                    s.put(Item::Page(page))?;
                    s.snapshot_page(&id)
                })?;
            }
            Err(Error::invalid("later step failed"))
        });
        assert!(result.is_err());
        let request_set = state.with(|s| {
            let entries = s.activity_list(None, None, 2)?;
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].change_set, entries[1].change_set);
            assert!(entries.iter().all(|e| e.actor == "person"));
            for title in ["first durable step", "second durable step"] {
                assert!(s.history_list(&id)?.iter().any(|v| v.title == title && v.change_set == entries[0].change_set));
            }
            s.activity_verify()?;
            Ok(entries[0].change_set.clone())
        }).unwrap();
        assert_eq!(received.lock().unwrap().len(), 1);
        assert_eq!(received.lock().unwrap()[0].revs[&id], base + 2);
        state.with_mut(|s| {
            let Item::Page(mut page) = s.get(&id).unwrap().clone() else { panic!() };
            page.title = "next request".into();
            page.favorite = !page.favorite;
            s.put(Item::Page(page))
        }).unwrap();
        state.with(|s| {
            assert_ne!(s.activity_list(None, None, 1)?[0].change_set, request_set);
            s.activity_verify()
        }).unwrap();
        assert_eq!(received.lock().unwrap().len(), 2);
        assert_eq!(received.lock().unwrap()[1].revs[&id], base + 3);
    }

    #[test]
    fn panicking_and_failing_change_sink_preserves_data_closes_sticky_and_retries() {
        use crate::actions::Actor;
        use crate::model::Item;
        use std::sync::atomic::AtomicUsize;
        let (session, paths) = crate::changes::tests::fixture();
        let id = session.items().find(|i| matches!(i, Item::Sticky(_))).unwrap().id().to_string();
        let state = AppState::new(paths);
        state.set_session(session);
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempt = attempts.clone();
        let closed = Arc::new(AtomicUsize::new(0));
        let close = closed.clone();
        let received = Arc::new(Mutex::new(Vec::new()));
        let events = received.clone();
        state.bind_events(move |change| {
            events.lock().unwrap().push(serde_json::to_value(change).unwrap());
            match attempt.fetch_add(1, Ordering::SeqCst) {
                0 => panic!("injected notification panic"),
                1 => Err(Error::invalid("injected notification error")),
                _ => Ok(()),
            }
        }, move |_| { close.fetch_add(1, Ordering::SeqCst); });
        state.with_mut(|s| crate::commands::stickies::delete_sticky_core(s, &Actor::Person, id.clone())).unwrap();
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
        assert_eq!(closed.load(Ordering::SeqCst), 1, "panic cannot skip sticky cleanup");
        assert!(!state.session.is_poisoned());
        assert!(state.with(|s| Ok(s.in_bin(&id))).unwrap());
        state.with_mut(|_| Ok(())).unwrap(); // returned sink error is retried too
        state.with_mut(|_| Ok(())).unwrap();
        let received = received.lock().unwrap();
        assert_eq!(received.len(), 3);
        assert_eq!(received[0], received[1]);
        assert_eq!(received[1], received[2]);
        assert_eq!(closed.load(Ordering::SeqCst), 1, "successful cleanup is not repeated");
        drop(received);
        drop(state.take_session().unwrap()); // locking remains usable
        assert!(!state.is_unlocked());
        let reopened = Session::unlock(&state.paths.vault, "throwaway", None).unwrap();
        assert!(reopened.in_bin(&id), "bin move was durable despite sink failure");
        state.set_session(reopened);
        state.with_mut(|_| Ok(())).unwrap();
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn change_delivery_preserves_concurrent_and_reentrant_commit_order() {
        use crate::model::Item;
        use std::sync::mpsc;
        let (session, paths) = crate::changes::tests::fixture();
        let id = session.items().find(|i| matches!(i, Item::Page(_))).unwrap().id().to_string();
        let base = session.rev(&id);
        let state = Arc::new(AppState::new(paths));
        state.set_session(session);
        let (entered, started) = mpsc::channel();
        let (resume, wait) = mpsc::channel();
        let wait = Mutex::new(wait);
        let events = Arc::new(Mutex::new(Vec::new()));
        let received = events.clone();
        let weak = Arc::downgrade(&state);
        let changed_id = id.clone();
        state.bind_events(move |change| {
            let rev = change.revs[&changed_id];
            received.lock().unwrap().push(rev);
            let state = weak.upgrade().unwrap();
            assert!(state.with(|s| Ok(s.get(&changed_id).is_some())).unwrap());
            if rev == base + 1 {
                entered.send(()).unwrap();
                wait.lock().unwrap().recv_timeout(Duration::from_secs(5)).unwrap();
                // A synchronous native event listener writes during delivery.
                state.with_mut(|s| s.put(s.get(&changed_id).unwrap().clone())).unwrap();
            }
            Ok(())
        }, |_| {});
        let writing = state.clone();
        let first_id = id.clone();
        let first = std::thread::spawn(move || writing.with_mut(|s| s.put(s.get(&first_id).unwrap().clone())));
        started.recv_timeout(Duration::from_secs(5)).unwrap();
        state.with_mut(|s| s.put(s.get(&id).unwrap().clone())).unwrap();
        resume.send(()).unwrap();
        first.join().unwrap().unwrap();
        assert_eq!(*events.lock().unwrap(), vec![base + 1, base + 2, base + 3]);
    }

    #[test]
    fn lock_and_session_replacement_discard_failed_generation_notifications() {
        use crate::model::Item;
        let (session, paths) = crate::changes::tests::fixture();
        let id = session.items().find(|i| matches!(i, Item::Page(_))).unwrap().id().to_string();
        let state = AppState::new(paths);
        state.set_session(session);
        state.bind_events(|_| Err(Error::invalid("injected failure")), |_| {});
        state.with_mut(|s| s.put(s.get(&id).unwrap().clone())).unwrap();
        assert_eq!(guard(&state.changes).pending.len(), 1);
        drop(state.take_session().unwrap());
        assert!(guard(&state.changes).pending.is_empty());
        let session = Session::unlock(&state.paths.vault, "throwaway", None).unwrap();
        state.set_session(session);
        state.with_mut(|s| s.put(s.get(&id).unwrap().clone())).unwrap();
        assert_eq!(guard(&state.changes).pending.len(), 1);
        let session = state.take_session().unwrap();
        state.set_session(session);
        let events = Arc::new(Mutex::new(Vec::new()));
        let received = events.clone();
        state.bind_events(move |change| { received.lock().unwrap().push(change); Ok(()) }, |_| {});
        assert!(events.lock().unwrap().is_empty());
        state.with_mut(|s| s.put(s.get(&id).unwrap().clone())).unwrap();
        assert_eq!(events.lock().unwrap().len(), 1);
    }

    #[test]
    fn empty_round_is_done_at_once() {
        let mut f = Flush::default();
        f.begin(HashSet::new());
        assert!(f.is_done());
    }
}
