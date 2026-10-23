//! An unlocked vault session. The data key and decrypted items live only here,
//! in Rust memory, and are wiped when the session is dropped (on lock).

pub mod header;
pub mod store;

use std::collections::HashMap;
use std::path::Path;

use zeroize::Zeroizing;

use crate::crypto::{self, KdfParams, Key};
use crate::error::{Error, Result};
use crate::model::{Item, Settings, SETTINGS_ID};
use header::{Header, FORMAT_VERSION};
use store::{RowId, Store, Table};

/// Largest single item accepted (a long page with lots of text).
const MAX_ITEM_BYTES: usize = 32 * 1024 * 1024;
/// Largest image attachment.
pub const MAX_ATTACHMENT_BYTES: usize = 25 * 1024 * 1024;

pub struct Session {
    dek: Key,
    vault_id: [u8; 16],
    store: Store,
    items: HashMap<String, Item>,
    /// True once anything changed since unlock; triggers a backup on lock.
    pub dirty: bool,
}

/// AAD binds each ciphertext to this vault, its table and its row id, so rows
/// can't be moved, swapped or copied between vaults without detection.
fn aad(vault_id: &[u8; 16], table: Table, id: &RowId) -> Vec<u8> {
    let mut out = Vec::with_capacity(48);
    out.extend_from_slice(b"cipherleaf");
    out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(vault_id);
    out.extend_from_slice(table.tag());
    out.extend_from_slice(id);
    out
}

pub fn parse_id(id: &str) -> Result<RowId> {
    let bytes = hex::decode(id).map_err(|_| Error::NotFound)?;
    bytes.try_into().map_err(|_| Error::NotFound)
}

impl Session {
    pub fn create(path: &Path, password: &str, keyfile: Option<&[u8; 32]>, kdf: KdfParams) -> Result<Session> {
        let (header, dek) = Header::create(password, keyfile, kdf)?;
        let store = Store::create(path, &header)?;
        let mut session = Session {
            dek,
            vault_id: header.vault_id,
            store,
            items: HashMap::new(),
            dirty: true,
        };
        session.put(Item::Settings(Settings::default()))?;
        Ok(session)
    }

    pub fn read_header(path: &Path) -> Result<Header> {
        Store::open(path)?.read_header()
    }

    pub fn unlock(path: &Path, password: &str, keyfile: Option<&[u8; 32]>) -> Result<Session> {
        let store = Store::open(path)?;
        let header = store.read_header()?;
        let dek = header.unlock(password, keyfile)?;
        let mut items = HashMap::new();
        for (id, data) in store.all(Table::Records)? {
            let plain = crypto::open(&dek, &aad(&header.vault_id, Table::Records, &id), &data)?;
            let item: Item = serde_json::from_slice(crypto::unpad(&plain)?).map_err(|_| Error::Corrupt)?;
            if item.id() != hex::encode(id) {
                return Err(Error::Corrupt);
            }
            items.insert(item.id().to_string(), item);
        }
        Ok(Session { dek, vault_id: header.vault_id, store, items, dirty: false })
    }

    fn seal_item(&self, item: &Item) -> Result<(RowId, Vec<u8>)> {
        let id = parse_id(item.id())?;
        let json = Zeroizing::new(serde_json::to_vec(item).map_err(|e| Error::Other(e.to_string()))?);
        if json.len() > MAX_ITEM_BYTES {
            return Err(Error::invalid("This item is too large to save."));
        }
        let padded = crypto::pad(&json);
        Ok((id, crypto::seal(&self.dek, &aad(&self.vault_id, Table::Records, &id), &padded)))
    }

    /// Encrypts and stores an item (insert or update).
    pub fn put(&mut self, item: Item) -> Result<()> {
        let (id, sealed) = self.seal_item(&item)?;
        self.store.put(Table::Records, &id, &sealed)?;
        self.items.insert(item.id().to_string(), item);
        self.dirty = true;
        Ok(())
    }

