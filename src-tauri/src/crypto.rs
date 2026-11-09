//! Cryptographic building blocks.
//!
//! - Argon2id stretches the master password (memory-hard, so it resists GPU cracking).
//! - HKDF-SHA256 mixes in the optional keyfile and derives the key-encryption key.
//! - XChaCha20-Poly1305 encrypts and authenticates every record. Its 192-bit
//!   nonce is safe to pick at random for every write.
//!
//! All key material is wrapped in `Zeroizing`, so it is wiped from memory when dropped.

use std::path::Path;

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};
use hkdf::Hkdf;
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;
use zeroize::Zeroizing;

use crate::error::{Error, Result};

pub const KEY_LEN: usize = 32;
pub const NONCE_LEN: usize = 24;
pub const TAG_LEN: usize = 16;

pub type Key = Zeroizing<[u8; KEY_LEN]>;

/// Fills a fixed-size array from the operating system's secure random generator.
pub fn random_bytes<const N: usize>() -> [u8; N] {
    let mut buf = [0u8; N];
    getrandom::fill(&mut buf).expect("the OS random number generator failed");
    buf
}

pub fn random_key() -> Key {
    Zeroizing::new(random_bytes::<KEY_LEN>())
}

/// Argon2id cost settings, stored in the vault header so a vault keeps opening
/// if the defaults change later.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KdfParams {
    pub m_kib: u32,
    pub t: u32,
    pub p: u32,
}

impl KdfParams {
    /// 256 MiB of memory, 3 passes, 4 lanes: about a second on a modern PC.
    pub const DEFAULT: KdfParams = KdfParams { m_kib: 256 * 1024, t: 3, p: 4 };

    /// Bounds check for parameters read from a file, so a crafted header
    /// can't make the app allocate unlimited memory.
    pub fn validate(&self) -> Result<()> {
        let ok = (8..=4 * 1024 * 1024).contains(&self.m_kib)
            && (1..=64).contains(&self.t)
            && (1..=64).contains(&self.p);
        if ok { Ok(()) } else { Err(Error::Corrupt) }
    }
}

/// Derives the key-encryption key (KEK) from the password and optional keyfile digest.
pub fn derive_kek(
    password: &str,
    keyfile: Option<&[u8; 32]>,
    salt: &[u8; 32],
    params: KdfParams,
    label: &[u8],
) -> Result<Key> {
    params.validate()?;
    // Normalize so the same password typed on different keyboards/OSes matches.
    let normalized: Zeroizing<String> = Zeroizing::new(password.nfc().collect());

    let argon_params = Params::new(params.m_kib, params.t, params.p, Some(KEY_LEN))
        .map_err(|e| Error::Other(format!("Argon2 parameters: {e}")))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);
    let mut stretched = Zeroizing::new([0u8; KEY_LEN]);
    argon
        .hash_password_into(normalized.as_bytes(), salt, stretched.as_mut())
        .map_err(|e| Error::Other(format!("Argon2: {e}")))?;

    // The input is always 32 or 64 bytes, so the concatenation is unambiguous.
    let mut ikm = Zeroizing::new(Vec::with_capacity(64));
    ikm.extend_from_slice(stretched.as_ref());
    if let Some(k) = keyfile {
        ikm.extend_from_slice(k);
    }
    let hk = Hkdf::<Sha256>::new(Some(salt), &ikm);
    let mut kek = Zeroizing::new([0u8; KEY_LEN]);
    // The label belongs to the vault format (`Format::kek_label`): the Notaya format and the older Cipherleaf one differ.
    hk.expand(label, kek.as_mut())
        .map_err(|_| Error::Other("HKDF expand failed".into()))?;
    Ok(kek)
}

/// Encrypts `plaintext`. Output layout: `nonce (24) || ciphertext || tag (16)`.
/// `aad` is authenticated but not encrypted: it binds the ciphertext to its location.
pub fn seal(key: &[u8; KEY_LEN], aad: &[u8], plaintext: &[u8]) -> Vec<u8> {
    let cipher = XChaCha20Poly1305::new(&(*key).into());
    let nonce_bytes = random_bytes::<NONCE_LEN>();
    let nonce = XNonce::from(nonce_bytes);
    let ct = cipher
        .encrypt(&nonce, Payload { msg: plaintext, aad })
        .expect("XChaCha20-Poly1305 encryption cannot fail for in-memory buffers");
    let mut out = Vec::with_capacity(NONCE_LEN + ct.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);
    out
}

/// Decrypts and verifies data produced by [`seal`]. Any modification of the
/// ciphertext, nonce, tag or `aad` makes this fail.
pub fn open(key: &[u8; KEY_LEN], aad: &[u8], data: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
    if data.len() < NONCE_LEN + TAG_LEN {
        return Err(Error::Corrupt);
    }
    let (nonce_bytes, ct) = data.split_at(NONCE_LEN);
    let mut nonce_arr = [0u8; NONCE_LEN];
    nonce_arr.copy_from_slice(nonce_bytes);
    let cipher = XChaCha20Poly1305::new(&(*key).into());
    cipher
        .decrypt(&XNonce::from(nonce_arr), Payload { msg: ct, aad })
        .map(Zeroizing::new)
        .map_err(|_| Error::Corrupt)
}

/// Pads a payload up to a size bucket so ciphertext lengths reveal less about
/// the content. Layout: `len (u32 LE) || data || zero padding`.
pub fn pad(data: &[u8]) -> Zeroizing<Vec<u8>> {
    let total = 4 + data.len();
    let bucket = if total <= 4096 {
        256
    } else if total <= 64 * 1024 {
        4096
    } else {
        64 * 1024
    };
    let padded_len = total.div_ceil(bucket) * bucket;
    let mut out = Zeroizing::new(Vec::with_capacity(padded_len));
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(data);
    out.resize(padded_len, 0);
    out
}

