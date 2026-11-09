use super::rules::Level;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::{Zeroize, Zeroizing};

#[derive(Clone, Serialize, Deserialize, Zeroize)]
#[serde(rename_all = "camelCase")]
pub struct Pairing {
    pub id: String,
    pub name: String,
    pub level: Level,
    pub created: i64,
    pub last_used: Option<i64>,
    pub token_hash: String,
}

#[derive(Clone, Serialize, Deserialize, Zeroize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub enabled: bool,
    pub pairings: Vec<Pairing>,
    pub hidden_notebooks: Vec<String>,
    pub big_change_confirm: bool,
    pub big_change_items: usize,
    pub paused: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: false,
            pairings: vec![],
            hidden_notebooks: vec![],
            big_change_confirm: true,
            big_change_items: 20,
            paused: false,
        }
    }
}

/// The approved protocol uses a persistent random 256-bit bearer credential,
/// revoked in the window. This is not a short pairing code or a rotating session
/// secret. Only approval waits expire; the saved credential survives restart.
pub fn create(name: String) -> (Pairing, Zeroizing<String>) {
    let token = Zeroizing::new(hex::encode(crate::crypto::random_bytes::<32>()));
    let pairing = Pairing {
        id: crate::model::new_id(),
        name,
        level: Level::Look,
        created: crate::model::now_ms(),
        last_used: None,
        token_hash: hash(&token),
    };
    (pairing, token)
}

/// Called while holding the session write lock, after the window's approval.
/// A duplicate never replaces or upgrades an existing credential.
pub fn insert_unique(settings: &mut Settings, pairing: Pairing) -> crate::error::Result<()> {
    if settings.pairings.len() >= 32 || settings.pairings.iter().any(|p| p.name == pairing.name) {
        return Err(crate::error::Error::invalid("pairing_refused"));
    }
    settings.pairings.push(pairing);
    Ok(())
}
/// Final pipe-side recheck, after the native-bound broker answer. Kept as a
/// Rust core so tests exercise the same persistence path without opening a pipe.
pub(crate) fn persist(state: &crate::state::AppState, pairing: Pairing, epoch: u64, cancelled: impl Fn() -> bool) -> crate::error::Result<()> {
    state.with_mut(|s| {
        let mut settings = s.settings();
        if !settings.ai.enabled || settings.ai.paused || cancelled()
            || crate::native_confirm::epoch(state) != epoch {
            return Err(crate::error::Error::invalid("off"));
        }
        if settings.ai.pairings.len() >= 32 { return Err(crate::error::Error::invalid("too_big")); }
        insert_unique(&mut settings.ai, pairing)?;
        s.put(crate::model::Item::Settings(settings))
    })
}
fn hash(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}
pub fn authenticate(settings: &Settings, id: &str, token: &str) -> Option<Pairing> {
    if token.len() != 64 || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let actual = sha2::digest::CtOutput::<Sha256>::new(Sha256::digest(token.as_bytes()));
    settings
        .pairings
        .iter()
        .find(|p| {
            let Ok(bytes) = hex::decode(&p.token_hash) else {
                return false;
            };
            let Ok(bytes) = <[u8; 32]>::try_from(bytes.as_slice()) else {
                return false;
            };
            // RustCrypto's constant-time output comparison; no home-grown primitive.
            p.id == id && actual == sha2::digest::CtOutput::<Sha256>::new(bytes.into())
        })
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn duplicate_client_pairing_never_replaces_an_approved_credential() {
        let mut settings = Settings::default();
        let (first, token) = create("Codex".into());
        insert_unique(&mut settings, first.clone()).unwrap();
        let (second, _) = create("Codex".into());
        assert!(insert_unique(&mut settings, second).is_err());
        assert_eq!(settings.pairings.len(), 1);
        assert!(authenticate(&settings,&first.id,&token).is_some());
        settings.pairings.clear(); // Only the person's revoke command removes it.
        let (replacement, _) = create("Codex".into());
        insert_unique(&mut settings, replacement).unwrap();
        assert!(authenticate(&settings,&first.id,&token).is_none());
    }
    #[test]
    fn token_is_returned_once_and_only_hash_is_persisted() {
        let (p, token) = create("Codex".into());
        assert_eq!(p.level, Level::Look);
        let mut s = Settings::default();
        s.pairings.push(p.clone());
        assert!(authenticate(&s, &p.id, &token).is_some());
        assert!(!serde_json::to_string(&s).unwrap().contains(token.as_str()));
        assert!(authenticate(&s, &p.id, &"0".repeat(64)).is_none());
        s.pairings.clear();
        assert!(authenticate(&s, &p.id, &token).is_none());
    }
}
