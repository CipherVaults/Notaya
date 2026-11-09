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

const HANDSHAKE_LIMIT: usize = 8;
const AUTHENTICATED_LIMIT: usize = 4;
const IO_DEADLINE: Duration = Duration::from_secs(5);

/// Separate budgets: waiting for a person never occupies an authenticated slot.
#[derive(Default)]
struct Slots {
    handshake: Arc<AtomicUsize>,
    authenticated: Arc<AtomicUsize>,
    pairing: Arc<AtomicUsize>,
}
struct Slot(Arc<AtomicUsize>);
impl Slot {
    fn take(counter: &Arc<AtomicUsize>, limit: usize) -> Option<Self> {
        counter.try_update(Ordering::SeqCst, Ordering::SeqCst, |n| (n < limit).then_some(n + 1))
            .ok().map(|_| Self(counter.clone()))
    }
}
impl Drop for Slot {
    fn drop(&mut self) { self.0.fetch_sub(1, Ordering::SeqCst); }
}

/// Reserve delivery capacity before committing a newly approved credential.
/// Holding a pairing approval itself does not occupy authenticated capacity.
fn with_authenticated_slot<T>(counter: &Arc<AtomicUsize>, persist: impl FnOnce() -> crate::error::Result<T>)
    -> crate::error::Result<(T, Slot)>
{
    let slot = Slot::take(counter, AUTHENTICATED_LIMIT)
        .ok_or_else(|| crate::error::Error::invalid("rate_limited"))?;
    Ok((persist()?, slot))
}

