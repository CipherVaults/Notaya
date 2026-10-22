//! The vault header: the only unencrypted part of the vault.
//!
//! Layout (139 bytes):
//! ```text
//! magic "CLF1" (4) | version u16 (2) | argon2 m,t,p u32 (12) | salt (32)
//! | flags u8 (1) | vault id (16) | wrapped data key: nonce+key+tag (72)
//! ```
//! The first 67 bytes are authenticated as AAD when the data key is wrapped.
//! Changing anything (e.g. lowering the Argon2 cost) makes unlocking fail.

use zeroize::Zeroizing;

use crate::crypto::{self, KdfParams, Key, KEY_LEN, NONCE_LEN, TAG_LEN};
use crate::error::{Error, Result};

const MAGIC: &[u8; 4] = b"CLF1";
pub const FORMAT_VERSION: u16 = 1;
const FLAG_KEYFILE: u8 = 0b0000_0001;
const AUTH_LEN: usize = 4 + 2 + 12 + 32 + 1 + 16;
const WRAPPED_LEN: usize = NONCE_LEN + KEY_LEN + TAG_LEN;

#[derive(Clone, Debug)]
pub struct Header {
    pub kdf: KdfParams,
    pub salt: [u8; 32],
    pub keyfile_required: bool,
    pub vault_id: [u8; 16],
    wrapped_key: Vec<u8>,
}

impl Header {
    /// Creates a header for a brand-new vault and returns it with the fresh data key.
    pub fn create(password: &str, keyfile: Option<&[u8; 32]>, kdf: KdfParams) -> Result<(Header, Key)> {
        let dek = crypto::random_key();
        let header = Self::wrap(&dek, password, keyfile, kdf, crypto::random_bytes::<16>())?;
        Ok((header, dek))
    }

    /// Re-wraps an existing data key under new credentials (password change).
    /// Uses a fresh salt but keeps the vault id, so existing records stay valid.
    pub fn rewrap(
        &self,
        dek: &Key,
        password: &str,
        keyfile: Option<&[u8; 32]>,
        kdf: KdfParams,
    ) -> Result<Header> {
        Self::wrap(dek, password, keyfile, kdf, self.vault_id)
    }

    fn wrap(
        dek: &Key,
        password: &str,
        keyfile: Option<&[u8; 32]>,
        kdf: KdfParams,
        vault_id: [u8; 16],
    ) -> Result<Header> {
        let mut header = Header {
            kdf,
            salt: crypto::random_bytes::<32>(),
            keyfile_required: keyfile.is_some(),
            vault_id,
            wrapped_key: Vec::new(),
        };
        let kek = crypto::derive_kek(password, keyfile, &header.salt, kdf)?;
        header.wrapped_key = crypto::seal(&kek, &header.auth_bytes(), dek.as_ref());
        Ok(header)
    }

    /// Recovers the data key. Wrong credentials and a tampered header both give
    /// the same `BadCredentials` error, so an attacker learns nothing extra.
    pub fn unlock(&self, password: &str, keyfile: Option<&[u8; 32]>) -> Result<Key> {
        if self.keyfile_required != keyfile.is_some() {
            return Err(Error::BadCredentials);
        }
        let kek = crypto::derive_kek(password, keyfile, &self.salt, self.kdf)?;
        let raw = crypto::open(&kek, &self.auth_bytes(), &self.wrapped_key)
            .map_err(|_| Error::BadCredentials)?;
        if raw.len() != KEY_LEN {
            return Err(Error::Corrupt);
        }
        let mut dek = Zeroizing::new([0u8; KEY_LEN]);
        dek.copy_from_slice(&raw);
        Ok(dek)
    }

    fn auth_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(AUTH_LEN);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
        out.extend_from_slice(&self.kdf.m_kib.to_le_bytes());
        out.extend_from_slice(&self.kdf.t.to_le_bytes());
        out.extend_from_slice(&self.kdf.p.to_le_bytes());
        out.extend_from_slice(&self.salt);
        out.push(if self.keyfile_required { FLAG_KEYFILE } else { 0 });
        out.extend_from_slice(&self.vault_id);
        out
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = self.auth_bytes();
        out.extend_from_slice(&self.wrapped_key);
        out
    }

    pub fn from_bytes(b: &[u8]) -> Result<Header> {
        if b.len() != AUTH_LEN + WRAPPED_LEN || &b[0..4] != MAGIC {
            return Err(Error::Corrupt);
        }
        let u32_at = |i: usize| u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
        let version = u16::from_le_bytes([b[4], b[5]]);
        if version != FORMAT_VERSION {
            return Err(Error::invalid(
                "This vault was made by a newer version of Cipherleaf.",
            ));
        }
        let kdf = KdfParams { m_kib: u32_at(6), t: u32_at(10), p: u32_at(14) };
        kdf.validate()?;
        let flags = b[50];
        if flags & !FLAG_KEYFILE != 0 {
            return Err(Error::Corrupt);
        }
        let mut salt = [0u8; 32];
        salt.copy_from_slice(&b[18..50]);
        let mut vault_id = [0u8; 16];
        vault_id.copy_from_slice(&b[51..67]);
        Ok(Header {
            kdf,
            salt,
            keyfile_required: flags & FLAG_KEYFILE != 0,
            vault_id,
            wrapped_key: b[AUTH_LEN..].to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAST: KdfParams = KdfParams { m_kib: 64, t: 1, p: 1 };

    #[test]
    fn create_and_unlock() {
        let (h, dek) = Header::create("pw", None, FAST).unwrap();
        let parsed = Header::from_bytes(&h.to_bytes()).unwrap();
        assert_eq!(*parsed.unlock("pw", None).unwrap(), *dek);
        assert!(matches!(parsed.unlock("PW", None), Err(Error::BadCredentials)));
    }

    #[test]
    fn keyfile_is_required_when_set() {
        let kf = crypto::random_bytes::<32>();
        let (h, dek) = Header::create("pw", Some(&kf), FAST).unwrap();
        assert!(h.unlock("pw", None).is_err());
        assert!(h.unlock("pw", Some(&crypto::random_bytes::<32>())).is_err());
        assert_eq!(*h.unlock("pw", Some(&kf)).unwrap(), *dek);
    }

    #[test]
    fn every_header_byte_is_protected() {
        let (h, _) = Header::create("pw", None, FAST).unwrap();
        let bytes = h.to_bytes();
        for i in 0..bytes.len() {
            let mut t = bytes.clone();
            t[i] ^= 0x01;
            let ok = Header::from_bytes(&t).and_then(|p| p.unlock("pw", None)).is_ok();
            assert!(!ok, "flipping header byte {i} must prevent unlocking");
        }
    }

    #[test]
    fn rewrap_keeps_data_key_and_vault_id() {
        let (h, dek) = Header::create("old", None, FAST).unwrap();
        let h2 = h.rewrap(&dek, "new", None, FAST).unwrap();
        assert_eq!(h2.vault_id, h.vault_id);
        assert_ne!(h2.salt, h.salt);
        assert_eq!(*h2.unlock("new", None).unwrap(), *dek);
        assert!(h2.unlock("old", None).is_err());
    }
}
