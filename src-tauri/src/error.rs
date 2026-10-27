//! One error type for the whole backend. Errors are sent to the UI as plain
//! strings, and they never contain secrets.

use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("The vault is locked.")]
    Locked,
    #[error("Wrong master password or keyfile for this Notaya vault.")]
    BadCredentials,
    #[error("The vault file is damaged or has been tampered with.")]
    Corrupt,
    #[error("A vault already exists in this folder.")]
    VaultExists,
    #[error("No vault was found in this folder.")]
    NoVault,
    #[error("That item no longer exists.")]
    NotFound,
    #[error("Too many attempts. Try again in {0} seconds.")]
    RateLimited(u64),
    #[error("{0}")]
    Invalid(String),
    #[error("Storage error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("File error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}

impl Error {
    pub fn invalid(msg: impl Into<String>) -> Self {
        Error::Invalid(msg.into())
    }
}

impl From<tauri::Error> for Error {
    fn from(e: tauri::Error) -> Self {
        Error::Other(e.to_string())
    }
}

impl Serialize for Error {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