#[derive(Default)]
struct Deadlines {
    frame: Option<Instant>,
    output: Option<Instant>,
    output_started: Option<Instant>,
}
impl Deadlines {
    fn expired(&self, now: Instant) -> bool {
        [self.frame, self.output].into_iter().flatten().any(|at| now.duration_since(at) >= IO_DEADLINE)
            || self.output_started.is_some_and(|at| now.duration_since(at) >= Duration::from_secs(30))
    }
}

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
                CloseHandle, GetLastError, LocalFree, ERROR_PIPE_CONNECTED, ERROR_PIPE_LISTENING, ERROR_NO_DATA, HANDLE, HLOCAL,
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
                    ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PeekNamedPipe, PIPE_NOWAIT,
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
                (HANDSHAKE_LIMIT + AUTHENTICATED_LIMIT + 2) as u32,
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

    fn poll_connected(pipe: &Handle) -> bool {
        unsafe {
            match ConnectNamedPipe(pipe.0, None) {
                Ok(()) => true,
                Err(_) => match GetLastError() {
                    ERROR_PIPE_CONNECTED => true,
                    ERROR_PIPE_LISTENING => false,
                    // A client may close its end before ConnectNamedPipe accepts
                    // it. Reset this instance, retaining first-instance ownership.
                    ERROR_NO_DATA => { let _ = DisconnectNamedPipe(pipe.0); false },
                    _ => { let _ = DisconnectNamedPipe(pipe.0); false },
                },
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
        let slots = Arc::new(Slots::default());
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
            // There is always a listener while enabled; stale/disconnected clients
            // cannot strand it. Existing authenticated workers keep their budget.
            if slots.handshake.load(Ordering::SeqCst) < HANDSHAKE_LIMIT {
                if let Some(p) = pipe.as_ref() {
                    let connected = poll_connected(p);
                    if connected {
                        // Keep the old instance alive until its replacement exists: no squatting gap.
                        let Ok(next) = listener(&name, &sid, false) else {
                            // Retain this owned instance even when replacement
                            // creation fails; disconnect and retry without a gap.
                            unsafe { let _ = DisconnectNamedPipe(p.0); }
                            std::thread::sleep(Duration::from_millis(25));
                            continue;
                        };
                        let Some(handshake) = Slot::take(&slots.handshake, HANDSHAKE_LIMIT) else { continue; };
                        let accepted = pipe.take().unwrap();
                        pipe = Some(next);
                        active.fetch_add(1, Ordering::SeqCst);
                        let count = active.clone();
                        let app = app.clone();
                        let slots = slots.clone();
                        // HANDLE is moved by its numeric value; one owner closes it on the worker.
                        let raw = accepted.0 .0 as usize;
                        std::mem::forget(accepted);
                        std::thread::spawn(move || {
                            let handle = Handle(HANDLE(raw as *mut _));
                            // Count cleanup also runs if a worker panics.
                            let _active = Slot(count);
                            connection(&app, handle, epoch, &slots, handshake);
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
    fn connection(app: &AppHandle, handle: Handle, epoch: u64, slots: &Slots, handshake: Slot) {
        let state = app.state::<crate::state::AppState>();
        let mut input = Vec::new();
        let mut output: VecDeque<Vec<u8>> = VecDeque::new();
        let mut written = 0;
        let (tx, rx) = mpsc::sync_channel::<ServerMessage>(4);
        let cancelled = Arc::new(AtomicBool::new(false));
        let busy = Arc::new(AtomicBool::new(false));
        let pairing = Arc::new(Mutex::new(None::<super::super::pairing::Pairing>));
        let approved_slot = Arc::new(Mutex::new(None::<Slot>));
        let mut client = None::<(String, Instant)>;
        let mut greeted = false;
        let mut rate = Rate::new();
        let mut closing = false;
        let started = Instant::now();
        let mut closing_at = None::<Instant>;
        let mut handshake = Some(handshake);
        let mut authenticated = None;
        let mut _pairing_slot = None;
        let mut deadlines = Deadlines::default();
        let mut active_id = None;
        loop {
            if deadlines.expired(Instant::now()) { break; }
            if pairing.lock().unwrap().is_some() && authenticated.is_none() {
                let Some(slot) = approved_slot.lock().unwrap().take()
                    .or_else(|| Slot::take(&slots.authenticated, AUTHENTICATED_LIMIT)) else { break; };
                authenticated = Some(slot);
                handshake = None;
                _pairing_slot = None;
            }
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
                // Only one pending pairing approval, distinct from handshakes.
                let Some(slot) = Slot::take(&slots.pairing, 1) else { break; };
                _pairing_slot = Some(slot);
                handshake = None;
                busy.store(true, Ordering::SeqCst);
                let pairing = pairing.clone();
                let busy = busy.clone();
                let tx = tx.clone();
                let stop = cancelled.clone();
                let app = app.clone();
                let authenticated_counter = slots.authenticated.clone();
                let approved_slot = approved_slot.clone();
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
                        let saved = with_authenticated_slot(&authenticated_counter, || state
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
                                super::super::pairing::insert_unique(&mut settings.ai, p.clone())?;
                                s.put(crate::model::Item::Settings(settings))
                            }));
                        if let Ok(((), slot)) = saved {
                            *approved_slot.lock().unwrap() = Some(slot);
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
            if (handshake.is_some() && started.elapsed() >= IO_DEADLINE)
                || (started.elapsed() > Duration::from_secs(300) && pairing.lock().unwrap().is_none()) {
                break;
            }
            if !closing {
                while let Ok(message) = rx.try_recv() {
                    let completed = match &message {
                        ServerMessage::Result { id, .. } | ServerMessage::Error { id: Some(id), .. } => active_id == Some(*id),
                        _ => false,
                    };
                    if completed {
                        active_id = None;
                        busy.store(false, Ordering::SeqCst);
                    }
                    if matches!(&message, ServerMessage::Error { id: None, .. })
                        && pairing.lock().unwrap().is_none() {
                        // A denied/failed pairing must not retain the sole
                        // approval slot until the five-minute timeout.
                        closing = true;
                        closing_at = Some(Instant::now());
                        _pairing_slot = None;
                    }
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
                deadlines.output.get_or_insert_with(Instant::now);
                deadlines.output_started.get_or_insert_with(Instant::now);
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
                if n > 0 { deadlines.output = Some(Instant::now()); }
                if written == bytes.len() {
                    if let Some(mut bytes) = output.pop_front() {
                        bytes.zeroize();
                    }
                    written = 0;
                    deadlines.output_started = None;
                }
            } else {
                deadlines.output = None;
                deadlines.output_started = None;
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
                if n > 0 { deadlines.frame.get_or_insert_with(Instant::now); }
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
                    deadlines.frame = (!input.is_empty()).then(Instant::now);
                    let Ok(message) = message else {
                        break;
                    };
                    if !rate.take() {
                        let id = match message { ClientMessage::Call { id, .. } => Some(id), _ => None };
                        output.push_back(encoded(&error(id, "rate_limited")).unwrap());
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
                        ClientMessage::Cancel { id } => {
                            if active_id == Some(id) && busy.load(Ordering::SeqCst) {
                                cancelled.store(true, Ordering::SeqCst);
                            }
                        }
                        ClientMessage::Call {
                            id,
                            action,
                            params,
                            preview,
                        } => {
                            if !greeted || busy.swap(true, Ordering::SeqCst) {
                                output.push_back(encoded(&error(Some(id), "rate_limited")).unwrap());
                                continue;
                            }
                            let Some(p) = pairing.lock().unwrap().clone() else {
                                output.push_back(encoded(&error(Some(id), "not_paired")).unwrap());
                                busy.store(false, Ordering::SeqCst);
                                continue;
                            };
                            cancelled.store(false, Ordering::SeqCst);
                            active_id = Some(id);
                            let stop = cancelled.clone();
                            let tx = tx.clone();
                            let app = app.clone();
                            std::thread::spawn(move || {
                                let state = app.state::<crate::state::AppState>();
                                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| dispatch::call(
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
                                ))).unwrap_or_else(|_| Err(crate::error::Error::invalid("invalid")));
                                let message = match result {
                                    Ok(value) => ServerMessage::Result {
                                        id,
                                        ok: true,
                                        value,
                                        rev: None,
                                    },
                                    Err(e) => error(Some(id), dispatch::error_code(&e)),
                                };
                                // Also acknowledge cancellation so the bridge can
                                // advance its serialized queue. A committed result
                                // is still delivered if cancellation arrived late.
                                let _ = tx.send(message);
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

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        #[ignore = "PC check: sandbox denies named-pipe client opens (Windows error 5)"]
        fn listener_recovers_after_client_disconnects_before_accept() {
            let name=format!(r"\\.\pipe\notaya-c5d-{}",crate::model::new_id());
            let sid=sid().unwrap();let pipe=listener(&name,&sid,true).unwrap();
            let open=||std::fs::OpenOptions::new().read(true).write(true).open(&name).unwrap();
            assert!(!poll_connected(&pipe));
            drop(open());
            assert!(!poll_connected(&pipe));
            let client=open();assert!(poll_connected(&pipe));
            assert!(listener(&name,&sid,true).is_err(),"first-instance ownership retained");
            drop(client);
            unsafe { let _=DisconnectNamedPipe(pipe.0); }
            let _next=open();assert!(poll_connected(&pipe));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pairing_at_capacity_never_persists_an_undeliverable_credential() {
        let slots=Slots::default();
        let active:Vec<_>=(0..AUTHENTICATED_LIMIT).map(|_|Slot::take(&slots.authenticated,AUTHENTICATED_LIMIT).unwrap()).collect();
        let mut persisted=false;
        let denied=with_authenticated_slot(&slots.authenticated, || { persisted=true; Ok(()) });
        assert!(denied.is_err());assert!(!persisted);
        drop(active);
        let ((), reservation)=with_authenticated_slot(&slots.authenticated, || { persisted=true; Ok(()) }).unwrap();
        assert!(persisted);assert_eq!(slots.authenticated.load(Ordering::SeqCst),1);
        drop(reservation);assert_eq!(slots.authenticated.load(Ordering::SeqCst),0);
        let failed:crate::error::Result<((),Slot)>=with_authenticated_slot(&slots.authenticated, || Err(crate::error::Error::NotFound));
        assert!(failed.is_err());assert_eq!(slots.authenticated.load(Ordering::SeqCst),0);
    }
    #[test]
    fn stalled_handshakes_and_pairing_do_not_take_authenticated_slots_and_release_on_drop() {
        let slots=Slots::default();
        let mut handshakes:Vec<_>=(0..HANDSHAKE_LIMIT).map(|_|Slot::take(&slots.handshake,HANDSHAKE_LIMIT).unwrap()).collect();
        assert!(Slot::take(&slots.handshake,HANDSHAKE_LIMIT).is_none());
        let pending=Slot::take(&slots.pairing,1).unwrap();
        assert!(Slot::take(&slots.pairing,1).is_none());
        let auth:Vec<_>=(0..AUTHENTICATED_LIMIT).map(|_|Slot::take(&slots.authenticated,AUTHENTICATED_LIMIT).unwrap()).collect();
        assert!(Slot::take(&slots.authenticated,AUTHENTICATED_LIMIT).is_none());
        handshakes.pop();
        assert!(Slot::take(&slots.handshake,HANDSHAKE_LIMIT).is_some());
        drop(handshakes);drop(pending);drop(auth);
        assert_eq!(slots.handshake.load(Ordering::SeqCst),0);
        assert_eq!(slots.authenticated.load(Ordering::SeqCst),0);
        assert_eq!(slots.pairing.load(Ordering::SeqCst),0);
    }
    #[test]
    fn incomplete_frames_and_stuck_or_dribbling_output_have_absolute_deadlines() {
        let at=Instant::now();
        let mut deadlines=Deadlines{frame:Some(at),..Default::default()};
        assert!(!deadlines.expired(at+Duration::from_secs(4)));
        assert!(deadlines.expired(at+IO_DEADLINE)); // Dribbling cannot extend a frame.
        deadlines.frame=None;deadlines.output=Some(at);deadlines.output_started=Some(at);
        assert!(deadlines.expired(at+IO_DEADLINE));
        deadlines.output=Some(at+Duration::from_secs(29));
        assert!(deadlines.expired(at+Duration::from_secs(30))); // Progress cannot hold a response forever.
        deadlines.output=None;deadlines.output_started=None;
        assert!(!deadlines.expired(at+Duration::from_secs(600))); // Idle authenticated sessions are fine.
    }
    #[test]
    fn rate_has_a_bounded_initial_burst() {
        let mut rate = Rate::new();
        for _ in 0..200 {
            assert!(rate.take());
        }
        assert!(!rate.take());
    }
}
