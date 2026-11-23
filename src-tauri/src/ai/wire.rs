use super::rules::Level;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    io::{self, Read, Write},
    path::Path,
};
use zeroize::Zeroizing;

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

pub fn write_frame<W: Write, M: Serialize>(
    writer: &mut W,
    message: &M,
    cap: usize,
) -> io::Result<()> {
    let bytes = Zeroizing::new(serde_json::to_vec(&Frame::new(message)).map_err(|_| invalid())?);
    if bytes.is_empty() || bytes.len() > cap {
        return Err(invalid());
    }
    writer.write_all(&(bytes.len() as u32).to_le_bytes())?;
    writer.write_all(&bytes)?;
    writer.flush()
}

pub fn read_frame<R: Read, M: DeserializeOwned>(reader: &mut R, cap: usize) -> io::Result<M> {
    let mut length = [0; 4];
    reader.read_exact(&mut length)?;
    let length = u32::from_le_bytes(length) as usize;
    if length == 0 || length > cap {
        return Err(invalid());
    }
    let mut bytes = Zeroizing::new(vec![0; length]);
    reader.read_exact(&mut bytes)?;
    decode(&bytes)
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bridge_fixtures_caps_and_never_messages() {
        let fixture=br#"{"v":1,"type":"call","id":7,"action":"page.get","params":{"id":"p","format":"blocks"},"preview":true}"#;
        let mut bytes = (fixture.len() as u32).to_le_bytes().to_vec();
        bytes.extend_from_slice(fixture);
        assert!(matches!(
            read_frame::<_, ClientMessage>(&mut bytes.as_slice(), MAX_REQUEST).unwrap(),
            ClientMessage::Call { id: 7, .. }
        ));
        for bad in [
            r#"{"v":1,"type":"ai_confirm_answer","id":7,"allow":true}"#,
            r#"{"v":1,"type":"confirm_answer","id":7,"allow":true}"#,
            r#"{"v":2,"type":"cancel","id":7}"#,
            r#"{"v":1,"type":"cancel","id":7,"allow":true}"#,
        ] {
            assert!(decode::<ClientMessage>(bad.as_bytes()).is_err());
        }
        for len in [0, MAX_REQUEST as u32 + 1] {
            assert!(
                read_frame::<_, ClientMessage>(&mut len.to_le_bytes().as_slice(), MAX_REQUEST)
                    .is_err()
            );
        }
        assert_ne!(
            pipe_name(Path::new(r"D:\test\data"), "S-1-5-21-1").unwrap(),
            pipe_name(Path::new(r"D:\real\data"), "S-1-5-21-1").unwrap()
        );
        assert_ne!(
            pipe_name(Path::new(r"D:\test\data"), "S-1-5-21-1").unwrap(),
            pipe_name(Path::new(r"D:\test\data"), "S-1-5-21-2").unwrap()
        );
    }
}
