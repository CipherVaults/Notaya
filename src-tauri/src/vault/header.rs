//! The vault header: the only unencrypted part of the vault.
//!
//! Layout (139 bytes):
//! ```text
//! magic "CLF1" (4) | version u16 (2) | argon2 m,t,p u32 (12) | salt (32)
//! | flags u8 (1) | vault id (16) | wrapped data key: nonce+key+tag (72)
//! ```
//! The first 67 bytes are authenticated as AAD when the data key is wrapped.
//! Changing anything (e.g. lowering the Argon2 cost) makes unlocking fail.
//!
//! An "open" vault (security off, amendment A21) has the open flag set in `flags`.
//! Its data key is wrapped under a built-in, public password, so the file is still
//! in the encrypted, tamper-checked format but is NOT secret: anyone with the file
//! and a copy of Notaya can read it. Turning security on or off, and changing the
//! master password or keyfile, re-key the vault (a fresh data key and vault id, see
//! `Session::rekey`): a header is never re-wrapped around the old data key, because
//! an older header and its password would then still open data written later.

use zeroize::Zeroizing;

use crate::crypto::{self, KdfParams, Key, KEY_LEN, NONCE_LEN, TAG_LEN};
use crate::error::{Error, Result};

const MAGIC: &[u8; 4] = b"CLF1";
pub const FORMAT_VERSION: u16 = 1;
const FLAG_KEYFILE: u8 = 0b0000_0001;
const FLAG_OPEN: u8 = 0b0000_0010;

/// The built-in password of an open vault. It is public on purpose: an open vault
/// is not confidential (see the module comment). It keeps the old app name on purpose:
/// changing it would stop every open vault made so far from opening (A28).
pub const OPEN_PASSWORD: &str = "cipherleaf-open-vault-security-off";

/// Cheap key stretching for an open vault: there is no secret to guess, so
/// spending 256 MiB on every start would only slow down opening the notes.
pub const OPEN_KDF: KdfParams = KdfParams { m_kib: 8 * 1024, t: 1, p: 1 };
const AUTH_LEN: usize = 4 + 2 + 12 + 32 + 1 + 16;
const WRAPPED_LEN: usize = NONCE_LEN + KEY_LEN + TAG_LEN;

#[derive(Clone, Debug)]
pub struct Header {
    pub kdf: KdfParams,
    pub salt: [u8; 32],
    pub keyfile_required: bool,
    /// Security is off: the data key is wrapped under [`OPEN_PASSWORD`].
    pub open: bool,
    pub vault_id: [u8; 16],
    wrapped_key: Vec<u8>,
}

impl Header {
    /// Creates a header for a brand-new vault and returns it with the fresh data key.
    pub fn create(password: &str, keyfile: Option<&[u8; 32]>, kdf: KdfParams) -> Result<(Header, Key)> {
        let dek = crypto::random_key();
        let header = Self::wrap(&dek, password, keyfile, kdf, crypto::random_bytes::<16>(), false)?;
        Ok((header, dek))
    }

    /// Creates a header for a brand-new open vault (security off).
    pub fn create_open() -> Result<(Header, Key)> {
        let dek = crypto::random_key();
        let header = Self::wrap(&dek, OPEN_PASSWORD, None, OPEN_KDF, crypto::random_bytes::<16>(), true)?;
        Ok((header, dek))
    }

    /// Re-wraps an existing data key under the built-in key (security off). Only tests use it:
    /// the app never re-wraps a data key (see the module comment); it re-keys the vault.
    pub fn rewrap_open(&self, dek: &Key) -> Result<Header> {
        Self::wrap(dek, OPEN_PASSWORD, None, OPEN_KDF, self.vault_id, true)
    }

    /// A header for a re-keyed vault (a fresh data key and vault id), under a master password.
    /// Used when security is turned on and when the master password or keyfile is changed.
    pub fn protected_for(
        dek: &Key,
        vault_id: [u8; 16],
        password: &str,
        keyfile: Option<&[u8; 32]>,
        kdf: KdfParams,
    ) -> Result<Header> {
        Self::wrap(dek, password, keyfile, kdf, vault_id, false)
    }

    /// A header for a re-keyed vault (a fresh data key and vault id), under the built-in key.
    pub fn open_for(dek: &Key, vault_id: [u8; 16]) -> Result<Header> {
        Self::wrap(dek, OPEN_PASSWORD, None, OPEN_KDF, vault_id, true)
    }

