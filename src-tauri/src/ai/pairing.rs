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
