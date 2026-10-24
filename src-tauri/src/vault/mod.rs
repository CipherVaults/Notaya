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
    /// Security is off (amendment A21): the data key is wrapped under the built-in key.
    open: bool,
    /// True once anything changed since unlock; triggers a backup on lock.
    pub dirty: bool,
    /// True once anything changed since the last hourly backup of an open vault. It is kept
    /// apart from `dirty` so an hourly copy never stands in for the backup made at lock or quit.
    pub hourly_due: bool,
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
    /// Records that something changed, for the backups made at lock or quit and the hourly ones.
    fn changed(&mut self) {
        self.dirty = true;
        self.hourly_due = true;
    }

    pub fn create(path: &Path, password: &str, keyfile: Option<&[u8; 32]>, kdf: KdfParams) -> Result<Session> {
        let (header, dek) = Header::create(password, keyfile, kdf)?;
        let store = Store::create(path, &header)?;
        let mut session = Session {
            dek,
            vault_id: header.vault_id,
            store,
            items: HashMap::new(),
            open: false,
            dirty: true,
            hourly_due: true,
        };
        session.put(Item::Settings(Settings::default()))?;
        Ok(session)
    }

    /// Creates a new vault with security off: no master password, opened by the built-in key.
    pub fn create_open(path: &Path) -> Result<Session> {
        let (header, dek) = Header::create_open()?;
        let store = Store::create(path, &header)?;
        let mut session = Session {
            dek,
            vault_id: header.vault_id,
            store,
            items: HashMap::new(),
            open: true,
            dirty: true,
            hourly_due: true,
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
        Self::load(store, header, dek)
    }

    /// Opens a vault that has security off. Fails with `BadCredentials` if the vault has a master password.
    pub fn unlock_open(path: &Path) -> Result<Session> {
        let store = Store::open(path)?;
        let header = store.read_header()?;
        let dek = header.unlock_open()?;
        Self::load(store, header, dek)
    }

    fn load(store: Store, header: Header, dek: Key) -> Result<Session> {
        let mut items = HashMap::new();
        for (id, data) in store.all(Table::Records)? {
            let plain = crypto::open(&dek, &aad(&header.vault_id, Table::Records, &id), &data)?;
            let item: Item = serde_json::from_slice(crypto::unpad(&plain)?).map_err(|_| Error::Corrupt)?;
            if item.id() != hex::encode(id) {
                return Err(Error::Corrupt);
            }
            items.insert(item.id().to_string(), item);
        }
        Ok(Session { dek, vault_id: header.vault_id, store, items, open: header.open, dirty: false, hourly_due: false })
    }

    /// Opens a vault file whichever way it is protected: a vault with security off needs
    /// no credentials (they are ignored), any other needs the password and keyfile.
    pub fn unlock_any(path: &Path, password: &str, keyfile: Option<&[u8; 32]>) -> Result<Session> {
        if Self::read_header(path)?.open {
            Self::unlock_open(path)
        } else {
            Self::unlock(path, password, keyfile)
        }
    }

    /// Whether the vault file needs a keyfile besides the master password.
    pub fn keyfile_required(&self) -> Result<bool> {
        Ok(self.store.read_header()?.keyfile_required)
    }

    /// True when security is off for this vault.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Whether the app's windows are hidden from screenshots: only while security is on
    /// and the person switched the setting on (it is off by default).
    pub fn protects_capture(&self) -> bool {
        !self.open && self.settings().screen_capture_protection
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
        self.changed();
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
        self.changed();
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
        self.changed();
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
        self.changed();
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
            self.changed();
        }
        Ok(removed)
    }

    /// Changes the master password and/or keyfile.
    ///
    /// The vault is re-keyed, not just re-wrapped (like turning security on or off): a fresh
    /// data key and vault id, with every record and attachment re-encrypted in one transaction
    /// (all or nothing). Otherwise an older header, which sits in every automatic backup, would
    /// still open the data written after the change with the OLD password. The caller makes a
    /// fresh backup afterwards; older backups keep opening with the old password.
    pub fn change_credentials(
        &mut self,
        current_password: &str,
        current_keyfile: Option<&[u8; 32]>,
        new_password: &str,
        new_keyfile: Option<&[u8; 32]>,
        kdf: KdfParams,
    ) -> Result<()> {
        if self.open {
            return Err(Error::invalid("Security is off. Turn it on to set a master password."));
        }
        let header = self.store.read_header()?;
        let check = header.unlock(current_password, current_keyfile)?;
        if *check != *self.dek {
            return Err(Error::BadCredentials);
        }
        self.rekey(false, |dek, id| Header::protected_for(dek, id, new_password, new_keyfile, kdf))
    }

    /// Turns security on for an open vault: the notes get a master password (and keyfile).
    ///
    /// The vault is re-keyed, not just re-wrapped: a fresh data key and vault id, with every
    /// record and attachment re-encrypted, all in one transaction (all or nothing). Otherwise
    /// a copy made while security was off, which anyone can open, would still open this vault's
    /// later data, because it would share the data key.
    pub fn make_protected(&mut self, password: &str, keyfile: Option<&[u8; 32]>, kdf: KdfParams) -> Result<()> {
        if !self.open {
            return Err(Error::invalid("This vault already has a master password."));
        }
        self.rekey(false, |dek, id| Header::protected_for(dek, id, password, keyfile, kdf))
    }

    /// Turns security off for a protected vault. The current master password (and
    /// keyfile) must be given and must open the file, so a stray call can't do it.
    /// Re-keys the vault like `make_protected`, so older protected copies stay unreadable
    /// to anyone who only has the new, open file.
    pub fn make_open(&mut self, current_password: &str, current_keyfile: Option<&[u8; 32]>) -> Result<()> {
        if self.open {
            return Err(Error::invalid("Security is already off."));
        }
        let header = self.store.read_header()?;
        let check = header.unlock(current_password, current_keyfile)?;
        if *check != *self.dek {
            return Err(Error::BadCredentials);
        }
        self.rekey(true, |dek, id| Header::open_for(dek, id))
    }

    /// Gives the vault a fresh data key and vault id, re-encrypts every record and attachment
    /// (one at a time, so memory stays small) and writes the new header, all in one
    /// transaction. On any error nothing changes, in the file or in memory.
    fn rekey(&mut self, open_after: bool, header_for: impl FnOnce(&Key, [u8; 16]) -> Result<Header>) -> Result<()> {
        let new_dek = crypto::random_key();
        let new_id = crypto::random_bytes::<16>();
        let header = header_for(&new_dek, new_id)?;
        let (old_dek, old_id) = (&self.dek, self.vault_id);
        self.store.transaction(|s| {
            for table in [Table::Records, Table::Attachments] {
                for id in s.ids(table)? {
                    let sealed = s.get(table, &id)?.ok_or(Error::Corrupt)?;
                    let plain = crypto::open(old_dek, &aad(&old_id, table, &id), &sealed)?;
                    s.put(table, &id, &crypto::seal(&new_dek, &aad(&new_id, table, &id), &plain))?;
                }
            }
            s.write_header(&header)
        })?;
        self.dek = new_dek;
        self.vault_id = new_id;
        self.open = open_after;
        self.changed();
        Ok(())
    }

    pub fn backup_to(&self, dest: &Path) -> Result<()> {
        self.store.backup_to(dest)
    }

    /// Checks that the file at `path` is a complete copy of this vault: the same header
    /// (byte for byte), the same number of records and attachments, and every one of them
    /// authenticates with this session's data key. It needs no password, and it reads one
    /// row at a time. A backup is counted as made only after this passes.
    pub fn verify_copy(&self, path: &Path) -> Result<()> {
        let copy = Store::open(path)?;
        if copy.read_header()?.to_bytes() != self.store.read_header()?.to_bytes() {
            return Err(Error::Corrupt);
        }
        for table in [Table::Records, Table::Attachments] {
            let ids = copy.ids(table)?;
            if ids.len() != self.store.ids(table)?.len() {
                return Err(Error::Corrupt);
            }
            for id in ids {
                let sealed = copy.get(table, &id)?.ok_or(Error::Corrupt)?;
                let plain = crypto::open(&self.dek, &aad(&self.vault_id, table, &id), &sealed)?;
                crypto::unpad(&plain)?;
            }
        }
        Ok(())
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

    #[test]
    fn the_hourly_flag_is_kept_apart_from_the_lock_time_flag() {
        let path = temp_vault();
        let s = Session::create_open(&path).unwrap();
        assert!(s.dirty && s.hourly_due, "a new vault has unsaved changes");
        drop(s);
        let mut s = Session::unlock_open(&path).unwrap();
        assert!(!s.dirty && !s.hourly_due, "nothing changed since the vault was opened");
        s.put(page("one")).unwrap();
        assert!(s.dirty && s.hourly_due);
        // An hourly backup clears only its own flag, so the backup at lock or quit is still made.
        s.hourly_due = false;
        assert!(s.dirty);
        s.put(page("two")).unwrap();
        assert!(s.dirty && s.hourly_due, "a later change makes both due again");
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
        let att;
        {
            let mut s = Session::create(&path, "old", None, FAST).unwrap();
            s.put(p).unwrap();
            att = s.put_attachment("image/png", b"pixels").unwrap();
            assert!(s.change_credentials("wrong", None, "new", Some(&kf), FAST).is_err());
            s.change_credentials("old", None, "new", Some(&kf), FAST).unwrap();
            // The open session still works after the change.
            assert!(s.get(&pid).is_some());
            assert_eq!(s.get_attachment(&att).unwrap().1, b"pixels".to_vec());
            s.put(page("written after")).unwrap();
        }
        assert!(Session::unlock(&path, "old", None).is_err());
        assert!(Session::unlock(&path, "new", None).is_err());
        let s = Session::unlock(&path, "new", Some(&kf)).unwrap();
        assert!(s.get(&pid).is_some());
        assert_eq!(s.get_attachment(&att).unwrap().1, b"pixels".to_vec());
        assert_eq!(s.items().filter(|i| matches!(i, Item::Page(_))).count(), 2);
    }

    #[test]
    fn changing_the_password_gives_a_fresh_data_key_and_vault_id() {
        let path = temp_vault();
        let mut s = Session::create(&path, "old", None, FAST).unwrap();
        s.put(page("secret text")).unwrap();
        let before = Session::read_header(&path).unwrap();
        let old_key = before.unlock("old", None).unwrap();
        s.change_credentials("old", None, "new", None, FAST).unwrap();
        let after = Session::read_header(&path).unwrap();
        assert_ne!(after.vault_id, before.vault_id);
        assert_ne!(*after.unlock("new", None).unwrap(), *old_key, "the data key must change");
        assert!(after.unlock("old", None).is_err());
    }

    #[test]
    fn a_failed_password_change_changes_nothing() {
        let path = temp_vault();
        let mut s = Session::create(&path, "old", None, FAST).unwrap();
        s.put_attachment("image/png", b"pixels").unwrap();
        s.put(page("keep")).unwrap();
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute("UPDATE attachments SET data = x'00010203'", []).unwrap();
        }
        assert!(s.change_credentials("old", None, "new", None, FAST).is_err());
        drop(s);
        // The old header is still in place; only the corrupt attachment is the problem.
        assert!(Session::read_header(&path).unwrap().unlock("old", None).is_ok());
        assert!(Session::read_header(&path).unwrap().unlock("new", None).is_err());
    }

    /// The audit's scenario (F-01): the old header sits in every automatic backup. After a
    /// password change, that header plus the OLD password must not open data written later.
    #[test]
    fn an_old_header_and_old_password_do_not_open_data_written_after_a_password_change() {
        let path = temp_vault();
        let old_copy = path.with_file_name("old-copy.clf");
        let mut s = Session::create(&path, "old-password-1", None, FAST).unwrap();
        s.put(page("before")).unwrap();
        s.backup_to(&old_copy).unwrap(); // an automatic backup lying next to the vault
        let old_header = Session::read_header(&old_copy).unwrap();
        s.change_credentials("old-password-1", None, "new-password-2", None, FAST).unwrap();
        s.put(page("written after the change")).unwrap();
        drop(s);

        // Put the OLD header into a copy of the CURRENT file.
        let franken = path.with_file_name("franken.clf");
        std::fs::copy(&path, &franken).unwrap();
        Store::open(&franken).unwrap().write_header(&old_header).unwrap();
        assert!(
            Session::unlock(&franken, "old-password-1", None).is_err(),
            "old header + old password must not open the current data"
        );
        // The real file still opens with the new password and has everything.
        let ok = Session::unlock(&path, "new-password-2", None).unwrap();
        assert_eq!(ok.items().filter(|i| matches!(i, Item::Page(_))).count(), 2);
    }

    #[test]
    fn verify_copy_accepts_a_good_backup_and_rejects_damaged_ones() {
        let path = temp_vault();
        let mut s = Session::create(&path, "pw", None, FAST).unwrap();
        s.put(page("one")).unwrap();
        s.put_attachment("image/png", b"pixels").unwrap();
        let good = path.with_file_name("good.clf");
        s.backup_to(&good).unwrap();
        s.verify_copy(&good).unwrap();

        // A zero-byte file, a missing file and some other vault are all refused.
        let empty = path.with_file_name("empty.clf");
        std::fs::write(&empty, b"").unwrap();
        assert!(s.verify_copy(&empty).is_err());
        assert!(s.verify_copy(&path.with_file_name("missing.clf")).is_err());
        let other = temp_vault();
        drop(Session::create(&other, "pw", None, FAST).unwrap());
        assert!(s.verify_copy(&other).is_err());

        // One flipped byte in a record, and in an attachment.
        for table in ["records", "attachments"] {
            let bad = path.with_file_name(format!("bad-{table}.clf"));
            std::fs::copy(&good, &bad).unwrap();
            let conn = Connection::open(&bad).unwrap();
            let (id, mut data): (Vec<u8>, Vec<u8>) = conn
                .query_row(&format!("SELECT id, data FROM {table} LIMIT 1"), [], |r| Ok((r.get(0)?, r.get(1)?)))
                .unwrap();
            data[40] ^= 0xff;
            conn.execute(&format!("UPDATE {table} SET data = ?1 WHERE id = ?2"), params![data, id]).unwrap();
            drop(conn);
            assert!(s.verify_copy(&bad).is_err(), "a damaged {table} row must be caught");
        }

        // A row missing from the copy.
        let short = path.with_file_name("short.clf");
        std::fs::copy(&good, &short).unwrap();
        Connection::open(&short).unwrap().execute("DELETE FROM attachments", []).unwrap();
        assert!(s.verify_copy(&short).is_err());
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

    #[test]
    fn an_open_vault_reopens_without_a_password_and_keeps_its_notes() {
        let path = temp_vault();
        let p = page("plain notes");
        let pid = p.id().to_string();
        {
            let mut s = Session::create_open(&path).unwrap();
            assert!(s.is_open());
            s.put(p).unwrap();
        }
        let s = Session::unlock_open(&path).unwrap();
        assert!(s.is_open());
        assert!(matches!(s.get(&pid), Some(Item::Page(pg)) if pg.text == "plain notes"));
        assert!(!s.settings().screen_capture_protection, "screenshot protection is off by default");
    }

    #[test]
    fn an_open_vault_is_still_in_the_encrypted_format() {
        let path = temp_vault();
        {
            let mut s = Session::create_open(&path).unwrap();
            s.put(page("TopSecretDiaryPhrase")).unwrap();
        }
        let raw = std::fs::read(&path).unwrap();
        assert!(!raw.windows(20).any(|w| w == b"TopSecretDiaryPhrase"));
        assert!(!raw.windows(5).any(|w| w == b"Diary"));
    }

    #[test]
    fn a_protected_vault_does_not_open_without_its_password() {
        let path = temp_vault();
        drop(Session::create(&path, "pw", None, FAST).unwrap());
        assert!(matches!(Session::unlock_open(&path), Err(Error::BadCredentials)));
    }

    #[test]
    fn turning_security_on_and_off_keeps_every_note() {
        let path = temp_vault();
        let kf = crypto::random_bytes::<32>();
        let p = page("survives both");
        let pid = p.id().to_string();
        let att;
        {
            let mut s = Session::create_open(&path).unwrap();
            s.put(p).unwrap();
            att = s.put_attachment("image/png", b"pixels").unwrap();
            // Turn on.
            s.make_protected("new-password", Some(&kf), FAST).unwrap();
            assert!(!s.is_open());
            assert!(s.make_protected("again", None, FAST).is_err(), "already protected");
        }
        assert!(matches!(Session::unlock_open(&path), Err(Error::BadCredentials)));
        assert!(Session::unlock(&path, "new-password", None).is_err());
        {
            let mut s = Session::unlock(&path, "new-password", Some(&kf)).unwrap();
            assert!(!s.is_open());
            assert!(s.get(&pid).is_some());
            assert_eq!(s.get_attachment(&att).unwrap().1, b"pixels".to_vec());
            // Wrong password or keyfile changes nothing.
            assert!(matches!(s.make_open("wrong", Some(&kf)), Err(Error::BadCredentials)));
            assert!(matches!(s.make_open("new-password", None), Err(Error::BadCredentials)));
            assert!(!s.is_open());
            s.change_credentials("new-password", Some(&kf), "newer-password", None, FAST).unwrap();
            // Turn off.
            s.make_open("newer-password", None).unwrap();
            assert!(s.is_open());
            assert!(s.make_open("newer-password", None).is_err(), "already open");
            assert!(s.change_credentials("x", None, "y", None, FAST).is_err(), "no password to change");
        }
        let s = Session::unlock_open(&path).unwrap();
        assert!(s.get(&pid).is_some());
        assert_eq!(s.get_attachment(&att).unwrap().1, b"pixels".to_vec());
        assert!(Session::unlock(&path, "newer-password", None).is_err());
    }

    #[test]
    fn the_open_flag_is_read_back_from_the_file() {
        let path = temp_vault();
        drop(Session::create_open(&path).unwrap());
        assert!(Session::read_header(&path).unwrap().open);
        let path2 = temp_vault();
        drop(Session::create(&path2, "pw", None, FAST).unwrap());
        assert!(!Session::read_header(&path2).unwrap().open);
    }

    #[test]
    fn turning_security_on_or_off_gives_a_fresh_data_key() {
        let path = temp_vault();
        let old_copy = path.with_file_name("old-open-copy.clf");
        let mut s = Session::create_open(&path).unwrap();
        s.put(page("secret text")).unwrap();
        s.backup_to(&old_copy).unwrap(); // a copy made while security was off: anyone can open it
        let open_key = Session::read_header(&old_copy).unwrap().unlock_open().unwrap();
        let id_before = Session::read_header(&path).unwrap().vault_id;

        s.make_protected("Correct-Horse-Battery-9", None, FAST).unwrap();
        let h = Session::read_header(&path).unwrap();
        let prot_key = h.unlock("Correct-Horse-Battery-9", None).unwrap();
        assert_ne!(*open_key, *prot_key, "the old open copy's key must not open the protected vault");
        assert_ne!(h.vault_id, id_before);
        // The records of the live vault do not open with the old key.
        let swapped = std::fs::read(&path).unwrap();
        assert!(!swapped.windows(11).any(|w| w == b"secret text"));

        let protected_copy = path.with_file_name("protected-copy.clf");
        s.backup_to(&protected_copy).unwrap();
        s.make_open("Correct-Horse-Battery-9", None).unwrap();
        let h2 = Session::read_header(&path).unwrap();
        let open_key2 = h2.unlock_open().unwrap();
        // After turning security off, the key of the earlier protected copy is not the open vault's key.
        assert_ne!(*prot_key, *open_key2);
        assert_ne!(h2.vault_id, h.vault_id);
        drop(s);
        // Everything still opens: the live open vault, and the protected copy with its password.
        let live = Session::unlock_open(&path).unwrap();
        assert_eq!(live.items().filter(|i| matches!(i, Item::Page(_))).count(), 1);
        assert!(Session::unlock(&protected_copy, "Correct-Horse-Battery-9", None).is_ok());
        // The protected copy's records are not readable with the open vault's key.
        assert!(Session::unlock_open(&protected_copy).is_err());
    }

    #[test]
    fn a_failed_rekey_changes_nothing() {
        let path = temp_vault();
        let mut s = Session::create_open(&path).unwrap();
        let att = s.put_attachment("image/png", b"pixels").unwrap();
        s.put(page("keep")).unwrap();
        // Corrupt one attachment row so the re-encryption fails part way.
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute("UPDATE attachments SET data = x'00010203'", []).unwrap();
        }
        assert!(s.make_protected("Correct-Horse-Battery-9", None, FAST).is_err());
        assert!(s.is_open(), "still open");
        drop(s);
        // The header is still the open one and the records still open.
        let again = Session::unlock_open(&path).unwrap();
        assert_eq!(again.items().filter(|i| matches!(i, Item::Page(_))).count(), 1);
        let _ = att;
    }
}
