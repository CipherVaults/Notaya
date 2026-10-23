//! Shared app state, locking, and the background auto-lock monitor.

use std::collections::HashSet;
use std::process::Child;
use std::sync::{Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant, SystemTime};

use tauri::{AppHandle, Emitter, Manager};

use crate::error::{Error, Result};
use crate::portable::Paths;
use crate::vault::Session;
use crate::{keepassxc, system, ui};

const MAX_BACKUPS: usize = 10;

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

pub struct AppState {
    pub paths: Paths,
    session: Mutex<Option<Session>>,
    attempts: Mutex<Attempts>,
    /// The KeePassXC process Cipherleaf started, if any (amendment A9).
    pub keepassxc: Mutex<Option<Child>>,
    /// Labels of windows that have called `window_ready` (they listen for `vault-locking`).
    ready: Mutex<HashSet<String>>,
    flush: Mutex<Flush>,
    flush_cv: Condvar,
    /// Lets only one flush round run at a time.
    flush_gate: Mutex<()>,
}

fn guard<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    // A panic while holding the lock must not brick the app; recover the data.
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl AppState {
    pub fn new(paths: Paths) -> Self {
        AppState {
            paths,
            session: Mutex::new(None),
            attempts: Mutex::new(Attempts { failures: 0, wait_until: None }),
            keepassxc: Mutex::new(None),
            ready: Mutex::new(HashSet::new()),
            flush: Mutex::new(Flush::default()),
            flush_cv: Condvar::new(),
            flush_gate: Mutex::new(()),
        }
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

    pub fn set_session(&self, session: Session) {
        *guard(&self.session) = Some(session);
    }

    pub fn take_session(&self) -> Option<Session> {
        guard(&self.session).take()
    }

    pub fn with<T>(&self, f: impl FnOnce(&Session) -> Result<T>) -> Result<T> {
        match guard(&self.session).as_ref() {
            Some(s) => f(s),
            None => Err(Error::Locked),
        }
    }

    pub fn with_mut<T>(&self, f: impl FnOnce(&mut Session) -> Result<T>) -> Result<T> {
        match guard(&self.session).as_mut() {
            Some(s) => f(s),
            None => Err(Error::Locked),
        }
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

/// Locks the vault: first lets every window save its pending edits (unless the
/// drive was removed, when saving can't work), then wipes keys and decrypted
/// data from memory, closes sticky notes, tells the UI to reset itself, and only
/// then does the slow work (backup, asking KeePassXC to lock).
///
/// Waits for the windows, so call it only from a thread other than the main
/// one (an `async` command, the monitor thread, the close handler's thread).
pub fn lock(app: &AppHandle, reason: &str) {
    let state = app.state::<AppState>();
    if reason != "drive-removed" && state.is_unlocked() {
        request_flush(app, FLUSH_TIMEOUT);
    }
    let session = state.take_session();
    // The UI comes first: hide the overlay, close stickies, reset the windows.
    crate::embed::hide();
    ui::close_stickies(app);
    let _ = app.emit("vault-locked", reason.to_string());
    // Slow parts after that.
    let backup = session.as_ref().filter(|s| s.dirty).map(|s| rotate_backup(&state.paths, s));
    if let Some(Err(e)) = backup {
        eprintln!("backup failed: {e}");
    }
    drop(session); // zeroizes the data key and all items
    keepassxc::lock_if_running(&state.paths, &state.keepassxc);
}

fn rotate_backup(paths: &Paths, session: &Session) -> Result<()> {
    std::fs::create_dir_all(&paths.backups)?;
    let name = format!("vault-{}.clf", crate::model::now_ms());
    session.backup_to(&paths.backups.join(name))?;
    let mut backups: Vec<_> = std::fs::read_dir(&paths.backups)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("vault-") && n.ends_with(".clf"))
        })
        .collect();
    backups.sort();
    while backups.len() > MAX_BACKUPS {
        let _ = std::fs::remove_file(backups.remove(0));
    }
    Ok(())
}

/// Checks once a second for reasons to lock: idle time, Win+L, sleep/hibernate,
/// the USB drive being removed, or the window being minimized.
pub fn spawn_monitor(app: AppHandle) {
    std::thread::spawn(move || {
        let mut last_wall = SystemTime::now();
        let mut desktop_seen_unlocked = false;
        loop {
            std::thread::sleep(Duration::from_secs(1));
            let state = app.state::<AppState>();

            // A big jump in wall-clock time means the PC was asleep or hibernating.
            let now = SystemTime::now();
            let slept = now.duration_since(last_wall).map(|d| d > Duration::from_secs(20)).unwrap_or(true);
            last_wall = now;

            let Ok(settings) = state.with(|s| Ok(s.settings())) else {
                desktop_seen_unlocked = false;
                continue;
            };

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
    fn empty_round_is_done_at_once() {
        let mut f = Flush::default();
        f.begin(HashSet::new());
        assert!(f.is_done());
    }
}
