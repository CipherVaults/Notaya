//! Only this module knows Windows handles. Authorization lives in dispatch.
use super::{
    dispatch,
    wire::{self, ClientMessage, EventKind, ServerMessage},
};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager};

pub struct Rate {
    tokens: f64,
    at: Instant,
}
impl Rate {
    pub fn new() -> Self {
        Self {
            tokens: 200.,
            at: Instant::now(),
        }
    }
    pub fn take(&mut self) -> bool {
        let now = Instant::now();
        self.tokens = (self.tokens + now.duration_since(self.at).as_secs_f64() * 50.).min(200.);
        self.at = now;
        if self.tokens < 1. {
            false
        } else {
            self.tokens -= 1.;
            true
        }
    }
}
pub fn spawn(app: AppHandle) {
    #[cfg(windows)]
    std::thread::spawn(move || native::serve(app));
    #[cfg(not(windows))]
    let _ = app;
}

#[cfg(windows)]
mod native {
    use super::*;
    use std::{
        collections::VecDeque,
        io,
        path::Path,
        sync::{mpsc, Mutex},
    };
    use windows::{
        core::{PCWSTR, PWSTR},
        Win32::{
            Foundation::{
                CloseHandle, GetLastError, LocalFree, ERROR_PIPE_CONNECTED, HANDLE, HLOCAL,
            },
            Security::{
                Authorization::{
                    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
                    SDDL_REVISION_1,
                },
                GetTokenInformation, TokenUser, PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES,
                TOKEN_QUERY, TOKEN_USER,
            },
            Storage::FileSystem::{
                ReadFile, WriteFile, FILE_FLAG_FIRST_PIPE_INSTANCE, PIPE_ACCESS_DUPLEX,
            },
            System::{
                Pipes::{
                    ConnectNamedPipe, CreateNamedPipeW, PeekNamedPipe, PIPE_NOWAIT,
                    PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE,
                },
                Threading::{GetCurrentProcess, OpenProcessToken},
            },
        },
    };
    use zeroize::Zeroize;
    struct Handle(HANDLE);
    impl Drop for Handle {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
    fn invalid() -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, "invalid")
    }
    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }
    fn sid() -> io::Result<String> {
        unsafe {
            let mut token = HANDLE::default();
            OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)
                .map_err(|_| invalid())?;
            let token = Handle(token);
            let mut size = 0;
            let _ = GetTokenInformation(token.0, TokenUser, None, 0, &mut size);
            if size == 0 || size > 65536 {
                return Err(invalid());
            }
            let mut buffer = vec![0usize; (size as usize).div_ceil(std::mem::size_of::<usize>())];
            GetTokenInformation(
                token.0,
                TokenUser,
                Some(buffer.as_mut_ptr().cast()),
                size,
                &mut size,
            )
            .map_err(|_| invalid())?;
            let user = &*buffer.as_ptr().cast::<TOKEN_USER>();
            let mut text = PWSTR::null();
            ConvertSidToStringSidW(user.User.Sid, &mut text).map_err(|_| invalid())?;
            let result = text.to_string().map_err(|_| invalid());
            let _ = LocalFree(Some(HLOCAL(text.0.cast())));
            result
        }
    }
    fn listener(name: &str, sid: &str, first: bool) -> io::Result<Handle> {
        unsafe {
            let acl = wide(&format!("D:P(A;;GA;;;{sid})"));
            let mut sd = PSECURITY_DESCRIPTOR::default();
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                PCWSTR(acl.as_ptr()),
                SDDL_REVISION_1,
                &mut sd,
                None,
            )
            .map_err(|_| invalid())?;
            let sa = SECURITY_ATTRIBUTES {
                nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                lpSecurityDescriptor: sd.0,
                bInheritHandle: false.into(),
            };
            let path = wide(name);
            let mode = if first {
                PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE
            } else {
                PIPE_ACCESS_DUPLEX
            };
            let handle = CreateNamedPipeW(
                PCWSTR(path.as_ptr()),
                mode,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_NOWAIT | PIPE_REJECT_REMOTE_CLIENTS,
                5,
                65536,
                65536,
                0,
                Some(&sa),
            );
            let _ = LocalFree(Some(HLOCAL(sd.0)));
            if handle.is_invalid() {
                Err(invalid())
            } else {
                Ok(Handle(handle))
            }
        }
    }
    pub fn serve(app: AppHandle) {
        let Ok(sid) = sid() else {
            return;
        };
        let state = app.state::<crate::state::AppState>();
        let Ok(data) = std::fs::canonicalize(&state.paths.data) else {
            return;
        };
        let Ok(name) = wire::pipe_name(Path::new(&data), &sid) else {
            return;
        };
        let active = Arc::new(AtomicUsize::new(0));
        let mut pipe = None;
        let mut epoch = state.ai.epoch.load(Ordering::SeqCst);
        loop {
            if state.is_quitting() {
                break;
            }
            let current = state.ai.epoch.load(Ordering::SeqCst);
            if current != epoch {
                pipe = None;
                epoch = current;
            }
            let enabled = state
                .with(|s| Ok(s.settings().ai.enabled && !s.settings().ai.paused))
                .unwrap_or(false);
            if !enabled {
                pipe = None;
            } else if pipe.is_none() && active.load(Ordering::SeqCst) == 0 {
                match listener(&name, &sid, true) {
                    Ok(p) => pipe = Some(p),
                    Err(_) => {
                        state
                            .ai
                            .emit("ai-server-error", serde_json::json!({"code":"not_running"}));
                    }
                }
            }
            if active.load(Ordering::SeqCst) < 4 {
                if let Some(p) = pipe.as_ref() {
                    let connected = unsafe {
                        match ConnectNamedPipe(p.0, None) {
                            Ok(()) => true,
                            Err(_) => GetLastError() == ERROR_PIPE_CONNECTED,
                        }
                    };
                    if connected {
                        // Keep the old instance alive until its replacement exists: no squatting gap.
                        let next = listener(&name, &sid, false).ok();
                        let accepted = pipe.take().unwrap();
                        pipe = next;
                        active.fetch_add(1, Ordering::SeqCst);
                        let count = active.clone();
                        let app = app.clone();
                        // HANDLE is moved by its numeric value; one owner closes it on the worker.
                        let raw = accepted.0 .0 as usize;
                        std::mem::forget(accepted);
                        std::thread::spawn(move || {
                            let handle = Handle(HANDLE(raw as *mut _));
                            connection(&app, handle, epoch);
                            count.fetch_sub(1, Ordering::SeqCst);
                        });
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(25));
        }
    }
    fn encoded(message: &ServerMessage) -> io::Result<Vec<u8>> {
        let mut bytes = vec![];
        wire::write_frame(&mut bytes, message, wire::MAX_RESPONSE)?;
        Ok(bytes)
    }
    fn error(id: Option<u64>, code: &str) -> ServerMessage {
        ServerMessage::Error {
            id,
            code: code.into(),
            message: if code == "locked" {
                "Notaya is locked".into()
            } else {
                code.into()
            },
        }
    }
    fn connection(app: &AppHandle, handle: Handle, epoch: u64) {
        let state = app.state::<crate::state::AppState>();
        let mut input = Vec::new();
        let mut output: VecDeque<Vec<u8>> = VecDeque::new();
        let mut written = 0;
        let (tx, rx) = mpsc::sync_channel::<ServerMessage>(4);
        let cancelled = Arc::new(AtomicBool::new(false));
        let busy = Arc::new(AtomicBool::new(false));
        let pairing = Arc::new(Mutex::new(None::<super::super::pairing::Pairing>));
        let mut client = None::<(String, Instant)>;
        let mut greeted = false;
        let mut rate = Rate::new();
        let mut closing = false;
        let started = Instant::now();
        let mut closing_at = None::<Instant>;
        loop {
            if input.len() > wire::MAX_REQUEST + 65540
                || output.iter().map(Vec::len).sum::<usize>() > wire::MAX_RESPONSE * 2
            {
                break;
            }
            if state.ai.epoch.load(Ordering::SeqCst) != epoch || !state.is_unlocked() {
                cancelled.store(true, Ordering::SeqCst);
                if !closing {
                    let reason = if !state.is_unlocked() {
                        "locked"
                    } else {
                        "stopped"
                    };
                    // Discard queued note bodies before the final fixed response.
                    // A partially sent frame cannot be replaced by another frame.
                    if written != 0 {
                        break;
                    }
                    for bytes in &mut output {
                        bytes.zeroize();
                    }
                    output.clear();
                    written = 0;
                    closing = true;
                    closing_at = Some(Instant::now());
                    let kind = if reason == "locked" {
                        EventKind::Locked
                    } else {
                        EventKind::Stopped
                    };
                    if let Ok(bytes) = encoded(&ServerMessage::Event {
                        kind,
                        id: None,
                        level: None,
                    }) {
                        output.push_back(bytes);
                    }
                    if let Ok(bytes) = encoded(&error(None, reason)) {
                        output.push_back(bytes);
                    }
                }
            }
            if !closing {
                let old = pairing.lock().unwrap().clone();
                if let Some(old) = old {
                    let current = state
                        .with(|s| {
                            Ok(s.settings()
                                .ai
                                .pairings
                                .into_iter()
                                .find(|p| p.id == old.id && p.token_hash == old.token_hash))
                        })
                        .ok()
                        .flatten();
                    if let Some(current) = current {
                        if current.level != old.level {
                            if let Ok(bytes) = encoded(&ServerMessage::Event {
                                kind: EventKind::LevelChanged,
                                id: None,
                                level: Some(current.level),
                            }) {
                                output.push_back(bytes);
                            }
                            *pairing.lock().unwrap() = Some(current);
                        }
                    } else {
                        cancelled.store(true, Ordering::SeqCst);
                        if written != 0 {
                            break;
                        }
                        for bytes in &mut output {
                            bytes.zeroize();
                        }
                        output.clear();
                        output.push_back(encoded(&error(None, "not_paired")).unwrap());
                        closing = true;
                        closing_at = Some(Instant::now());
                    }
                }
            }
            if !closing
                && !busy.load(Ordering::SeqCst)
                && pairing.lock().unwrap().is_none()
                && client
                    .as_ref()
                    .is_some_and(|(_, at)| at.elapsed() > Duration::from_millis(250))
            {
                let (name, _) = client.take().unwrap();
                busy.store(true, Ordering::SeqCst);
                let pairing = pairing.clone();
                let busy = busy.clone();
                let tx = tx.clone();
                let stop = cancelled.clone();
                let app = app.clone();
                std::thread::spawn(move || {
                    let state = app.state::<crate::state::AppState>();
                    let allowed = state.ai.confirms.request(
                        "pairing",
                        serde_json::json!({"client":name}),
                        crate::actions::Confirm::Ask,
                        |v| state.ai.emit("ai-pairing", v),
                        || {
                            stop.load(Ordering::SeqCst)
                                || state.ai.epoch.load(Ordering::SeqCst) != epoch
                        },
                    );
                    if allowed {
                        let (p, token) = super::super::pairing::create(name);
                        let saved = state
                            .with_mut(|s| {
                                let mut settings = s.settings();
                                if !settings.ai.enabled
                                    || settings.ai.paused
                                    || stop.load(Ordering::SeqCst)
                                    || state.ai.epoch.load(Ordering::SeqCst) != epoch
                                {
                                    return Err(crate::error::Error::invalid("off"));
                                }
                                if settings.ai.pairings.len() >= 32 {
                                    return Err(crate::error::Error::invalid("too_big"));
                                }
                                settings.ai.pairings.push(p.clone());
                                s.put(crate::model::Item::Settings(settings))
                            })
                            .is_ok();
                        if saved {
                            *pairing.lock().unwrap() = Some(p.clone());
                            let _ = tx.try_send(ServerMessage::PairingResult {
                                pairing_id: p.id,
                                token: token.to_string(),
                            });
                            let _ = tx.try_send(ServerMessage::Welcome {
                                notaya_version: env!("CARGO_PKG_VERSION").into(),
                                level: p.level,
                                locked: false,
                            });
                        } else {
                            let _ = tx.try_send(error(None, "pairing_refused"));
                        }
                    } else {
                        let _ = tx.try_send(error(None, "pairing_refused"));
                    }
                    busy.store(false, Ordering::SeqCst);
                });
            }
            if started.elapsed() > Duration::from_secs(300) && pairing.lock().unwrap().is_none() {
                break;
            }
            if !closing {
                while let Ok(message) = rx.try_recv() {
                    let bytes = match encoded(&message) {
                        Ok(bytes) => bytes,
                        Err(_) => {
                            let id = match message {
                                ServerMessage::Result { id, .. } => Some(id),
                                _ => None,
                            };
                            encoded(&error(id, "too_big")).unwrap()
                        }
                    };
                    output.push_back(bytes);
                }
            }
            if let Some(bytes) = output.front() {
                let mut n = 0;
                let result = unsafe {
                    WriteFile(
                        handle.0,
                        Some(&bytes[written..bytes.len().min(written + 32768)]),
                        Some(&mut n),
                        None,
                    )
                };
                if result.is_err() {
                    break;
                }
                written += n as usize;
                if written == bytes.len() {
                    if let Some(mut bytes) = output.pop_front() {
                        bytes.zeroize();
                    }
                    written = 0;
                }
            }
            if closing {
                if output.is_empty()
                    || closing_at.is_some_and(|at| at.elapsed() > Duration::from_secs(1))
                {
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            let mut available = 0;
            if unsafe { PeekNamedPipe(handle.0, None, 0, None, Some(&mut available), None) }
                .is_err()
            {
                break;
            }
            if available > 0 {
                let mut bytes = vec![0; available.min(65536) as usize];
                let mut n = 0;
                if unsafe { ReadFile(handle.0, Some(&mut bytes), Some(&mut n), None) }.is_err() {
                    break;
                }
                input.extend_from_slice(&bytes[..n as usize]);
            }
            if input.len() >= 4 {
                let length = u32::from_le_bytes(input[..4].try_into().unwrap()) as usize;
                if length == 0 || length > wire::MAX_REQUEST {
                    break;
                }
                if input.len() >= length + 4 {
                    let mut frame = input.drain(..length + 4).collect::<Vec<_>>();
                    let message = wire::read_frame::<_, ClientMessage>(
                        &mut frame.as_slice(),
                        wire::MAX_REQUEST,
                    );
                    frame.zeroize();
                    let Ok(message) = message else {
                        break;
                    };
                    if !rate.take() {
                        output.push_back(encoded(&error(None, "rate_limited")).unwrap());
                        continue;
                    }
                    match message {
                        ClientMessage::Hello {
                            bridge_version,
                            client: client_name,
                        } => {
                            if greeted
                                || bridge_version.len() > 128
                                || client_name.is_empty()
                                || client_name.len() > 256
                                || client_name.chars().any(char::is_control)
                            {
                                break;
                            }
                            greeted = true;
                            // The bridge uses Off welcome to begin pairing; approved auth below raises it.
                            output.push_back(
                                encoded(&ServerMessage::Welcome {
                                    notaya_version: env!("CARGO_PKG_VERSION").into(),
                                    level: super::super::rules::Level::Off,
                                    locked: false,
                                })
                                .unwrap(),
                            );
                            // Pairing starts only if no auth arrives; the bridge sends auth for an existing token.
                            // Store the label until the post-welcome handshake has been observed.
                            state
                                .ai
                                .emit("ai-connection", serde_json::json!({"client":client_name}));
                            client = Some((client_name, Instant::now()));
                        }
                        ClientMessage::Auth {
                            pairing_id,
                            mut token,
                        } => {
                            if !greeted
                                || busy.load(Ordering::SeqCst)
                                || pairing.lock().unwrap().is_some()
                            {
                                token.zeroize();
                                break;
                            }
                            client = None;
                            let found = state
                                .with(|s| {
                                    Ok(super::super::pairing::authenticate(
                                        &s.settings().ai,
                                        &pairing_id,
                                        &token,
                                    ))
                                })
                                .ok()
                                .flatten();
                            token.zeroize();
                            if let Some(p) = found {
                                let _ = state.with_mut(|s| {
                                    let mut settings = s.settings();
                                    if let Some(pairing) = settings
                                        .ai
                                        .pairings
                                        .iter_mut()
                                        .find(|pairing| pairing.id == p.id)
                                    {
                                        pairing.last_used = Some(crate::model::now_ms());
                                    }
                                    s.put(crate::model::Item::Settings(settings))
                                });
                                output.push_back(
                                    encoded(&ServerMessage::Welcome {
                                        notaya_version: env!("CARGO_PKG_VERSION").into(),
                                        level: p.level,
                                        locked: false,
                                    })
                                    .unwrap(),
                                );
                                *pairing.lock().unwrap() = Some(p);
                            } else {
                                output.push_back(encoded(&error(None, "not_paired")).unwrap());
                                closing = true;
                                closing_at = Some(Instant::now());
                            }
                        }
                        ClientMessage::Cancel { id: _ } => {
                            cancelled.store(true, Ordering::SeqCst);
                        }
                        ClientMessage::Call {
                            id,
                            action,
                            params,
                            preview,
                        } => {
                            if !greeted || busy.swap(true, Ordering::SeqCst) {
                                break;
                            }
                            let Some(p) = pairing.lock().unwrap().clone() else {
                                output.push_back(encoded(&error(Some(id), "not_paired")).unwrap());
                                busy.store(false, Ordering::SeqCst);
                                continue;
                            };
                            cancelled.store(false, Ordering::SeqCst);
                            let stop = cancelled.clone();
                            let busy = busy.clone();
                            let tx = tx.clone();
                            let app = app.clone();
                            std::thread::spawn(move || {
                                let state = app.state::<crate::state::AppState>();
                                let result = dispatch::call(
                                    &state,
                                    &p,
                                    &action,
                                    params,
                                    preview.unwrap_or(false),
                                    || stop.load(Ordering::SeqCst),
                                    || {
                                        let _ = tx.try_send(ServerMessage::Event {
                                            kind: EventKind::ConfirmPending,
                                            id: Some(id),
                                            level: None,
                                        });
                                    },
                                );
                                let message = match result {
                                    Ok(value) => ServerMessage::Result {
                                        id,
                                        ok: true,
                                        value,
                                        rev: None,
                                    },
                                    Err(e) => error(Some(id), dispatch::error_code(&e)),
                                };
                                if !stop.load(Ordering::SeqCst) {
                                    let _ = tx.try_send(message);
                                }
                                busy.store(false, Ordering::SeqCst);
                            });
                        }
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        cancelled.store(true, Ordering::SeqCst);
        input.zeroize();
        for bytes in &mut output {
            bytes.zeroize();
        }
        if let Some(p) = pairing.lock().unwrap().as_ref() {
            let _ = state.with_actor(&dispatch::actor(p), |s| {
                state.ai.flush_reads(s, Some(&p.id));
                s.coordination.release(&dispatch::actor(p), None);
                Ok(())
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rate_has_a_bounded_initial_burst() {
        let mut rate = Rate::new();
        for _ in 0..200 {
            assert!(rate.take());
        }
        assert!(!rate.take());
    }
}