pub fn unpad(data: &[u8]) -> Result<&[u8]> {
    if data.len() < 4 {
        return Err(Error::Corrupt);
    }
    let len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    data.get(4..4 + len).ok_or(Error::Corrupt)
}

pub fn sha256(data: &[u8]) -> [u8; 32] {
    let digest = Sha256::digest(data);
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

const MAX_KEYFILE_BYTES: u64 = 64 * 1024 * 1024;

/// A keyfile can be any file. Only its SHA-256 digest is used.
pub fn keyfile_digest(path: &Path) -> Result<Zeroizing<[u8; 32]>> {
    use std::io::Read;
    // Locality is checked before any access to the supplied path. Read from
    // that same guarded handle, so a later junction swap cannot redirect it.
    let file = crate::keyfile_path::open(path)?;
    let meta = file.metadata().map_err(|_| Error::invalid("The keyfile could not be read."))?;
    if meta.len() == 0 || meta.len() > MAX_KEYFILE_BYTES {
        return Err(Error::invalid("A keyfile must be between 1 byte and 64 MB."));
    }
    let mut bytes = Zeroizing::new(Vec::new());
    file.take(MAX_KEYFILE_BYTES + 1).read_to_end(&mut bytes)
        .map_err(|_| Error::invalid("The keyfile could not be read."))?;
    if bytes.is_empty() || bytes.len() as u64 > MAX_KEYFILE_BYTES {
        return Err(Error::invalid("A keyfile must be between 1 byte and 64 MB."));
    }
    Ok(Zeroizing::new(sha256(&bytes)))
}

/// Writes a new keyfile with 512 bits of randomness.
pub fn generate_keyfile(path: &Path) -> Result<()> {
    let bytes = Zeroizing::new(random_bytes::<64>());
    std::fs::write(path, bytes.as_ref())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAST: KdfParams = KdfParams { m_kib: 64, t: 1, p: 1 };

    #[test]
    fn seal_open_round_trip() {
        let key = random_key();
        let sealed = seal(&key, b"aad", b"hello vault");
        assert_eq!(&open(&key, b"aad", &sealed).unwrap()[..], b"hello vault");
    }

    #[test]
    fn nonces_are_unique() {
        let key = random_key();
        let a = seal(&key, b"", b"same");
        let b = seal(&key, b"", b"same");
        assert_ne!(a, b, "identical plaintexts must produce different ciphertexts");
    }

    #[test]
    fn tampering_is_detected() {
        let key = random_key();
        let mut sealed = seal(&key, b"aad", b"secret");
        let last = sealed.len() - 1;
        sealed[last] ^= 1;
        assert!(open(&key, b"aad", &sealed).is_err());
        sealed[last] ^= 1;
        sealed[NONCE_LEN] ^= 1;
        assert!(open(&key, b"aad", &sealed).is_err());
    }

    #[test]
    fn wrong_aad_or_key_fails() {
        let key = random_key();
        let sealed = seal(&key, b"row-1", b"secret");
        assert!(open(&key, b"row-2", &sealed).is_err());
        assert!(open(&random_key(), b"row-1", &sealed).is_err());
    }

    const LABEL: &[u8] = b"test/key-encryption-key";

    #[test]
    fn kek_depends_on_the_label_too() {
        let salt = random_bytes::<32>();
        let a = derive_kek("pw", None, &salt, FAST, b"notaya/v2/key-encryption-key").unwrap();
        let b = derive_kek("pw", None, &salt, FAST, b"cipherleaf/v1/key-encryption-key").unwrap();
        assert_ne!(*a, *b);
    }

    #[test]
    fn kek_depends_on_password_keyfile_and_salt() {
        let salt = random_bytes::<32>();
        let kf = random_bytes::<32>();
        let a = derive_kek("correct horse", None, &salt, FAST, LABEL).unwrap();
        let b = derive_kek("correct horse", None, &salt, FAST, LABEL).unwrap();
        let c = derive_kek("correct horsf", None, &salt, FAST, LABEL).unwrap();
        let d = derive_kek("correct horse", Some(&kf), &salt, FAST, LABEL).unwrap();
        let e = derive_kek("correct horse", None, &random_bytes::<32>(), FAST, LABEL).unwrap();
        assert_eq!(*a, *b);
        assert_ne!(*a, *c);
        assert_ne!(*a, *d);
        assert_ne!(*a, *e);
    }

    #[test]
    fn unicode_passwords_are_normalized() {
        let salt = random_bytes::<32>();
        // "é" as one code point vs. "e" + combining accent.
        let a = derive_kek("caf\u{e9}", None, &salt, FAST, LABEL).unwrap();
        let b = derive_kek("cafe\u{301}", None, &salt, FAST, LABEL).unwrap();
        assert_eq!(*a, *b);
    }

    #[test]
    fn padding_round_trip_and_buckets() {
        for len in [0usize, 1, 100, 252, 253, 5000, 70_000] {
            let data = vec![7u8; len];
            let padded = pad(&data);
            assert_eq!(unpad(&padded).unwrap(), &data[..]);
            assert!(padded.len() % 256 == 0);
        }
        assert_eq!(pad(&[1, 2, 3]).len(), pad(&[0; 200]).len());
    }

    #[test]
    fn crafted_kdf_params_are_rejected() {
        assert!(KdfParams { m_kib: u32::MAX, t: 3, p: 4 }.validate().is_err());
        assert!(KdfParams { m_kib: 1024, t: 0, p: 4 }.validate().is_err());
        assert!(KdfParams::DEFAULT.validate().is_ok());
    }
}
