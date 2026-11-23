//! Section 7 wire contract. Lane C5 can include this module (and `tools::Level`) or copy
//! these serde types verbatim; it must use the same JSON fixtures and pipe-name function.
//! Every frame has `v: 1` and a snake_case `type` tag. IDs on the pipe are bridge-owned u64s.
use super::tools::Level;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

pub const WIRE_VERSION: u32 = 1;
pub const MAX_REQUEST: usize = 8 * 1024 * 1024;
pub const MAX_RESPONSE: usize = 32 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
pub struct Frame<M> {
    pub v: u32,
    #[serde(flatten)]
    pub message: M,
}

impl<M> Frame<M> {
    pub fn new(message: M) -> Self {
        Self {
            v: WIRE_VERSION,
            message,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClientMessage {
    Hello {
        bridge_version: String,
        client: String,
    },
    Auth {
        pairing_id: String,
        token: String,
    },
    Call {
        id: u64,
        action: String,
        params: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        preview: Option<bool>,
    },
    Cancel {
        id: u64,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ServerMessage {
    Welcome {
        notaya_version: String,
        level: Level,
        locked: bool,
    },
    PairingResult {
        pairing_id: String,
        token: String,
    },
    Result {
        id: u64,
        ok: bool,
        value: Value,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        rev: Option<u64>,
    },
    Error {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<u64>,
        code: String,
        message: String,
    },
    Event {
        kind: EventKind,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        level: Option<Level>,
    },
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    LevelChanged,
    Locked,
    Unlocked,
    Stopped,
    ConfirmPending,
}

fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid")
}

fn wipe(bytes: &mut [u8]) {
    for b in bytes.iter_mut() {
        // Volatile byte write prevents dead-store elimination without adding a crate (G2).
        unsafe { std::ptr::write_volatile(b, 0) };
    }
    std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
}

pub fn write_frame<W: Write, M: Serialize>(
    writer: &mut W,
    message: &M,
    cap: usize,
) -> io::Result<()> {
    let mut bytes = serde_json::to_vec(&Frame::new(message)).map_err(|_| invalid())?;
    let res = (|| {
        if bytes.is_empty() || bytes.len() > cap {
            return Err(invalid());
        }
        writer.write_all(&(bytes.len() as u32).to_le_bytes())?;
        writer.write_all(&bytes)?;
        writer.flush()
    })();
    wipe(&mut bytes);
    res
}

pub fn read_frame<R: Read, M: DeserializeOwned>(reader: &mut R, cap: usize) -> io::Result<M> {
    let mut length = [0; 4];
    reader.read_exact(&mut length)?;
    let length = u32::from_le_bytes(length) as usize;
    if length == 0 || length > cap {
        return Err(invalid());
    }
    let mut bytes = vec![0; length];
    let res = reader.read_exact(&mut bytes).and_then(|()| decode(&bytes));
    wipe(&mut bytes);
    res
}

fn decode<M: DeserializeOwned>(bytes: &[u8]) -> io::Result<M> {
    let frame: Frame<M> = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    if frame.v != WIRE_VERSION {
        return Err(invalid());
    }
    Ok(frame.message)
}

/// Hash UTF-8(full canonical Windows data path without the extended-path prefix) || UTF-8(SID).
/// ASCII case is folded on Windows; no delimiter, trailing slash or Unicode normalization.
/// C5 must use this exact algorithm, rather than hashing the home folder or a relative path.
pub fn pipe_name(data: &Path, sid: &str) -> io::Result<String> {
    let path = data.to_str().ok_or_else(invalid)?;
    #[cfg(windows)]
    let path = path
        .strip_prefix(r"\\?\")
        .unwrap_or(path)
        .replace('/', "\\")
        .to_ascii_lowercase();
    let mut digest = Sha256::new();
    digest.update(path.as_bytes());
    digest.update(sid.as_bytes());
    let hex: String = digest.finalize()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    Ok(format!(r"\\.\pipe\notaya-ai-{hex}"))
}

/// Same release selection as portable::Paths, without migrations, probes or vault reads.
pub fn data_dir(exe_dir: &Path, home: Option<&Path>, local: Option<&Path>) -> io::Result<PathBuf> {
    let root = match home {
        Some(home) if !home.as_os_str().is_empty() => home.to_path_buf(),
        _ if exe_dir.join("portable.txt").is_file() || exe_dir.join("data").is_dir() => {
            exe_dir.to_path_buf()
        }
        _ => local
            .filter(|p| p.is_absolute())
            .map(|p| p.join("Notaya Notes"))
            .unwrap_or_else(|| exe_dir.to_path_buf()),
    };
    // Never create a missing home and silently fall back to the person's real notes.
    fs::canonicalize(root.join("data"))
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pairing {
    pub pairing_id: String,
    pub token: String,
    pub client: String,
}

impl Drop for Pairing {
    fn drop(&mut self) {
        // SAFETY: String's backing buffer is overwritten with valid zero bytes (ASCII NUL).
        unsafe { wipe(self.token.as_bytes_mut()) };
    }
}

impl Pairing {
    pub fn into_auth_message(mut self) -> ClientMessage {
        ClientMessage::Auth {
            pairing_id: std::mem::take(&mut self.pairing_id),
            token: std::mem::take(&mut self.token),
        }
    }
}

impl ClientMessage {
    pub fn wipe_secret(&mut self) {
        if let ClientMessage::Auth { token, .. } = self {
            unsafe { wipe(token.as_bytes_mut()) };
        }
    }
}

fn valid_pairing(pairing: &Pairing) -> bool {
    !pairing.pairing_id.is_empty()
        && pairing.pairing_id.len() <= 128
        && pairing
            .pairing_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        && pairing.token.len() == 64
        && pairing.token.bytes().all(|b| b.is_ascii_hexdigit())
        && !pairing.client.is_empty()
        && pairing.client.len() <= 256
}

fn plain_metadata(path: &Path) -> io::Result<fs::Metadata> {
    let meta = fs::symlink_metadata(path)?;
    if meta.file_type().is_symlink() {
        return Err(invalid());
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 {
            return Err(invalid());
        } // FILE_ATTRIBUTE_REPARSE_POINT
    }
    Ok(meta)
}

pub fn load_pairing(data: &Path, client: &str) -> io::Result<Option<Pairing>> {
    let dir = data.join("ai");
    match plain_metadata(&dir) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
        Ok(meta) if !meta.is_dir() => return Err(invalid()),
        _ => {}
    }
    let mut found = None;
    for (index, entry) in fs::read_dir(dir)?.enumerate() {
        if index >= 1024 {
            return Err(invalid());
        }
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if !name.starts_with("pairing-") || !name.ends_with(".json") {
            continue;
        }
        let meta = plain_metadata(&entry.path())?;
        if !meta.is_file() || meta.len() > 4096 {
            return Err(invalid());
        }
        let mut bytes = Vec::new();
        let read_res = fs::File::open(entry.path())?
            .take(4097)
            .read_to_end(&mut bytes);
        if read_res.is_err() {
            wipe(&mut bytes);
            read_res?;
        }
        let parsed: io::Result<Pairing> =
            serde_json::from_slice(&bytes).map_err(|_| invalid());
        wipe(&mut bytes);
        let pairing = parsed?;
        if !valid_pairing(&pairing) || name != format!("pairing-{}.json", pairing.pairing_id) {
            return Err(invalid());
        }
        if pairing.client == client {
            if found.is_some() {
                return Err(invalid());
            } // do not guess between credentials
            found = Some(pairing);
        }
    }
    Ok(found)
}

/// New credentials never overwrite an earlier pairing (or an arbitrary path from the server).
pub fn save_pairing(data: &Path, pairing: &Pairing) -> io::Result<()> {
    if !valid_pairing(pairing) {
        return Err(invalid());
    }
    let dir = data.join("ai");
    match fs::create_dir(&dir) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(e),
    }
    if !plain_metadata(&dir)?.is_dir() {
        return Err(invalid());
    }
    let path = dir.join(format!("pairing-{}.json", pairing.pairing_id));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    let bytes = serde_json::to_vec(pairing).map_err(|_| invalid())?;
    file.write_all(&bytes)?;
    file.sync_all()
}

pub trait Transport {
    fn send(&mut self, message: &ClientMessage) -> io::Result<()>;
    fn poll(&mut self) -> io::Result<Vec<ServerMessage>>;
}

#[cfg(windows)]
pub mod native {
    use super::*;
    use std::{
        os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
        time::{Duration, Instant},
    };
    use windows::{
        Win32::{
            Foundation::{CloseHandle, HANDLE, HLOCAL, LocalFree},
            Security::{
                Authorization::ConvertSidToStringSidW, GetTokenInformation, TOKEN_QUERY,
                TOKEN_USER, TokenUser,
            },
            System::{
                Pipes::{GetNamedPipeServerProcessId, PeekNamedPipe},
                Threading::{
                    GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_NAME_WIN32,
                    PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
                },
            },
        },
        core::PWSTR,
    };

    struct OwnedHandle(HANDLE);
    impl Drop for OwnedHandle {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }

    pub fn current_sid() -> io::Result<String> {
        unsafe {
            let mut token = HANDLE::default();
            OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)
                .map_err(|_| invalid())?;
            let token = OwnedHandle(token);
            let mut size = 0;
            let _ = GetTokenInformation(token.0, TokenUser, None, 0, &mut size);
            if size == 0 || size > 65536 {
                return Err(invalid());
            }
            // TOKEN_USER contains pointers, so use aligned storage rather than casting a Vec<u8>.
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
            let mut sid = PWSTR::null();
            ConvertSidToStringSidW(user.User.Sid, &mut sid).map_err(|_| invalid())?;
            let result = sid.to_string().map_err(|_| invalid());
            let _ = LocalFree(Some(HLOCAL(sid.0.cast())));
            result
        }
    }

    pub struct Connection {
        file: fs::File,
        buffer: Vec<u8>,
        started: Option<Instant>,
        // Keep the verified process open while this pipe is in use, so its PID cannot be recycled.
        _server: OwnedHandle,
    }

    impl Connection {
        pub fn connect(name: &str, beside: &Path) -> io::Result<Self> {
            Self::connect_expected(name, &beside.join("Notaya.exe"))
        }

        fn connect_expected(name: &str, expected: &Path) -> io::Result<Self> {
            // SECURITY_SQOS_PRESENT | SECURITY_IDENTIFICATION: a server cannot impersonate us.
            let file = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .custom_flags(0x0010_0000 | 0x0001_0000)
                .open(name)?;
            let handle = HANDLE(file.as_raw_handle());
            unsafe {
                let mut pid = 0;
                GetNamedPipeServerProcessId(handle, &mut pid).map_err(|_| invalid())?;
                let server = OwnedHandle(
                    OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
                        .map_err(|_| invalid())?,
                );
                let mut path = vec![0u16; 32768];
                let mut length = path.len() as u32;
                QueryFullProcessImageNameW(
                    server.0,
                    PROCESS_NAME_WIN32,
                    PWSTR(path.as_mut_ptr()),
                    &mut length,
                )
                .map_err(|_| invalid())?;
                use std::os::windows::ffi::OsStringExt;
                let actual = PathBuf::from(std::ffi::OsString::from_wide(&path[..length as usize]));
                // Compare file identity, not a case-sensitive display string or a filename alone.
                if file_identity(&actual)? != file_identity(expected)? {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "not_running",
                    ));
                }
                Ok(Self {
                    file,
                    buffer: Vec::new(),
                    started: None,
                    _server: server,
                })
            }
        }
    }

    fn file_identity(path: &Path) -> io::Result<(u32, u32, u32)> {
        use windows::Win32::Storage::FileSystem::{
            BY_HANDLE_FILE_INFORMATION, GetFileInformationByHandle,
        };
        // Share delete so checking an image does not obstruct normal updates.
        let file = fs::OpenOptions::new().read(true).share_mode(7).open(path)?;
        let mut info = BY_HANDLE_FILE_INFORMATION::default();
        unsafe {
            GetFileInformationByHandle(HANDLE(file.as_raw_handle()), &mut info)
                .map_err(|_| invalid())?;
        }
        Ok((
            info.dwVolumeSerialNumber,
            info.nFileIndexHigh,
            info.nFileIndexLow,
        ))
    }

    impl Transport for Connection {
        fn send(&mut self, message: &ClientMessage) -> io::Result<()> {
            write_frame(&mut self.file, message, MAX_REQUEST)
        }
        fn poll(&mut self) -> io::Result<Vec<ServerMessage>> {
            let mut available = 0;
            unsafe {
                PeekNamedPipe(
                    HANDLE(self.file.as_raw_handle()),
                    None,
                    0,
                    None,
                    Some(&mut available),
                    None,
                )
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "not_running"))?;
            }
            if available > 0 {
                let mut chunk = vec![0; (available as usize).min(65536)];
                let read = self.file.read(&mut chunk)?;
                if read == 0 {
                    return Err(io::ErrorKind::UnexpectedEof.into());
                }
                self.buffer.extend_from_slice(&chunk[..read]);
                self.started.get_or_insert_with(Instant::now);
            }
            let mut messages = Vec::new();
            let mut consumed = 0;
            while self.buffer.len() - consumed >= 4 {
                let length =
                    u32::from_le_bytes(self.buffer[consumed..consumed + 4].try_into().unwrap())
                        as usize;
                if length == 0 || length > MAX_RESPONSE {
                    return Err(invalid());
                }
                if self.buffer.len() - consumed - 4 < length {
                    break;
                }
                messages.push(read_frame(
                    &mut std::io::Cursor::new(&self.buffer[consumed..consumed + 4 + length]),
                    MAX_RESPONSE,
                )?);
                consumed += 4 + length;
            }
            if consumed > 0 {
                self.buffer.drain(..consumed);
                self.started = if self.buffer.is_empty() {
                    None
                } else {
                    Some(Instant::now())
                };
            }
            if self
                .started
                .is_some_and(|t| t.elapsed() > Duration::from_secs(30))
            {
                return Err(invalid());
            }
            Ok(messages)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::{os::windows::io::FromRawHandle, sync::mpsc};
        use windows::{
            Win32::{
                Foundation::ERROR_PIPE_CONNECTED,
                Storage::FileSystem::{FILE_FLAG_FIRST_PIPE_INSTANCE, PIPE_ACCESS_DUPLEX},
                System::Pipes::{
                    ConnectNamedPipe, CreateNamedPipeW, PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE,
                    PIPE_WAIT,
                },
            },
            core::PCWSTR,
        };

        fn fake_server() -> (String, fs::File) {
            let name = format!(
                r"\\.\pipe\notaya-bridge-test-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            );
            let wide: Vec<_> = name.encode_utf16().chain(Some(0)).collect();
            let handle = unsafe {
                CreateNamedPipeW(
                    PCWSTR(wide.as_ptr()),
                    PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE,
                    PIPE_TYPE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
                    1,
                    65536,
                    65536,
                    1000,
                    None,
                )
            };
            assert!(!handle.is_invalid());
            (name, unsafe { fs::File::from_raw_handle(handle.0) })
        }

        fn accept(file: &fs::File) {
            let result = unsafe { ConnectNamedPipe(HANDLE(file.as_raw_handle()), None) };
            if let Err(error) = result {
                assert_eq!(error.code(), ERROR_PIPE_CONNECTED.to_hresult());
            }
        }

        #[test]
        #[ignore = "PC check: the lane sandbox denies named-pipe client opens (Windows error 5)"]
        fn windows_fake_pipe_frames_both_directions_and_verifies_server_before_auth() {
            let (name, mut server) = fake_server();
            let (finish, wait) = mpsc::channel();
            let thread = std::thread::spawn(move || {
                accept(&server);
                assert!(matches!(
                    read_frame(&mut server, MAX_REQUEST).unwrap(),
                    ClientMessage::Hello { .. }
                ));
                assert!(
                    matches!(read_frame(&mut server, MAX_REQUEST).unwrap(), ClientMessage::Auth { pairing_id, .. } if pairing_id == "fake")
                );
                let mut reply = Vec::new();
                write_frame(
                    &mut reply,
                    &ServerMessage::Welcome {
                        notaya_version: "fake".into(),
                        level: Level::Look,
                        locked: false,
                    },
                    MAX_RESPONSE,
                )
                .unwrap();
                for chunk in reply.chunks(3) {
                    server.write_all(chunk).unwrap();
                }
                wait.recv_timeout(Duration::from_secs(5)).unwrap();
            });
            // Production requires the adjacent Notaya.exe; this fake uses the test binary's
            // actual image, with the same PID and file-identity verification, not a bypass.
            let mut connection =
                Connection::connect_expected(&name, &std::env::current_exe().unwrap()).unwrap();
            connection
                .send(&ClientMessage::Hello {
                    bridge_version: "test".into(),
                    client: "fake".into(),
                })
                .unwrap();
            connection
                .send(&ClientMessage::Auth {
                    pairing_id: "fake".into(),
                    token: "a".repeat(64),
                })
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(3);
            loop {
                let messages = connection.poll().unwrap();
                if !messages.is_empty() {
                    assert!(matches!(
                        messages[0],
                        ServerMessage::Welcome {
                            level: Level::Look,
                            ..
                        }
                    ));
                    break;
                }
                assert!(Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(2));
            }
            finish.send(()).unwrap();
            thread.join().unwrap();
        }

        #[test]
        #[ignore = "PC check: the lane sandbox denies named-pipe client opens (Windows error 5)"]
        fn another_process_image_is_refused_without_sending_a_byte() {
            let test_root = PathBuf::from(std::env::var_os("NOTAYA_HOME").unwrap()).join(format!("wrong-image-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
            fs::create_dir_all(&test_root).unwrap();
            fs::write(test_root.join("Notaya.exe"), b"inert image fixture, never executed").unwrap();
            let (name, mut server) = fake_server();
            let thread = std::thread::spawn(move || {
                accept(&server);
                let mut byte = [0];
                assert!(matches!(server.read(&mut byte), Ok(0) | Err(_)));
            });
            assert!(Connection::connect(&name, &test_root).is_err());
            thread.join().unwrap();
        }

        #[test]
        fn current_sid_and_canonical_windows_path_produce_a_stable_isolated_name() {
            let sid = current_sid().unwrap();
            assert!(sid.starts_with("S-1-"));
            assert_eq!(
                pipe_name(Path::new(r"\\?\D:\Tests\data"), &sid).unwrap(),
                pipe_name(Path::new(r"d:\tests\DATA"), &sid).unwrap()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Cursor;
    #[test]
    fn both_ends_share_the_exact_json_contract() {
        let from_bridge = [
            json!({"v":1,"type":"hello","bridge_version":"1.6.0","client":"Codex"}),
            json!({"v":1,"type":"auth","pairing_id":"p1","token":"abc"}),
            json!({"v":1,"type":"call","id":7,"action":"page.get","params":{"format":"blocks"},"preview":false}),
            json!({"v":1,"type":"cancel","id":7}),
        ];
        for value in from_bridge {
            let frame: Frame<ClientMessage> = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(serde_json::to_value(frame).unwrap(), value);
        }
        let from_notaya = [
            json!({"v":1,"type":"welcome","notaya_version":"1.6.0","level":"look","locked":false}),
            json!({"v":1,"type":"pairing_result","pairing_id":"p1","token":"abc"}),
            json!({"v":1,"type":"result","id":7,"ok":true,"value":{"text":"\u{200f}ملاحظات  "},"rev":5}),
            json!({"v":1,"type":"error","id":7,"code":"locked","message":"Notaya is locked"}),
            json!({"v":1,"type":"event","kind":"level_changed"}),
            json!({"v":1,"type":"event","kind":"confirm_pending","id":7}),
        ];
        for value in from_notaya {
            let frame: Frame<ServerMessage> = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(serde_json::to_value(frame).unwrap(), value);
        }
    }

    #[test]
    fn frames_handle_fragmentation_coalescing_and_hostile_lengths() {
        struct Fragmented(Cursor<Vec<u8>>);
        impl Read for Fragmented {
            fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
                let len = out.len().min(1);
                self.0.read(&mut out[..len])
            }
        }
        let mut bytes = Vec::new();
        for id in [1, 2] {
            write_frame(&mut bytes, &ClientMessage::Cancel { id }, MAX_REQUEST).unwrap();
        }
        let mut fragmented = Fragmented(Cursor::new(bytes.clone()));
        for expected in [1, 2] {
            assert!(
                matches!(read_frame(&mut fragmented, MAX_REQUEST).unwrap(), ClientMessage::Cancel { id } if id == expected)
            );
        }
        for header in [0u32, MAX_RESPONSE as u32 + 1, u32::MAX] {
            assert!(
                read_frame::<_, ServerMessage>(
                    &mut Cursor::new(header.to_le_bytes()),
                    MAX_RESPONSE
                )
                .is_err()
            );
        }
        bytes.pop();
        let mut truncated = Cursor::new(bytes);
        read_frame::<_, ClientMessage>(&mut truncated, MAX_REQUEST).unwrap();
        assert!(read_frame::<_, ClientMessage>(&mut truncated, MAX_REQUEST).is_err());
        for bad in [
            br#"{"v":2,"type":"cancel","id":1}"#.as_slice(),
            br#"{"v":1,"type":"ai_confirm_answer","id":1}"#,
            &[0xff],
        ] {
            assert!(decode::<ClientMessage>(bad).is_err());
        }
        assert!(write_frame(&mut Vec::new(), &ClientMessage::Cancel { id: 1 }, 1).is_err());
    }

    fn temp_data() -> PathBuf {
        let root = std::env::var_os("NOTAYA_HOME").expect("tests require a throwaway NOTAYA_HOME");
        let dir = PathBuf::from(root).join(format!(
            "bridge-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(dir.join("data")).unwrap();
        dir
    }

    #[test]
    fn home_selection_and_pipe_names_isolate_test_copies() {
        let portable = temp_data();
        let other = temp_data();
        let data = data_dir(&portable, None, Some(&other)).unwrap();
        assert_eq!(data, fs::canonicalize(portable.join("data")).unwrap());
        assert_ne!(
            pipe_name(&data, "S-1-5-21-1").unwrap(),
            pipe_name(&data, "S-1-5-21-2").unwrap()
        );
        assert_ne!(
            pipe_name(&data, "S-1-5-21-1").unwrap(),
            pipe_name(&other.join("data"), "S-1-5-21-1").unwrap()
        );
        assert!(data_dir(&portable, Some(&other.join("missing")), Some(&portable)).is_err());
        let bare = other.join("program");
        fs::create_dir_all(other.join("Notaya Notes/data")).unwrap();
        assert_eq!(
            data_dir(&bare, None, Some(&other)).unwrap(),
            fs::canonicalize(other.join("Notaya Notes/data")).unwrap()
        );
    }

    #[test]
    fn pairing_files_are_bounded_client_specific_and_never_overwritten() {
        let root = temp_data();
        let data = root.join("data");
        let mut pairing = Pairing {
            pairing_id: "p-1".into(),
            token: "a".repeat(64),
            client: "Codex".into(),
        };
        assert!(load_pairing(&data, "Codex").unwrap().is_none());
        save_pairing(&data, &pairing).unwrap();
        assert!(save_pairing(&data, &pairing).is_err());
        assert_eq!(
            load_pairing(&data, "Codex").unwrap().unwrap().pairing_id,
            "p-1"
        );
        assert!(load_pairing(&data, "Claude Code").unwrap().is_none());
        pairing.pairing_id = "../../escape".into();
        assert!(save_pairing(&data, &pairing).is_err());
        pairing.pairing_id = "p2".into();
        pairing.token = "invalid".into();
        assert!(save_pairing(&data, &pairing).is_err());
        assert_eq!(fs::read_dir(data.join("ai")).unwrap().count(), 1);
    }

    #[test]
    fn wire_frame_roundtrip_and_wipe_buffer() {
        let msg = ClientMessage::Auth {
            pairing_id: "p-1".into(),
            token: "b".repeat(64),
        };
        let mut buf = Vec::new();
        write_frame(&mut buf, &msg, MAX_REQUEST).unwrap();
        let parsed: ClientMessage = read_frame(&mut buf.as_slice(), MAX_REQUEST).unwrap();
        assert!(matches!(parsed, ClientMessage::Auth { .. }));
        wipe(&mut buf);
        assert!(buf.iter().all(|b| *b == 0));
    }
}
