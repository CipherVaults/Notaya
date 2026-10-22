//! Shared app state, locking, and the background auto-lock monitor.

use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant, SystemTime};

use tauri::{AppHandle, Emitter, Manager};

use crate::error::{Error, Result};
use crate::portable::Paths;
use crate::vault::Session;
use crate::{clipboard, system, ui};

const MAX_BACKUPS: usize = 10;

struct Attempts {
    failures: u32,
    wait_until: Option<Instant>,
}

struct ClipGuard {
    sequence: u32,
    clear_at: Instant,
}

pub struct AppState {
    pub paths: Paths,
    session: Mutex<Option<Session>>,
    attempts: Mutex<Attempts>,
    clip: Mutex<Option<ClipGuard>>,
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
            clip: Mutex::new(None),
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

    /// Copies a secret and schedules it to be cleared.
    pub fn copy_secret(&self, text: &str, clear_after_secs: u32) -> Result<()> {
        let sequence = clipboard::set_secret(text)?;
        *guard(&self.clip) = Some(ClipGuard {
            sequence,
            clear_at: Instant::now() + Duration::from_secs(clear_after_secs as u64),
        });
        Ok(())
    }

    fn clear_clipboard(&self, only_if_due: bool) {
        let mut clip = guard(&self.clip);
        if let Some(g) = clip.as_ref() {
            if !only_if_due || Instant::now() >= g.clear_at {
                clipboard::clear_if_unchanged(g.sequence);
                *clip = None;
            }
        }
    }
}

/// Locks the vault: wipes keys and decrypted data from memory, closes sticky
/// notes, clears a copied secret, and tells the UI to reset itself.
pub fn lock(app: &AppHandle, reason: &str) {
    let state = app.state::<AppState>();
    if let Some(session) = state.take_session() {
        if session.dirty {
            if let Err(e) = rotate_backup(&state.paths, &session) {
                eprintln!("backup failed: {e}");
            }
        }
        drop(session); // zeroizes the data key and all items
    }
    state.clear_clipboard(false);
    ui::close_stickies(app);
    let _ = app.emit("vault-locked", reason.to_string());
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
            state.clear_clipboard(true);

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