    fn wrap(
        dek: &Key,
        password: &str,
        keyfile: Option<&[u8; 32]>,
        kdf: KdfParams,
        vault_id: [u8; 16],
        open: bool,
    ) -> Result<Header> {
        let mut header = Header {
            kdf,
            salt: crypto::random_bytes::<32>(),
            keyfile_required: keyfile.is_some(),
            open,
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

    /// Recovers the data key of an open vault. Fails for a vault that has a master password.
    pub fn unlock_open(&self) -> Result<Key> {
        if !self.open {
            return Err(Error::BadCredentials);
        }
        self.unlock(OPEN_PASSWORD, None)
    }

    fn auth_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(AUTH_LEN);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
        out.extend_from_slice(&self.kdf.m_kib.to_le_bytes());
        out.extend_from_slice(&self.kdf.t.to_le_bytes());
        out.extend_from_slice(&self.kdf.p.to_le_bytes());
        out.extend_from_slice(&self.salt);
        out.push(self.flags());
        out.extend_from_slice(&self.vault_id);
        out
    }

    fn flags(&self) -> u8 {
        (if self.keyfile_required { FLAG_KEYFILE } else { 0 }) | (if self.open { FLAG_OPEN } else { 0 })
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
                "This vault was made by a newer version of Notaya.",
            ));
        }
        let kdf = KdfParams { m_kib: u32_at(6), t: u32_at(10), p: u32_at(14) };
        kdf.validate()?;
        let flags = b[50];
        if flags & !(FLAG_KEYFILE | FLAG_OPEN) != 0 {
            return Err(Error::Corrupt);
        }
        // An open vault has no master password, so it cannot also need a keyfile.
        if flags & FLAG_OPEN != 0 && flags & FLAG_KEYFILE != 0 {
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
            open: flags & FLAG_OPEN != 0,
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
    fn a_header_for_a_new_password_wraps_the_given_fresh_key_and_id() {
        // Changing the password builds a header for a NEW data key and vault id (the old
        // header is never re-wrapped around the old key, see the module comment).
        let (h, old_dek) = Header::create("old", None, FAST).unwrap();
        let new_dek = crypto::random_key();
        let new_id = crypto::random_bytes::<16>();
        let h2 = Header::protected_for(&new_dek, new_id, "new", None, FAST).unwrap();
        assert_eq!(h2.vault_id, new_id);
        assert_ne!(h2.vault_id, h.vault_id);
        assert_ne!(h2.salt, h.salt);
        assert_eq!(*h2.unlock("new", None).unwrap(), *new_dek);
        assert_ne!(*h2.unlock("new", None).unwrap(), *old_dek);
        assert!(h2.unlock("old", None).is_err());
    }

    #[test]
    fn an_open_header_opens_without_credentials_and_is_marked() {
        let (h, dek) = Header::create_open().unwrap();
        let parsed = Header::from_bytes(&h.to_bytes()).unwrap();
        assert!(parsed.open && !parsed.keyfile_required);
        assert_eq!(*parsed.unlock_open().unwrap(), *dek);
        assert!(parsed.unlock("anything else", None).is_err());
    }

    #[test]
    fn a_protected_header_never_opens_as_open() {
        let (h, _) = Header::create("pw", None, FAST).unwrap();
        assert!(!h.open);
        assert!(matches!(h.unlock_open(), Err(Error::BadCredentials)));
        // Even the built-in password does not open it.
        assert!(h.unlock(OPEN_PASSWORD, None).is_err());
    }

    #[test]
    fn the_open_flag_cannot_be_flipped_either_way() {
        let (open, _) = Header::create_open().unwrap();
        let (prot, _) = Header::create("pw", None, FAST).unwrap();
        let flags_at = 50;
        let mut b = open.to_bytes();
        b[flags_at] &= !FLAG_OPEN;
        assert!(Header::from_bytes(&b).and_then(|p| p.unlock_open()).is_err());
        assert!(Header::from_bytes(&b).and_then(|p| p.unlock(OPEN_PASSWORD, None)).is_err());
        let mut b = prot.to_bytes();
        b[flags_at] |= FLAG_OPEN;
        assert!(Header::from_bytes(&b).and_then(|p| p.unlock_open()).is_err());
        assert!(Header::from_bytes(&b).and_then(|p| p.unlock("pw", None)).is_err());
    }

    #[test]
    fn every_open_header_byte_is_protected() {
        let (h, _) = Header::create_open().unwrap();
        let bytes = h.to_bytes();
        for i in 0..bytes.len() {
            let mut t = bytes.clone();
            t[i] ^= 0x01;
            assert!(Header::from_bytes(&t).and_then(|p| p.unlock_open()).is_err(), "byte {i}");
        }
    }

    #[test]
    fn wrapping_between_open_and_protected_keeps_the_key_and_id() {
        let (h, dek) = Header::create_open().unwrap();
        let p = Header::protected_for(&dek, h.vault_id, "pw", None, FAST).unwrap();
        assert!(!p.open && p.vault_id == h.vault_id);
        assert_eq!(*p.unlock("pw", None).unwrap(), *dek);
        let o = p.rewrap_open(&dek).unwrap();
        assert!(o.open && o.vault_id == h.vault_id && !o.keyfile_required);
        assert_eq!(*o.unlock_open().unwrap(), *dek);
        assert!(o.unlock("pw", None).is_err());
    }

    #[test]
    fn open_with_a_keyfile_flag_is_refused() {
        let (open, _) = Header::create_open().unwrap();
        let mut b = open.to_bytes();
        b[50] |= FLAG_KEYFILE;
        assert!(matches!(Header::from_bytes(&b), Err(Error::Corrupt)));
    }
}
