//! Pictures are encrypted as they are read. The temporary file holds ciphertext
//! outside the vault until the final transaction and is removed when closed.

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};

use super::{RowId, Session, Store, Table, MAX_ATTACHMENT_BYTES};
use crate::{crypto, error::{Error, Result}};

pub(crate) struct ImportPictures {
    session: [u8; 16],
    vault: [u8; 16],
    file: Option<File>,
    entries: Vec<(RowId, usize)>,
}

impl ImportPictures {
    pub(crate) fn new(s: &Session) -> Self {
        Self { session: s.native_session_id(), vault: s.vault_id, file: None, entries: Vec::new() }
    }

    pub(crate) fn matches(&self, s: &Session) -> bool {
        self.session == s.native_session_id() && self.vault == s.vault_id
    }

    pub(crate) fn stage(&mut self, s: &Session, mime: &str, bytes: &[u8]) -> Result<String> {
        if !self.matches(s) {
            return Err(Error::Locked);
        }
        if bytes.len() > MAX_ATTACHMENT_BYTES {
            return Err(Error::invalid("Images must be smaller than 900 MB."));
        }
        if self.file.is_none() {
            self.file = Some(open_temporary()?);
        }
        let id = crypto::random_bytes();
        let sealed = s.seal_blob(&id, mime, bytes);
        self.file.as_mut().unwrap().write_all(&sealed)?;
        self.entries.push((id, sealed.len()));
        Ok(hex::encode(id))
    }

    pub(super) fn write_to(&mut self, store: &Store) -> Result<()> {
        if let Some(file) = &mut self.file {
            file.rewind()?;
            let mut sealed = Vec::new();
            for (id, len) in &self.entries {
                sealed.resize(*len, 0);
                file.read_exact(&mut sealed)?;
                store.put(Table::Attachments, id, &sealed)?;
            }
        }
        Ok(())
    }
}

fn open_temporary() -> Result<File> {
    let path = std::env::temp_dir().join(format!("notaya-import-pictures-{}", crate::model::new_id()));
    let mut options = OpenOptions::new();
    options.read(true).write(true).create_new(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use windows::Win32::Storage::FileSystem::FILE_FLAG_DELETE_ON_CLOSE;
        options.share_mode(0).custom_flags(FILE_FLAG_DELETE_ON_CLOSE.0);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(&path)?;
    #[cfg(unix)]
    std::fs::remove_file(&path)?;
    Ok(file)
}