    /// Stores several items atomically: either all are saved or none.
    pub fn put_many(&mut self, items: Vec<Item>) -> Result<()> {
        let sealed = items.iter().map(|i| self.seal_item(i)).collect::<Result<Vec<_>>>()?;
        self.store.transaction(|s| {
            for (id, data) in &sealed {
                s.put(Table::Records, id, data)?;
            }
            Ok(())
        })?;
        for item in items {
            self.items.insert(item.id().to_string(), item);
        }
        self.dirty = true;
        Ok(())
    }

    pub fn remove_many(&mut self, ids: &[String]) -> Result<()> {
        let rows = ids.iter().map(|id| parse_id(id)).collect::<Result<Vec<_>>>()?;
        self.store.transaction(|s| {
            for row in &rows {
                s.delete(Table::Records, row)?;
            }
            Ok(())
        })?;
        for id in ids {
            self.items.remove(id);
        }
        self.dirty = true;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Item> {
        self.items.get(id)
    }

    pub fn items(&self) -> impl Iterator<Item = &Item> {
        self.items.values()
    }

    pub fn settings(&self) -> Settings {
        match self.items.get(SETTINGS_ID) {
            Some(Item::Settings(s)) => s.clone().sanitized(),
            _ => Settings::default(),
        }
    }

    pub fn put_attachment(&mut self, mime: &str, bytes: &[u8]) -> Result<String> {
        if bytes.len() > MAX_ATTACHMENT_BYTES {
            return Err(Error::invalid("Images must be smaller than 25 MB."));
        }
        let id_hex = crate::model::new_id();
        let id = parse_id(&id_hex)?;
        let mut payload = Zeroizing::new(Vec::with_capacity(1 + mime.len() + bytes.len()));
        payload.push(mime.len() as u8);
        payload.extend_from_slice(mime.as_bytes());
        payload.extend_from_slice(bytes);
        let padded = crypto::pad(&payload);
        let sealed = crypto::seal(&self.dek, &aad(&self.vault_id, Table::Attachments, &id), &padded);
        self.store.put(Table::Attachments, &id, &sealed)?;
        self.dirty = true;
        Ok(id_hex)
    }

    pub fn get_attachment(&self, id_hex: &str) -> Result<(String, Vec<u8>)> {
        let id = parse_id(id_hex)?;
        let data = self.store.get(Table::Attachments, &id)?.ok_or(Error::NotFound)?;
        let plain = crypto::open(&self.dek, &aad(&self.vault_id, Table::Attachments, &id), &data)?;
        let body = crypto::unpad(&plain)?;
        let mime_len = *body.first().ok_or(Error::Corrupt)? as usize;
        let mime = body.get(1..1 + mime_len).ok_or(Error::Corrupt)?;
        let mime = String::from_utf8(mime.to_vec()).map_err(|_| Error::Corrupt)?;
        Ok((mime, body[1 + mime_len..].to_vec()))
    }

    /// Decrypts and authenticates every attachment (one at a time, then drops
    /// it). `unlock` already checks every record; attachments are otherwise
    /// only checked when they are shown. A restore uses this to check the
    /// whole file before it replaces the live vault.
    pub fn verify_attachments(&self) -> Result<()> {
        for id in self.store.ids(Table::Attachments)? {
            self.get_attachment(&hex::encode(id))?;
        }
        Ok(())
    }

    /// Deletes attachments no page or sticky note refers to any more.
    pub fn collect_garbage(&mut self) -> Result<usize> {
        let mut referenced = String::new();
        for item in self.items.values() {
            match item {
                Item::Page(p) => {
                    referenced.push_str(&p.content);
                    referenced.push_str(&p.cover);
                }
                Item::Sticky(s) => referenced.push_str(&s.content),
                _ => {}
            }
        }
        let mut removed = 0;
        for id in self.store.ids(Table::Attachments)? {
            if !referenced.contains(&hex::encode(id)) {
                self.store.delete(Table::Attachments, &id)?;
                removed += 1;
            }
        }
        if removed > 0 {
            self.dirty = true;
        }
        Ok(removed)
    }

    /// Changes the master password and/or keyfile. Only the header changes: the
    /// data key is re-wrapped, so this is instant and can't lose data.
    pub fn change_credentials(
        &mut self,
        current_password: &str,
        current_keyfile: Option<&[u8; 32]>,
        new_password: &str,
        new_keyfile: Option<&[u8; 32]>,
        kdf: KdfParams,
    ) -> Result<()> {
        let header = self.store.read_header()?;
        let check = header.unlock(current_password, current_keyfile)?;
        if *check != *self.dek {
            return Err(Error::BadCredentials);
        }
        let new_header = header.rewrap(&self.dek, new_password, new_keyfile, kdf)?;
        self.store.write_header(&new_header)?;
        self.dirty = true;
        Ok(())
    }

    pub fn backup_to(&self, dest: &Path) -> Result<()> {
        self.store.backup_to(dest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{new_id, now_ms, Page, Sticky};
    use rusqlite::{params, Connection};

    const FAST: KdfParams = KdfParams { m_kib: 64, t: 1, p: 1 };

    fn temp_vault() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("cipherleaf-test-{}", new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("vault.clf")
    }

    fn page(text: &str) -> Item {
        Item::Page(Page {
            id: new_id(),
            section_id: new_id(),
            title: "Diary".into(),
            content: format!("{{\"type\":\"doc\",\"text\":\"{text}\"}}"),
            text: text.into(),
            order: 0,
            favorite: false,
            icon: String::new(),
            cover: String::new(),
            parent: String::new(),
            trashed_at: None,
            created: now_ms(),
            updated: now_ms(),
        })
    }

    fn sticky(text: &str) -> Item {
        Item::Sticky(Sticky {
            id: new_id(),
            content: format!("{{\"type\":\"doc\",\"text\":\"{text}\"}}"),
            text: text.into(),
            color: "yellow".into(),
            x: 100.0,
            y: 100.0,
            w: 320.0,
            h: 320.0,
            on_top: true,
            open: false,
            created: now_ms(),
            updated: now_ms(),
        })
    }

    #[test]
    fn create_reopen_and_read() {
        let path = temp_vault();
        let p = page("hello");
        let pid = p.id().to_string();
        {
            let mut s = Session::create(&path, "pw", None, FAST).unwrap();
            s.put(p).unwrap();
        }
        let s = Session::unlock(&path, "pw", None).unwrap();
        assert!(matches!(s.get(&pid), Some(Item::Page(pg)) if pg.text == "hello"));
        assert_eq!(s.settings().idle_lock_minutes, 5);
        assert!(matches!(Session::unlock(&path, "nope", None), Err(Error::BadCredentials)));
    }

    #[test]
    fn no_plaintext_on_disk() {
        let path = temp_vault();
        {
            let mut s = Session::create(&path, "pw", None, FAST).unwrap();
            s.put(page("TopSecretDiaryPhrase")).unwrap();
            s.put(sticky("Bank PIN Hunter2-Correct-Horse")).unwrap();
            s.put_attachment("image/png", b"PNGSECRETPIXELS").unwrap();
        }
        let raw = std::fs::read(&path).unwrap();
        for needle in [&b"TopSecretDiaryPhrase"[..], b"Hunter2-Correct-Horse", b"Bank PIN", b"Diary", b"PNGSECRETPIXELS", b"yellow"] {
            assert!(
                !raw.windows(needle.len()).any(|w| w == needle),
                "found plaintext {:?} in the vault file",
                String::from_utf8_lossy(needle)
            );
        }
    }

    #[test]
    fn tampered_row_is_rejected() {
        let path = temp_vault();
        {
            let mut s = Session::create(&path, "pw", None, FAST).unwrap();
            s.put(page("x")).unwrap();
        }
        let conn = Connection::open(&path).unwrap();
        let (id, mut data): (Vec<u8>, Vec<u8>) = conn
            .query_row("SELECT id, data FROM records LIMIT 1", [], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap();
        data[40] ^= 0xff;
        conn.execute("UPDATE records SET data = ?1 WHERE id = ?2", params![data, id]).unwrap();
        drop(conn);
        assert!(matches!(Session::unlock(&path, "pw", None), Err(Error::Corrupt)));
    }

    #[test]
    fn swapped_rows_are_rejected() {
        let path = temp_vault();
        {
            let mut s = Session::create(&path, "pw", None, FAST).unwrap();
            s.put(page("a")).unwrap();
        }
        let conn = Connection::open(&path).unwrap();
        let rows: Vec<(Vec<u8>, Vec<u8>)> = {
            let mut st = conn.prepare("SELECT id, data FROM records").unwrap();
            st.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap().map(|r| r.unwrap()).collect()
        };
        assert_eq!(rows.len(), 2); // settings + page
        conn.execute("UPDATE records SET data = ?1 WHERE id = ?2", params![rows[1].1, rows[0].0]).unwrap();
        conn.execute("UPDATE records SET data = ?1 WHERE id = ?2", params![rows[0].1, rows[1].0]).unwrap();
        drop(conn);
        assert!(matches!(Session::unlock(&path, "pw", None), Err(Error::Corrupt)));
    }

    #[test]
    fn change_password_keeps_data() {
        let path = temp_vault();
        let kf = crypto::random_bytes::<32>();
        let p = page("keep me");
        let pid = p.id().to_string();
        {
            let mut s = Session::create(&path, "old", None, FAST).unwrap();
            s.put(p).unwrap();
            assert!(s.change_credentials("wrong", None, "new", Some(&kf), FAST).is_err());
            s.change_credentials("old", None, "new", Some(&kf), FAST).unwrap();
        }
        assert!(Session::unlock(&path, "old", None).is_err());
        assert!(Session::unlock(&path, "new", None).is_err());
        let s = Session::unlock(&path, "new", Some(&kf)).unwrap();
        assert!(s.get(&pid).is_some());
    }

    #[test]
    fn attachments_round_trip_and_gc() {
        let path = temp_vault();
        let mut s = Session::create(&path, "pw", None, FAST).unwrap();
        let keep = s.put_attachment("image/png", b"keep").unwrap();
        let drop_me = s.put_attachment("image/png", b"drop").unwrap();
        let mut pg = page("img");
        if let Item::Page(p) = &mut pg {
            p.content = format!("{{\"src\":\"http://clatt.localhost/{keep}\"}}");
        }
        s.put(pg).unwrap();
        assert_eq!(s.get_attachment(&keep).unwrap(), ("image/png".to_string(), b"keep".to_vec()));
        assert_eq!(s.collect_garbage().unwrap(), 1);
        assert!(s.get_attachment(&drop_me).is_err());
        assert!(s.get_attachment(&keep).is_ok());
    }

    #[test]
    fn a_page_cover_keeps_its_image_alive() {
        let path = temp_vault();
        let mut s = Session::create(&path, "pw", None, FAST).unwrap();
        let cover = s.put_attachment("image/png", b"cover").unwrap();
        let mut pg = page("with cover");
        if let Item::Page(p) = &mut pg {
            p.cover = format!("http://clatt.localhost/{cover}");
        }
        s.put(pg).unwrap();
        assert_eq!(s.collect_garbage().unwrap(), 0);
        assert!(s.get_attachment(&cover).is_ok());
    }

    #[test]
    fn backup_is_a_valid_encrypted_vault() {
        let path = temp_vault();
        let backup = path.with_file_name("backup.clf");
        let p = page("backed up");
        let pid = p.id().to_string();
        {
            let mut s = Session::create(&path, "pw", None, FAST).unwrap();
            s.put(p).unwrap();
            s.backup_to(&backup).unwrap();
        }
        let s = Session::unlock(&backup, "pw", None).unwrap();
        assert!(s.get(&pid).is_some());
    }

    #[test]
    fn pages_saved_before_icons_and_covers_still_load() {
        let old = r#"{"kind":"page","id":"p","sectionId":"s","title":"t","content":"","text":"","order":0,"favorite":false,"trashedAt":null,"created":1,"updated":1}"#;
        let item: Item = serde_json::from_str(old).expect("old page record should load");
        match item {
            Item::Page(p) => assert!(p.icon.is_empty() && p.cover.is_empty()),
            _ => panic!("expected a page"),
        }
    }
}
