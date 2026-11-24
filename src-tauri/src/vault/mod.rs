//! An unlocked vault session. The data key and decrypted items live only here,
//! in Rust memory, and are wiped when the session is dropped (on lock).

pub mod header;
pub mod store;
pub mod history;
pub mod activity;
mod anchor;
pub(crate) mod backup;
mod import_pictures;
pub(crate) use import_pictures::ImportPictures;
#[cfg(test)]
mod security_tests;

use std::collections::{HashMap, HashSet};
use std::path::Path;

use zeroize::Zeroizing;

use crate::crypto::{self, KdfParams, Key};
use crate::actions::Actor;
use crate::changes::{self, Coordination, ItemsChanged, Revisioned};
use crate::error::{Error, Result};
use crate::model::{Item, Settings, SETTINGS_ID};
use header::{Format, Header};
use store::{RowId, Store, Table};

/// Largest single item accepted (a long page with lots of text).
const MAX_ITEM_BYTES: usize = 32 * 1024 * 1024;
/// Largest image attachment. One SQLite value holds at most 1,000,000,000 bytes, so 900 MB is the ceiling for
/// anything stored as one encrypted record (the owner asked for 2 GB on 2026-11-04; see EX-2026-11-04-43).
pub const MAX_ATTACHMENT_BYTES: usize = 900 * 1024 * 1024;

pub struct Session {
    // Memory-only identity: a dialog opened before lock/unlock cannot approve a
    // later session, even when it opens the same vault with the same row revisions.
    native_session_id: [u8; 16],
    /// The format the file is in. A vault from V1.3.0 or earlier is `Legacy` until `upgrade_format` runs.
    format: Format,
    dek: Key,
    vault_id: [u8; 16],
    store: Store,
    items: HashMap<String, Item>,
    // Dropped with the session. Interior mutability is serialized by AppState's
    // session mutex, alongside item reads, writes and session replacement.
    pub(crate) query_cache: std::cell::RefCell<crate::database::windows::QueryCache>,
    /// Security is off (amendment A21): the data key is wrapped under the built-in key.
    open: bool,
    /// True once anything changed since unlock; triggers a backup on lock.
    pub dirty: bool,
    /// True once anything changed since the last hourly backup of an open vault. It is kept
    /// apart from `dirty` so an hourly copy never stands in for the backup made at lock or quit.
    pub hourly_due: bool,
    pub revs: HashMap<String, u64>,
    pending: std::collections::BTreeMap<String, (String, String)>,
    pub coordination: Coordination,
    actor: Option<Actor>,
    ai_writers: HashMap<String, (u64, Actor)>,
    replacement: Option<crate::model::ReplacedEdit>,
    history_index: history::Index,
    change_set: Option<String>,
    request_ai: Option<Actor>,
    action_name: Option<&'static str>,
    activity_head: activity::Head,
    anchor: anchor::Anchor,
    person_bursts: HashMap<String, i64>,
    journal_maintenance_at: i64,
    ai_action_failed: bool,
    ai_action_notices: Vec<crate::commands::database::Reminder>,
    ai_safety_backup: bool,
}

/// AAD binds each ciphertext to this vault, its table and its row id, so rows
/// can't be moved, swapped or copied between vaults without detection.
fn aad(format: Format, vault_id: &[u8; 16], table: Table, id: &RowId) -> Vec<u8> {
    let mut out = Vec::with_capacity(48);
    out.extend_from_slice(format.row_binding());
    out.extend_from_slice(&format.version().to_le_bytes());
    out.extend_from_slice(vault_id);
    out.extend_from_slice(table.tag());
    out.extend_from_slice(id);
    out
}

pub fn parse_id(id: &str) -> Result<RowId> {
    let bytes = hex::decode(id).map_err(|_| Error::NotFound)?;
    bytes.try_into().map_err(|_| Error::NotFound)
}

/// Keep even a bare attachment id in a content field. Scanning hex runs
/// once avoids searching every surviving note body once per attachment.
fn keep_attachment_refs(unreferenced: &mut HashSet<String>, text: &str) {
    if unreferenced.is_empty() {
        return;
    }
    for run in text.split(|c: char| !c.is_ascii_hexdigit()).filter(|run| run.len() >= 32) {
        for start in 0..=run.len() - 32 {
            unreferenced.remove(&run[start..start + 32]);
        }
    }
}

impl Session {
    pub(crate) fn native_session_id(&self) -> [u8; 16] { self.native_session_id }
    /// One dispatcher request, including nested cores, journals and attachment
    /// cleanup. The session mutex remains held until commit and event enqueue.
    pub(crate) fn atomic_action<T>(&mut self, safety_backup: bool, f: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        let snapshot = (self.items.clone(), self.dirty, self.hourly_due,
            self.revs.clone(), self.pending.clone(), self.coordination.clone(),
            self.actor.clone(), self.ai_writers.clone(), self.replacement.clone(),
            self.history_index.clone(), self.change_set.clone(), self.request_ai.clone(),
            self.action_name, self.activity_head.clone(), self.anchor.clone(), self.person_bursts.clone(),
            self.journal_maintenance_at, self.ai_action_failed, self.ai_action_notices.clone(), self.ai_safety_backup);
        self.store.begin_action()?;
        self.ai_action_failed = false;
        self.ai_action_notices.clear();
        self.ai_safety_backup = safety_backup;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(self)));
        let result = match result {
            Ok(Ok(_)) if self.ai_action_failed => Ok(Err(Error::invalid("invalid"))),
            result => result,
        };
        let result = match result {
            Ok(Ok(value)) => match self.store.commit_action() {
                Ok(()) => {
                    self.ai_safety_backup = false;
                    crate::commands::automation::publish_notices(std::mem::take(&mut self.ai_action_notices));
                    return Ok(value);
                },
                Err(error) => Ok(Err(error)),
            },
            other => other,
        };
        self.store.rollback_action();
        self.clear_query_cache();
        (self.items, self.dirty, self.hourly_due, self.revs, self.pending,
            self.coordination, self.actor, self.ai_writers, self.replacement,
            self.history_index, self.change_set, self.request_ai, self.action_name,
            self.activity_head, self.anchor, self.person_bursts, self.journal_maintenance_at,
            self.ai_action_failed, self.ai_action_notices, self.ai_safety_backup) = snapshot;
        match result { Ok(result) => result, Err(panic) => std::panic::resume_unwind(panic) }
    }

    pub(crate) fn ai_safety_backup_ready(&self) -> bool { self.ai_safety_backup }

    pub(crate) fn automation_failed(&mut self) {
        if self.request_ai.is_some() { self.ai_action_failed = true; }
    }
    pub(crate) fn automation_notices(&mut self, notices: Vec<crate::commands::database::Reminder>) {
        if self.request_ai.is_some() {
            self.ai_action_notices.extend(notices);
        } else {
            crate::commands::automation::publish_notices(notices);
        }
    }

    /// Records that something changed, for the backups made at lock or quit and the hourly ones.
    fn changed(&mut self) {
        self.dirty = true;
        self.hourly_due = true;
    }

    pub fn create(path: &Path, password: &str, keyfile: Option<&[u8; 32]>, kdf: KdfParams) -> Result<Session> {
        let (header, dek) = Header::create(password, keyfile, kdf)?;
        let store = Store::create(path, &header)?;
        let mut session = Session { native_session_id: crypto::random_bytes(),
            format: header.format,
            dek,
            vault_id: header.vault_id,
            store,
            items: HashMap::new(),
            open: false,
            dirty: true,
            hourly_due: true,
            revs: Default::default(), pending: Default::default(), coordination: Default::default(), actor: None, ai_writers: Default::default(), replacement: None,
            query_cache: Default::default(), history_index: Default::default(), change_set: None, request_ai: None, action_name: None, activity_head: Default::default(), anchor: Default::default(), person_bursts: Default::default(), journal_maintenance_at: 0, ai_action_failed: false, ai_action_notices: Vec::new(), ai_safety_backup: false,
        };
        session.put(Item::Settings(Settings::default()))?;
        Ok(session)
    }

    /// Creates a new vault with security off: no master password, opened by the built-in key.
    pub fn create_open(path: &Path) -> Result<Session> {
        let (header, dek) = Header::create_open()?;
        let store = Store::create(path, &header)?;
        let mut session = Session { native_session_id: crypto::random_bytes(),
            format: header.format,
            dek,
            vault_id: header.vault_id,
            store,
            items: HashMap::new(),
            open: true,
            dirty: true,
            hourly_due: true,
            revs: Default::default(), pending: Default::default(), coordination: Default::default(), actor: None, ai_writers: Default::default(), replacement: None,
            query_cache: Default::default(), history_index: Default::default(), change_set: None, request_ai: None, action_name: None, activity_head: Default::default(), anchor: Default::default(), person_bursts: Default::default(), journal_maintenance_at: 0, ai_action_failed: false, ai_action_notices: Vec::new(), ai_safety_backup: false,
        };
        session.put(Item::Settings(Settings::default()))?;
        Ok(session)
    }

    /// Makes a vault the way V1.3.0 and earlier did (the old file format). Only tests use it, to
    /// have something to upgrade.
    #[cfg(test)]
    pub fn create_legacy(path: &Path, password: &str, keyfile: Option<&[u8; 32]>, kdf: KdfParams) -> Result<Session> {
        let (header, dek) = Header::create_legacy(password, keyfile, kdf)?;
        Self::new_in(path, header, dek, false)
    }

    /// Like [`Session::create_legacy`], with security off.
    #[cfg(test)]
    pub fn create_legacy_open(path: &Path) -> Result<Session> {
        let (header, dek) = Header::create_legacy_open()?;
        Self::new_in(path, header, dek, true)
    }

    #[cfg(test)]
    fn new_in(path: &Path, header: Header, dek: Key, open: bool) -> Result<Session> {
        let store = Store::create(path, &header)?;
        let mut session = Session { native_session_id: crypto::random_bytes(),
            format: header.format,
            dek,
            vault_id: header.vault_id,
            store,
            items: HashMap::new(),
            open,
            dirty: true,
            hourly_due: true,
            revs: Default::default(), pending: Default::default(), coordination: Default::default(), actor: None, ai_writers: Default::default(), replacement: None,
            query_cache: Default::default(), history_index: Default::default(), change_set: None, request_ai: None, action_name: None, activity_head: Default::default(), anchor: Default::default(), person_bursts: Default::default(), journal_maintenance_at: 0, ai_action_failed: false, ai_action_notices: Vec::new(), ai_safety_backup: false,
        };
        session.put(Item::Settings(Settings::default()))?;
        Ok(session)
    }

    pub fn read_header(path: &Path) -> Result<Header> {
        Store::open_read_only(path)?.read_header()
    }

    /// Reads the live header for the lock screen without recovering the live vault.
    pub(crate) fn read_live_header(paths: &crate::portable::Paths) -> Result<Header> {
        backup::with_preflight(&paths.data, &paths.vault, |_, header, _| Ok(header))
    }

    /// Both live-vault entry points call this while holding AppState's open gate.
    /// Authenticate and finish the safety copy before opening the writable store.
    pub(crate) fn unlock_live(
        paths: &crate::portable::Paths,
        credentials: Option<(&str, Option<&[u8; 32]>)>,
    ) -> Result<Option<(Session, Option<String>)>> {
        backup::clean_stale_preflight(&paths.data);
        let preflight = backup::with_preflight(
            &paths.data,
            &paths.vault,
            |source, header, recovered| {
                let dek = match credentials {
                    Some((password, keyfile)) => header.unlock(password, keyfile)?,
                    None if header.open => header.unlock_open()?,
                    None => return Ok(None),
                };
                let name = if backup::needs_upgrade(&source, &header, &dek)? {
                    Some(backup::before_upgrade(&paths.data, &source, &header, &dek)?)
                } else {
                    if recovered {
                        // Validate existing records and journals before recovery
                        // on the live file, even when no safety copy is due.
                        Self::load(source, header.clone(), dek.clone())?;
                    }
                    None
                };
                Ok(Some((header, dek, name)))
            },
        )?;
        let Some((header, dek, name)) = preflight else {
            return Ok(None);
        };

        let store = Store::open(&paths.vault)?;
        if name.is_some() {
            // DDL, the anchor and settings latch, and the legacy re-key must all
            // roll back together. A failed attempt leaves the saved copy reusable.
            store.begin_action()?;
        }
        if store.read_header()?.to_bytes() != header.to_bytes() {
            return Err(Error::Corrupt);
        }
        let mut session = Self::load(store, header, dek)?;
        if name.is_some() {
            let (password, keyfile) = credentials.unwrap_or(("", None));
            session.upgrade_format(password, keyfile)?;
            session.store.commit_action()?;
        }
        Ok(Some((session, name)))
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
            let plain = crypto::open(&dek, &aad(header.format, &header.vault_id, Table::Records, &id), &data)?;
            let item: Item = serde_json::from_slice(crypto::unpad(&plain)?).map_err(|_| Error::Corrupt)?;
            if item.id() != hex::encode(id) {
                return Err(Error::Corrupt);
            }
            items.insert(item.id().to_string(), item);
        }
        store.ensure_journal_tables()?;
        let mut session = Session { native_session_id: crypto::random_bytes(), format: header.format, dek, vault_id: header.vault_id, store, items, open: header.open, dirty: false, hourly_due: false, revs: Default::default(), pending: Default::default(), coordination: Default::default(), actor: None, ai_writers: Default::default(), replacement: None,
            query_cache: Default::default(), history_index: Default::default(), change_set: None, request_ai: None, action_name: None, activity_head: Default::default(), anchor: Default::default(), person_bursts: Default::default(), journal_maintenance_at: 0, ai_action_failed: false, ai_action_notices: Vec::new(), ai_safety_backup: false };
        session.load_history()?;
        session.load_activity_head()?;
        session.load_anchor()?;
        Ok(session)
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
        Ok((id, crypto::seal(&self.dek, &aad(self.format, &self.vault_id, Table::Records, &id), &padded)))
    }

    fn seal_json<T: serde::Serialize>(&self, table: Table, id: &RowId, value: &T) -> Result<Vec<u8>> {
        let bytes = Zeroizing::new(serde_json::to_vec(value).map_err(|_| Error::Corrupt)?);
        Ok(crypto::seal(&self.dek, &aad(self.format, &self.vault_id, table, id), &crypto::pad(&bytes)))
    }

    fn open_json<T: serde::de::DeserializeOwned>(&self, table: Table, id: &RowId, sealed: &[u8]) -> Result<T> {
        let plain = crypto::open(&self.dek, &aad(self.format, &self.vault_id, table, id), sealed)?;
        serde_json::from_slice(crypto::unpad(&plain)?).map_err(|_| Error::Corrupt)
    }

    fn read_json<T: serde::de::DeserializeOwned>(&self, table: Table, id: &RowId) -> Result<Option<T>> {
        self.store.get(table, id)?.map(|bytes| self.open_json(table, id, &bytes)).transpose()
    }

    /// Encrypts and stores an item (insert or update).
    pub fn put(&mut self, item: Item) -> Result<()> {
        self.put_many(vec![item])
    }

    /// Stores several items atomically: either all are saved or none.
    pub fn put_many(&mut self, items: Vec<Item>) -> Result<()> {
        self.put_many_checked(items, None, |_, _| Ok(()), |_| Ok(()))
    }

    /// Reports sealed items. The last checkpoint precedes the backup and any journal or
    /// record writes. An import's pictures join its records in the same transaction.
    pub(crate) fn put_many_checked(
        &mut self,
        mut items: Vec<Item>,
        mut pictures: Option<ImportPictures>,
        mut checkpoint: impl FnMut(usize, usize) -> Result<()>,
        before_write: impl FnOnce(&Session) -> Result<()>,
    ) -> Result<()> {
        for item in &mut items {
            if let Item::Settings(settings) = item { settings.journals_anchored = true; }
        }
        let mut seen = std::collections::HashSet::new();
        items.reverse(); items.retain(|item| seen.insert(item.id().to_string())); items.reverse();
        let actor = self.actor.clone().unwrap_or(Actor::Person);
        self.ai_write_targets(&actor, &items)?;
        for item in &items {
            self.check_write(&actor, item.id(), None)?;
            if let Some(parent) = Self::parent_of(item) {
                self.coordination.check(&actor, |other| self.is_within(parent, other))?;
            }
        }
        self.validate_hierarchy(&items)?;
        let replacing = self.replacement.as_ref().is_some_and(|r| items.iter().any(|i| i.id() == r.id));
        if replacing {
            let mut settings = self.settings();
            settings.replaced_edits.push(self.replacement.as_ref().unwrap().clone());
            items.push(Item::Settings(settings));
        }
        let removed_templates: std::collections::BTreeMap<String, std::collections::HashSet<String>> = items.iter().filter_map(|item| {
            let Item::Database(new) = item else { return None; };
            let Some(Item::Database(old)) = self.get(&new.id) else { return None; };
            let removed: std::collections::HashSet<_> = old.templates.iter().filter(|t| !new.templates.iter().any(|n| n.id == t.id)).map(|t| t.id.clone()).collect();
            (!removed.is_empty()).then_some((new.id.clone(), removed))
        }).collect();
        if !removed_templates.is_empty() {
            self.activity_verify()?;
            // A person's replacement-history copy can also contain a template.
            // Remove those copies at the same boundary as indirect activity undo.
            let mut settings = items.iter().find_map(|i| if let Item::Settings(s) = i { Some(s.clone()) } else { None }).unwrap_or_else(|| self.settings());
            let before = settings.replaced_edits.len();
            settings.replaced_edits.retain(|e| !removed_templates.contains_key(&e.id));
            if before != settings.replaced_edits.len() {
                items.retain(|i| !matches!(i, Item::Settings(_)));
                items.push(Item::Settings(settings));
            }
        }
        let total = items.len();
        checkpoint(0, total)?;
        let mut sealed = Vec::with_capacity(total);
        for (index, item) in items.iter().enumerate() {
            sealed.push(self.seal_item(item)?);
            checkpoint(index + 1, total)?;
        }
        checkpoint(total + 1, total)?;
        before_write(self)?;
        self.maintain_journals()?;
        let journal = self.prepare_journal(&actor, &items, &[])?;
        let purged = self.store.transaction(|s| {
            if let Some(pictures) = &mut pictures {
                pictures.write_to(s)?;
            }
            for (id, data) in &sealed {
                s.put(Table::Records, id, data)?;
            }
            journal.write(s)?;
            if removed_templates.is_empty() { return Ok(None); }
            let head = self.purge_undo_in_store(journal.head(&self.activity_head), &HashSet::new(), &removed_templates)?;
            let anchor = self.refresh_anchor_head(journal.anchor.next.clone(), &head, s)?;
            // Template removal has no delete_bin_tree caller to collect files.
            // Use the prospective items and just-written snapshots/undo here,
            // so file cleanup is atomic with the purge and keeps other content.
            let replaced: std::collections::HashSet<_> = items.iter().map(Item::id).collect();
            self.collect_garbage_for(
                self.items.values().filter(|item| !replaced.contains(item.id())).chain(items.iter()),
                &head, journal.picture_ids(), &HashSet::new(),
            )?;
            Ok(Some((head, anchor)))
        })?;
        self.accept_journal(journal);
        if let Some((head, anchor)) = purged { self.activity_head = head; self.anchor = anchor; }
        for item in items {
            self.record_change(&item, &actor);
            self.items.insert(item.id().to_string(), item);
        }
        if replacing { self.replacement = None; }
        self.changed();
        Ok(())
    }

    #[cfg(test)]
    pub fn remove_many(&mut self, ids: &[String]) -> Result<()> {
        self.purge_many(ids, Vec::new())
    }

    /// Permanently delete items and publish any surviving relation edits as
    /// one durable write. The history, activity and encrypted attachments share
    /// that write, so a failed purge cannot leave partially removed data.
    pub(crate) fn purge_many(&mut self, ids: &[String], mut updates: Vec<Item>) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        let mut removed = HashSet::new();
        let ids: Vec<_> = ids.iter().filter(|id| removed.insert(id.as_str())).cloned().collect();
        self.activity_verify()?;
        let actor = self.actor.clone().unwrap_or(Actor::Person);
        self.ai_write_targets(&actor, &updates)?;
        for item in &updates {
            self.check_write(&actor, item.id(), None)?;
            if let Some(parent) = Self::parent_of(item) {
                self.coordination.check(&actor, |other| self.is_within(parent, other))?;
            }
        }
        for id in &ids {
            self.check_write(&actor, id, None)?;
        }

        let mut settings = self.settings();
        let before = settings.replaced_edits.len();
        settings.replaced_edits.retain(|e| !removed.contains(e.id.as_str()));
        let settings = (before != settings.replaced_edits.len()).then_some(Item::Settings(settings));
        let sealed_settings = settings.as_ref().map(|i| self.seal_item(i)).transpose()?;
        let sealed_updates = updates.iter().map(|item| self.seal_item(item)).collect::<Result<Vec<_>>>()?;
        let rows = ids.iter().map(|id| parse_id(id)).collect::<Result<Vec<_>>>()?;
        let versions: Vec<_> = ids.iter().filter_map(|id| self.history_index.get(id)).flatten()
            .map(|m| (m.version_id.clone(), None)).collect();
        let mut history_rows = Vec::with_capacity(2 * versions.len());
        for (id, _) in &versions {
            history_rows.push(parse_id(id)?);
            history_rows.push(history::body_id(id)?);
        }
        let mut journal = self.prepare_journal(&actor, &updates, &ids)?;
        journal.anchor = self.prepare_anchor(
            journal.head(&self.activity_head),
            journal.history_additions().chain(versions).collect(),
        )?;
        updates.extend(settings);
        let replaced: HashSet<_> = updates.iter().map(Item::id).collect();
        let (head, anchor) = self.store.transaction(|s| {
            if let Some((id, bytes)) = &sealed_settings {
                s.put(Table::Records, id, bytes)?;
            }
            for (id, bytes) in &sealed_updates {
                s.put(Table::Records, id, bytes)?;
            }
            s.delete_many(Table::Records, &rows)?;
            s.delete_many(Table::History, &history_rows)?;
            journal.write(s)?;
            let head = self.purge_undo_in_store(journal.head(&self.activity_head), &removed, &Default::default())?;
            let anchor = self.refresh_anchor_head(journal.anchor.next.clone(), &head, s)?;
            self.collect_garbage_for(
                self.items.values()
                    .filter(|item| !removed.contains(item.id()) && !replaced.contains(item.id()))
                    .chain(updates.iter()),
                &head, journal.picture_ids(), &removed,
            )?;
            Ok((head, anchor))
        })?;

        self.accept_journal(journal);
        self.activity_head = head;
        self.anchor = anchor;
        self.clear_query_cache();
        for id in &ids {
            self.history_index.remove(id);
            if let Some(item) = self.items.remove(id) {
                self.note_change(&item, &actor);
            }
            self.person_bursts.remove(id);
            self.ai_writers.remove(id);
        }
        for item in updates {
            self.note_change(&item, &actor);
            self.items.insert(item.id().into(), item);
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

    pub fn rev(&self, id: &str) -> u64 { self.revs.get(id).copied().unwrap_or(0) }

    pub fn revisioned<T: serde::Serialize>(&self, id: &str, item: T) -> Revisioned<T> {
        Revisioned { item, rev: self.rev(id) }
    }

    pub fn parent_of(item: &Item) -> Option<&str> {
        match item {
            Item::Section(x) => Some(&x.notebook_id),
            Item::Page(x) => Some(if x.parent.is_empty() { &x.section_id } else { &x.parent }),
            Item::Database(x) => Some(if x.parent.is_empty() { &x.section_id } else { &x.parent }),
            Item::Row(x) => Some(&x.db), _ => None,
        }
    }

    fn validate_hierarchy(&self, items: &[Item]) -> Result<()> {
        use std::collections::hash_map::Entry;

        let proposed: HashMap<_, _> = items.iter().filter_map(|item| {
            let id = item.id();
            let parent = Self::parent_of(item);
            (parent != self.get(id).and_then(Self::parent_of)).then_some((id, parent))
        }).collect();

        // Check changed edges and their ancestors so unrelated cycles in older
        // vaults do not prevent writes. Each walk can reuse earlier walks.
        let mut visited = HashMap::new();
        for (walk, root) in proposed.keys().copied().enumerate() {
            let mut current = Some(root);
            while let Some(id) = current {
                match visited.entry(id) {
                    Entry::Occupied(entry) if *entry.get() == walk => {
                        return Err(Error::invalid("An item cannot be contained in its own descendants."));
                    }
                    Entry::Occupied(_) => break,
                    Entry::Vacant(entry) => {
                        entry.insert(walk);
                        current = proposed.get(id).copied()
                            .unwrap_or_else(|| self.get(id).and_then(Self::parent_of));
                    }
                }
            }
        }
        Ok(())
    }

    /// Authorize both existing locations and the actual prospective ancestry.
    /// Used by undo preview/recheck and by the final shared write boundary.
    pub(crate) fn ai_write_targets(&self, actor: &Actor, items: &[Item]) -> Result<Vec<String>> {
        let actor = self.request_ai.as_ref().unwrap_or(actor);
        if !matches!(actor, Actor::Ai { .. }) { return Ok(vec![]); }
        let proposed: HashMap<_, _> = items.iter().map(|i| (i.id(), i)).collect();
        let hidden = self.settings().ai.hidden_notebooks;
        let mut targets = std::collections::HashSet::new();
        for item in items {
            // Only a changed item affects its descendants. Destination ancestors
            // must be checked themselves, without treating siblings as writes.
            self.check_write(actor, item.id(), None)?;
            for prospective in [false, true] {
                let mut pending = vec![(item.id().to_string(), std::collections::HashSet::new())];
                while let Some((id, mut path)) = pending.pop() {
                    if id.is_empty() { continue; }
                    if !path.insert(id.clone()) { return Err(Error::invalid("invalid")); }
                    if hidden.contains(&id) { return Err(Error::NotFound); }
                    targets.insert(id.clone());
                    self.coordination.check(actor, |other| other == id)?;
                    let current = self.get(&id);
                    let next = if prospective { proposed.get(id.as_str()).copied().or(current) } else { current };
                    if path.len() > 1 && next.is_none() { return Err(Error::NotFound); }
                    if [current, next].into_iter().flatten().any(|i| matches!(i, Item::Database(d) if d.locked)) {
                        return Err(Error::invalid("invalid"));
                    }
                    if let Some(next) = next {
                        if let Some(parent) = Self::parent_of(next) {
                            pending.push((parent.to_string(), path.clone()));
                        }
                        // A page/database also retains its section reference when parented.
                        let section = match next {
                            Item::Page(p) if !p.parent.is_empty() => Some(&p.section_id),
                            Item::Database(d) if !d.parent.is_empty() => Some(&d.section_id),
                            _ => None,
                        };
                        if let Some(section) = section { pending.push((section.clone(), path.clone())); }
                    }
                }
            }
        }
        Ok(targets.into_iter().collect())
    }

    pub fn is_within(&self, id: &str, root: &str) -> bool {
        let mut current = Some(id);
        let mut seen = std::collections::HashSet::new();
        while let Some(id) = current {
            if id == root { return true; }
            if !seen.insert(id) { break; }
            current = self.get(id).and_then(Self::parent_of);
        }
        false
    }

    /// Includes the roots and all their descendants exactly once, even if
    /// several bin entries overlap or a damaged tree contains a cycle.
    pub(crate) fn subtree_ids(&self, roots: &[String]) -> Vec<String> {
        let mut children: HashMap<&str, Vec<&str>> = HashMap::new();
        for item in self.items() {
            if let Some(parent) = Self::parent_of(item) {
                children.entry(parent).or_default().push(item.id());
            }
        }
        let mut pending: Vec<_> = roots.iter().filter_map(|id| self.get(id)).map(Item::id).collect();
        let mut found = HashSet::new();
        while let Some(id) = pending.pop() {
            if found.insert(id) {
                if let Some(child_ids) = children.get(id) {
                    pending.extend(child_ids);
                }
            }
        }
        let mut ids: Vec<_> = found.into_iter().map(str::to_owned).collect();
        ids.sort_unstable();
        ids
    }

    pub fn own_trash(item: &Item) -> Option<i64> {
        match item { Item::Notebook(x) => x.trashed_at, Item::Section(x) => x.trashed_at,
            Item::Page(x) => x.trashed_at, Item::Sticky(x) => x.trashed_at,
            Item::Database(x) => x.trashed_at, Item::Row(x) => x.trashed_at, _ => None }
    }

    pub fn in_bin(&self, id: &str) -> bool {
        let mut current = Some(id);
        let mut seen = std::collections::HashSet::new();
        while let Some(id) = current {
            if !seen.insert(id) { return true; }
            let Some(item) = self.get(id) else { return false };
            if Self::own_trash(item).is_some() { return true; }
            current = Self::parent_of(item);
        }
        false
    }

    pub fn check_write(&self, actor: &Actor, id: &str, expect_rev: Option<u64>) -> Result<()> {
        if matches!(actor, Actor::Ai { .. }) {
            self.coordination.check(actor, |other| self.is_within(id, other) || self.is_within(other, id))?;
            if expect_rev.is_some_and(|r| r != self.rev(id)) {
                return Err(Error::invalid("revision_conflict: This item changed; read it again."));
            }
        }
        Ok(())
    }

    /// Trusted action dispatcher entry: check all targets before invoking a write core.
    pub fn write_as<T>(&mut self, actor: &Actor, ids: &[String], expect_rev: Option<u64>, f: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        for id in ids { self.check_write(actor, id, expect_rev)?; }
        self.run_as(actor, f)
    }

    pub fn run_as<T>(&mut self, actor: &Actor, f: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        let outer = self.change_set.is_none();
        if outer { self.change_set = Some(crate::model::new_id()); }
        let previous_ai = self.request_ai.clone();
        if matches!(actor, Actor::Ai { .. }) { self.request_ai = Some(actor.clone()); }
        let previous = self.actor.replace(actor.clone());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(self)));
        self.actor = previous;
        self.request_ai = previous_ai;
        if outer { self.change_set = None; }
        match result { Ok(result) => result, Err(panic) => std::panic::resume_unwind(panic) }
    }

    /// C5 supplies the compiled action name here, outside action parameters.
    /// Nested cores and automations share this request's change set.
    pub fn run_action<T>(&mut self, actor: &Actor, name: &'static str, f: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        let previous = self.action_name.replace(name);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.run_as(actor, f)));
        self.action_name = previous;
        match result { Ok(result) => result, Err(panic) => std::panic::resume_unwind(panic) }
    }

    pub fn claim(&mut self, actor: &Actor, ids: &[String]) -> Result<()> {
        for id in ids {
            if self.get(id).is_none() { return Err(Error::NotFound); }
            self.check_write(actor, id, None)?;
        }
        self.coordination.claim(actor, ids, false)
    }

    pub fn person_save<T>(&mut self, id: &str, base_rev: Option<u64>, f: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        if base_rev.is_some_and(|r| r < self.rev(id)) {
            if let (Some((ai_rev, Actor::Ai { label, .. })), Some(item)) = (self.ai_writers.get(id), self.get(id)) {
                if base_rev.is_some_and(|base| base < *ai_rev) {
                self.replacement = Some(crate::model::ReplacedEdit { id: id.into(), rev: self.rev(id),
                    by: format!("ai:{label}"), status: "replaced by your edit".into(),
                    previous: serde_json::to_string(item).map_err(|_| Error::Corrupt)? });
                }
            }
        }
        let result = self.run_as(&Actor::Person, f);
        self.replacement = None;
        if result.is_ok() {
            self.ai_writers.remove(id);
            self.coordination.editing(id, "save", false);
        }
        result
    }

    pub(crate) fn clear_query_cache(&mut self) {
        self.query_cache.get_mut().invalidate();
    }

    fn record_change(&mut self, item: &Item, actor: &Actor) {
        // Use the vault-wide write boundary. Relations can span several
        // databases, and item-map changes can reorder rows tied on manual order.
        self.clear_query_cache();
        self.note_change(item, actor);
    }

    fn note_change(&mut self, item: &Item, actor: &Actor) {
        let id = item.id().to_string();
        *self.revs.entry(id.clone()).or_default() += 1;
        let kind = match item { Item::Notebook(_) => "notebook", Item::Section(_) => "section",
            Item::Page(_) => "page", Item::Sticky(_) => "sticky", Item::Database(_) => "database",
            Item::Row(_) => "row", Item::Settings(_) => "settings" };
        self.pending.insert(id.clone(), (kind.into(), changes::by(actor)));
        if matches!(actor, Actor::Ai { .. }) {
            self.ai_writers.insert(id.clone(), (self.rev(&id), actor.clone()));
            let mut ancestors = std::collections::HashSet::from([id.clone()]);
            // New items are not in the cache until after record_change.
            let mut current = Self::parent_of(item);
            while let Some(parent) = current {
                if !ancestors.insert(parent.to_string()) { break; }
                current = self.get(parent).and_then(Self::parent_of);
            }
            self.coordination.touch(actor, |root| ancestors.contains(root));
            let _ = self.coordination.claim(actor, &[id], true);
        }
    }

    pub fn drain_changes(&mut self) -> Option<ItemsChanged> {
        if self.pending.is_empty() { return None; }
        let pending = std::mem::take(&mut self.pending);
        let ids: Vec<_> = pending.keys().cloned().collect();
        let kinds: Vec<_> = pending.values().map(|(kind, _)| kind.clone()).collect();
        let by = pending.values().last().unwrap().1.clone();
        let revs = ids.iter().map(|id| (id.clone(), self.rev(id))).collect();
        Some(ItemsChanged { ids, kinds, tree: pending.values().any(|(k, _)| k != "settings"), by, revs })
    }

    pub fn settings(&self) -> Settings {
        match self.items.get(SETTINGS_ID) {
            Some(Item::Settings(s)) => s.clone().sanitized(),
            _ => Settings::default(),
        }
    }

    pub fn put_attachment(&mut self, mime: &str, bytes: &[u8]) -> Result<String> {
        if bytes.len() > MAX_ATTACHMENT_BYTES {
            return Err(Error::invalid("Images must be smaller than 900 MB."));
        }
        self.put_blob(mime, bytes)
    }

    /// A file attached to a database row (any type; at most `database::types::MAX_FILE_BYTES`, 900 MB).
    pub fn put_file(&mut self, mime: &str, bytes: &[u8]) -> Result<String> {
        if bytes.len() > crate::database::types::MAX_FILE_BYTES {
            return Err(Error::invalid("Files must be smaller than 900 MB."));
        }
        self.put_blob(mime, bytes)
    }

    fn put_blob(&mut self, mime: &str, bytes: &[u8]) -> Result<String> {
        let id = crypto::random_bytes();
        let sealed = self.seal_blob(&id, mime, bytes);
        self.store.put(Table::Attachments, &id, &sealed)?;
        self.changed();
        Ok(hex::encode(id))
    }

    fn seal_blob(&self, id: &RowId, mime: &str, bytes: &[u8]) -> Vec<u8> {
        let mut payload = Zeroizing::new(Vec::with_capacity(1 + mime.len() + bytes.len()));
        payload.push(mime.len() as u8);
        payload.extend_from_slice(mime.as_bytes());
        payload.extend_from_slice(bytes);
        let padded = crypto::pad(&payload);
        crypto::seal(&self.dek, &aad(self.format, &self.vault_id, Table::Attachments, id), &padded)
    }

    pub fn get_attachment(&self, id_hex: &str) -> Result<(String, Vec<u8>)> {
        let id = parse_id(id_hex)?;
        let data = self.store.get(Table::Attachments, &id)?.ok_or(Error::NotFound)?;
        let plain = crypto::open(&self.dek, &aad(self.format, &self.vault_id, Table::Attachments, &id), &data)?;
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
        self.verify_journals()?;
        Ok(())
    }

    /// Deletes attachments no page or sticky note refers to any more.
    pub fn collect_garbage(&mut self) -> Result<usize> {
        let removed = self.store.transaction(|_| {
            self.collect_garbage_for(
                self.items.values(), &self.activity_head, std::iter::empty(), &HashSet::new(),
            )
        })?;
        if removed > 0 {
            self.changed();
        }
        Ok(removed)
    }

    fn collect_garbage_for<'a>(
        &'a self,
        items: impl Iterator<Item = &'a Item>,
        head: &activity::Head,
        extra_pictures: impl Iterator<Item = &'a String>,
        removed: &HashSet<&str>,
    ) -> Result<usize> {
        let mut unreferenced: HashSet<_> = self.store.ids(Table::Attachments)?
            .into_iter().map(hex::encode).collect();
        if unreferenced.is_empty() {
            return Ok(0);
        }
        for id in self.history_index.iter().filter(|(page, _)| !removed.contains(page.as_str()))
            .flat_map(|(_, versions)| versions).flat_map(|meta| &meta.picture_ids).chain(extra_pictures)
        {
            unreferenced.remove(id);
        }
        for item in items {
            if unreferenced.is_empty() {
                return Ok(0);
            }
            match item {
                Item::Page(p) => {
                    keep_attachment_refs(&mut unreferenced, &p.content);
                    keep_attachment_refs(&mut unreferenced, &p.cover);
                    keep_attachment_refs(&mut unreferenced, &p.icon);
                }
                Item::Sticky(s) => keep_attachment_refs(&mut unreferenced, &s.content),
                Item::Row(r) => {
                    keep_attachment_refs(&mut unreferenced, &r.content);
                    keep_attachment_refs(&mut unreferenced, &r.cover);
                    keep_attachment_refs(&mut unreferenced, &r.cells);
                }
                Item::Database(d) => {
                    for t in &d.templates {
                        keep_attachment_refs(&mut unreferenced, &t.content);
                        keep_attachment_refs(&mut unreferenced, &t.cells);
                    }
                }
                Item::Settings(settings) => {
                    for edit in &settings.replaced_edits {
                        keep_attachment_refs(&mut unreferenced, &edit.previous);
                    }
                }
                _ => {}
            }
        }
        if !unreferenced.is_empty() {
            for id in self.undo_attachment_ids(head)? {
                unreferenced.remove(&id);
            }
        }
        let rows: Vec<_> = unreferenced.iter().map(|id| parse_id(id)).collect::<Result<_>>()?;
        self.store.delete_many(Table::Attachments, &rows)?;
        Ok(rows.len())
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
        let (old_dek, old_id, old_format) = (&self.dek, self.vault_id, self.format);
        self.store.transaction(|s| {
            for table in [Table::Records, Table::Attachments, Table::History, Table::Activity, Table::Anchor] {
                for id in s.ids(table)? {
                    let sealed = s.get(table, &id)?.ok_or(Error::Corrupt)?;
                    let plain = crypto::open(old_dek, &aad(old_format, &old_id, table, &id), &sealed)?;
                    s.put(table, &id, &crypto::seal(&new_dek, &aad(header.format, &new_id, table, &id), &plain))?;
                }
            }
            s.write_header(&header)
        })?;
        self.dek = new_dek;
        self.vault_id = new_id;
        self.format = header.format;
        self.open = open_after;
        self.clear_query_cache();
        self.changed();
        Ok(())
    }

    /// True when the vault is in the format V1.4.0 and later write.
    pub fn is_current_format(&self) -> bool {
        self.format == Format::Current
    }

    /// Re-encrypts a vault made by V1.3.0 or earlier in the Notaya format: a fresh data key and
    /// vault id, every record and attachment re-encrypted in one transaction (all or nothing,
    /// like any other re-key), and the header rewritten with the Notaya names. The master password
    /// and keyfile are the ones that just unlocked it and stay the same (they are ignored for a vault
    /// with security off). On any error the vault stays as it was, still readable in its old format, and
    /// this runs again at the next unlock. Returns whether anything was done.
    pub fn upgrade_format(&mut self, password: &str, keyfile: Option<&[u8; 32]>) -> Result<bool> {
        if self.format == Format::Current {
            return Ok(false);
        }
        if self.open {
            self.rekey(true, |dek, id| Header::open_for(dek, id))?;
        } else {
            let kdf = self.store.read_header()?.kdf;
            self.rekey(false, |dek, id| Header::protected_for(dek, id, password, keyfile, kdf))?;
        }
        Ok(true)
    }

    /// [`Session::upgrade_format`] for the moment right after an unlock: a failure is not an error
    /// there (the notes are open and readable either way), the upgrade is simply tried again later.
    pub fn upgrade_format_quietly(&mut self, password: &str, keyfile: Option<&[u8; 32]>) -> bool {
        // After one failure the upgrade is not tried again until the program is restarted: it costs a full
        // re-encryption (and one more password stretch), and the same fault would only repeat.
        static FAILED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if FAILED.load(std::sync::atomic::Ordering::Relaxed) {
            return false;
        }
        match self.upgrade_format(password, keyfile) {
            Ok(done) => done,
            Err(e) => {
                FAILED.store(true, std::sync::atomic::Ordering::Relaxed);
                eprintln!("vault: could not upgrade the file format yet ({e}); the notes stay as they are");
                false
            }
        }
    }

    pub fn backup_to(&self, dest: &Path) -> Result<()> {
        self.store.backup_to(dest)
    }

    /// Checks that the file at `path` is a complete copy of this vault: the same header
    /// (byte for byte), the same number of records and attachments, and every one of them
    /// authenticates with this session's data key. It needs no password, and it reads one
    /// row at a time. A backup is counted as made only after this passes.
    pub fn verify_copy(&self, path: &Path) -> Result<()> {
        let copy = Store::open_read_only(path)?;
        let header = self.store.read_header()?;
        if backup::matches_source(&self.store, &copy, &header, &self.dek, false)? {
            Ok(())
        } else {
            Err(Error::Corrupt)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{new_id, now_ms, Page, Sticky};
    use rusqlite::{params, Connection};

    const FAST: KdfParams = KdfParams { m_kib: 64, t: 1, p: 1 };

    fn temp_vault() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("notaya-test-{}", new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("vault.nota")
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

    #[test]
    fn every_kind_of_change_makes_both_backup_flags_due() {
        let path = temp_vault();
        let mut s = Session::create_open(&path).unwrap();
        let p = page("flags");
        let id = p.id().to_string();
        s.put(p).unwrap();
        let settled = |s: &mut Session| {
            s.dirty = false;
            s.hourly_due = false;
        };

        settled(&mut s);
        s.put_many(vec![page("a"), page("b")]).unwrap();
        assert!(s.dirty && s.hourly_due, "put_many");

        settled(&mut s);
        s.remove_many(&[id]).unwrap();
        assert!(s.dirty && s.hourly_due, "remove_many");

        settled(&mut s);
        s.put_attachment("image/png", b"pixels").unwrap();
        assert!(s.dirty && s.hourly_due, "put_attachment");

        // The picture is not used by any page, so the cleanup removes it.
        settled(&mut s);
        assert!(s.collect_garbage().unwrap() >= 1);
        assert!(s.dirty && s.hourly_due, "collect_garbage when it removed something");

        settled(&mut s);
        assert_eq!(s.collect_garbage().unwrap(), 0);
        assert!(!s.dirty && !s.hourly_due, "a cleanup that removed nothing changes nothing");

        // Turning security on re-keys the vault: every record is new, so a backup is due.
        settled(&mut s);
        s.make_protected("a long enough password 1", None, FAST).unwrap();
        assert!(s.dirty && s.hourly_due, "make_protected");

        settled(&mut s);
        s.change_credentials("a long enough password 1", None, "another long password 2", None, FAST).unwrap();
        assert!(s.dirty && s.hourly_due, "change_credentials");
    }

    fn page(text: &str) -> Item {
        Item::Page(Page {
            source: None,
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

    #[test]
    fn person_put_many_rejects_mixed_hierarchy_cycles_atomically() {
        use crate::database::types::{Database, Row};

        let path = temp_vault();
        let mut session = Session::create_open(&path).unwrap();
        let mut first = page("First page");
        let second = page("Second page");
        let first_id = first.id().to_string();
        session.put(first.clone()).unwrap();

        let records = |s: &Session| -> HashMap<_, _> {
            s.store.all(Table::Records).unwrap().into_iter().collect()
        };
        let before = records(&session);
        let Item::Page(first_page) = &mut first else { unreachable!() };
        first_page.parent = second.id().to_string();
        let Item::Page(mut second) = second else { unreachable!() };
        second.parent.clone_from(&first_id);
        let second_id = second.id.clone();
        let attempt = session.run_as(&Actor::Person, |s| s.put_many(vec![first, Item::Page(second)]));
        assert!(matches!(attempt, Err(Error::Invalid(_))));
        assert!(matches!(session.get(&first_id), Some(Item::Page(p)) if p.parent.is_empty()));
        assert!(session.get(&second_id).is_none());
        assert_eq!(records(&session), before);

        let Item::Page(mut child) = page("Child page") else { unreachable!() };
        let mut db = Database::new(new_id(), child.section_id.clone(), "Trial database".into(), 1);
        let row = Row::new(new_id(), &db.id, "Trial row".into(), 1);
        child.parent.clone_from(&row.id);
        db.parent.clone_from(&child.id);
        let (db_id, row_id, child_id) = (db.id.clone(), row.id.clone(), child.id.clone());
        let attempt = session.run_as(&Actor::Person, |s| {
            s.put_many(vec![Item::Database(db), Item::Row(row), Item::Page(child)])
        });
        assert!(matches!(attempt, Err(Error::Invalid(_))));
        for id in [db_id, row_id, child_id] {
            assert!(session.get(&id).is_none());
        }
        assert_eq!(records(&session), before);
        drop(session);
        assert_eq!(records(&Session::unlock_open(&path).unwrap()), before);
    }

    #[test]
    fn reopened_vault_with_legacy_cycle_allows_unrelated_writes_and_repair() {
        let path = temp_vault();
        let mut session = Session::create_open(&path).unwrap();
        crate::commands::notes::create_notebook_core(
            &mut session, &Actor::Person, "Fixture notebook".into(), String::new(), String::new(),
        ).unwrap();
        let section_id = session.items().find_map(|item| match item {
            Item::Section(section) => Some(section.id.clone()),
            _ => None,
        }).unwrap();
        let fixture_page = |text| {
            let Item::Page(mut page) = page(text) else { unreachable!() };
            page.section_id.clone_from(&section_id);
            page
        };
        let mut first = fixture_page("Fixture page A");
        let mut second = fixture_page("Fixture page B");
        let mut third = fixture_page("Fixture page C");
        let second_id = second.id.clone();
        let third_id = third.id.clone();
        first.parent.clone_from(&second_id);
        second.parent.clone_from(&third_id);
        third.parent.clone_from(&first.id);

        // Seed the records as an older version could have saved them.
        session.store.transaction(|store| {
            for item in [Item::Page(first.clone()), Item::Page(second), Item::Page(third)] {
                let (id, data) = session.seal_item(&item)?;
                store.put(Table::Records, &id, &data)?;
            }
            Ok(())
        }).unwrap();
        drop(session);

        let mut session = Session::unlock_open(&path).unwrap();
        assert_eq!(session.get(&first.id).and_then(Session::parent_of), Some(second_id.as_str()));
        assert_eq!(session.get(&second_id).and_then(Session::parent_of), Some(third_id.as_str()));
        assert_eq!(session.get(&third_id).and_then(Session::parent_of), Some(first.id.as_str()));

        let unrelated = fixture_page("Fixture unrelated page");
        let unrelated_id = unrelated.id.clone();
        session.put(Item::Page(unrelated.clone())).unwrap();
        let mut settings = session.settings();
        settings.idle_lock_minutes += 1;
        let idle_lock_minutes = settings.idle_lock_minutes;
        session.put(Item::Settings(settings)).unwrap();

        let mut moved_into_cycle = unrelated.clone();
        moved_into_cycle.parent.clone_from(&first.id);
        let mut moved_within_cycle = first.clone();
        moved_within_cycle.parent.clone_from(&third_id);
        let mut new_cycle_child = fixture_page("Fixture proposed child");
        new_cycle_child.parent.clone_from(&unrelated_id);
        let new_cycle_child_id = new_cycle_child.id.clone();
        let mut new_cycle_parent = unrelated;
        new_cycle_parent.parent.clone_from(&new_cycle_child_id);

        let records = |s: &Session| -> HashMap<_, _> {
            s.store.all(Table::Records).unwrap().into_iter().collect()
        };
        let before = records(&session);
        let companion = Item::Page(fixture_page("Fixture companion page"));
        for mut changes in [
            vec![Item::Page(new_cycle_parent), Item::Page(new_cycle_child)],
            vec![Item::Page(moved_into_cycle)],
            vec![Item::Page(moved_within_cycle)],
        ] {
            changes.insert(0, companion.clone());
            assert!(matches!(session.put_many(changes), Err(Error::Invalid(_))));
            assert!(session.get(companion.id()).is_none());
            assert!(session.get(&new_cycle_child_id).is_none());
            assert_eq!(session.get(&unrelated_id).and_then(Session::parent_of), Some(section_id.as_str()));
            assert_eq!(session.get(&first.id).and_then(Session::parent_of), Some(second_id.as_str()));
            assert_eq!(records(&session), before);
        }

        first.parent.clear();
        let first_id = first.id.clone();
        session.put(Item::Page(first)).unwrap();
        drop(session);

        let session = Session::unlock_open(&path).unwrap();
        assert_eq!(session.get(&first_id).and_then(Session::parent_of), Some(section_id.as_str()));
        assert!(session.is_within(&second_id, &section_id));
        assert!(matches!(session.get(&unrelated_id), Some(Item::Page(page)) if page.parent.is_empty()));
        assert_eq!(session.settings().idle_lock_minutes, idle_lock_minutes);
    }

    fn sticky(text: &str) -> Item {
        Item::Sticky(Sticky { trashed_at: None,
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
        let old_copy = path.with_file_name("old-copy.nota");
        let mut s = Session::create(&path, "old-password-1", None, FAST).unwrap();
        s.put(page("before")).unwrap();
        s.backup_to(&old_copy).unwrap(); // an automatic backup lying next to the vault
        let old_header = Session::read_header(&old_copy).unwrap();
        s.change_credentials("old-password-1", None, "new-password-2", None, FAST).unwrap();
        s.put(page("written after the change")).unwrap();
        drop(s);

        // Put the OLD header into a copy of the CURRENT file.
        let franken = path.with_file_name("franken.nota");
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
        let good = path.with_file_name("good.nota");
        s.backup_to(&good).unwrap();
        s.verify_copy(&good).unwrap();

        // A zero-byte file, a missing file and some other vault are all refused.
        let empty = path.with_file_name("empty.nota");
        std::fs::write(&empty, b"").unwrap();
        assert!(s.verify_copy(&empty).is_err());
        assert!(s.verify_copy(&path.with_file_name("missing.nota")).is_err());
        let other = temp_vault();
        drop(Session::create(&other, "pw", None, FAST).unwrap());
        assert!(s.verify_copy(&other).is_err());

        // One flipped byte in a record, and in an attachment.
        for table in ["records", "attachments"] {
            let bad = path.with_file_name(format!("bad-{table}.nota"));
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
        let short = path.with_file_name("short.nota");
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
        let backup = path.with_file_name("backup.nota");
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
        let old_copy = path.with_file_name("old-open-copy.nota");
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

        let protected_copy = path.with_file_name("protected-copy.nota");
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

    fn header_magic(path: &std::path::Path) -> Vec<u8> {
        Session::read_header(path).unwrap().to_bytes()[0..4].to_vec()
    }

    #[test]
    fn an_open_vault_from_v1_3_0_is_upgraded_in_place_and_keeps_everything() {
        let path = temp_vault();
        let (p, att, id_before);
        {
            let mut s = Session::create_legacy_open(&path).unwrap();
            let one = page("old note");
            p = one.id().to_string();
            s.put(one).unwrap();
            s.put(sticky("old sticky")).unwrap();
            att = s.put_attachment("image/png", b"pixels").unwrap();
            id_before = Session::read_header(&path).unwrap().vault_id;
        }
        assert_eq!(header_magic(&path), b"CLF1");
        let mut s = Session::unlock_open(&path).unwrap();
        assert!(!s.is_current_format());
        s.dirty = false;
        s.hourly_due = false;
        assert!(s.upgrade_format("", None).unwrap());
        assert!(s.is_current_format() && s.dirty && s.hourly_due, "a backup of the upgraded file is due");
        assert!(!s.upgrade_format("", None).unwrap(), "a second call does nothing");
        // The session keeps working in the new format.
        s.put(page("new note")).unwrap();
        drop(s);

        assert_eq!(header_magic(&path), b"NOTA");
        let h = Session::read_header(&path).unwrap();
        assert!(h.open);
        assert_ne!(h.vault_id, id_before, "the upgraded vault has a fresh data key and id");
        let s = Session::unlock_open(&path).unwrap();
        assert!(s.is_current_format());
        assert!(matches!(s.get(&p), Some(Item::Page(pg)) if pg.text == "old note"));
        assert_eq!(s.items().filter(|i| matches!(i, Item::Page(_))).count(), 2);
        assert_eq!(s.items().filter(|i| matches!(i, Item::Sticky(_))).count(), 1);
        assert_eq!(s.get_attachment(&att).unwrap().1, b"pixels".to_vec());
        s.verify_attachments().unwrap();
        // No row of the old format is left: the plain text of the file does not show the old words either.
        let raw = std::fs::read(&path).unwrap();
        assert!(!raw.windows(8).any(|w| w == b"old note"));
    }

    #[test]
    fn a_protected_vault_from_v1_3_0_is_upgraded_with_its_password_and_keyfile() {
        let path = temp_vault();
        let kf = crypto::random_bytes::<32>();
        let p;
        {
            let mut s = Session::create_legacy(&path, "Correct-Horse-Battery-9", Some(&kf), FAST).unwrap();
            let one = page("private");
            p = one.id().to_string();
            s.put(one).unwrap();
            s.put_attachment("image/png", b"pixels").unwrap();
        }
        let kdf_before = Session::read_header(&path).unwrap().kdf;
        let mut s = Session::unlock(&path, "Correct-Horse-Battery-9", Some(&kf)).unwrap();
        assert!(s.upgrade_format("Correct-Horse-Battery-9", Some(&kf)).unwrap());
        drop(s);

        assert_eq!(header_magic(&path), b"NOTA");
        assert_eq!(Session::read_header(&path).unwrap().kdf, kdf_before, "the password settings stay");
        let s = Session::unlock(&path, "Correct-Horse-Battery-9", Some(&kf)).unwrap();
        assert!(matches!(s.get(&p), Some(Item::Page(pg)) if pg.text == "private"));
        s.verify_attachments().unwrap();
        drop(s);
        assert!(matches!(Session::unlock(&path, "Correct-Horse-Battery-9", None), Err(Error::BadCredentials)));
        assert!(matches!(Session::unlock(&path, "wrong password", Some(&kf)), Err(Error::BadCredentials)));
    }

    #[test]
    fn a_legacy_session_saves_in_the_old_format_until_it_is_upgraded() {
        let path = temp_vault();
        drop(Session::create_legacy_open(&path).unwrap());
        let mut s = Session::unlock_open(&path).unwrap();
        let added = page("added before the upgrade");
        let id = added.id().to_string();
        s.put(added).unwrap();
        drop(s);
        assert_eq!(header_magic(&path), b"CLF1");
        let s = Session::unlock_open(&path).unwrap();
        assert!(s.get(&id).is_some());
    }

    #[test]
    fn a_failed_upgrade_changes_nothing_and_the_notes_stay_readable() {
        let path = temp_vault();
        {
            let mut s = Session::create_legacy_open(&path).unwrap();
            s.put_attachment("image/png", b"pixels").unwrap();
            s.put(page("keep")).unwrap();
        }
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute("UPDATE attachments SET data = x'00010203'", []).unwrap();
        }
        let before = std::fs::read(&path).unwrap();
        let mut s = Session::unlock_open(&path).unwrap();
        assert!(s.upgrade_format("", None).is_err());
        assert!(!s.upgrade_format_quietly("", None), "the quiet form reports it as not done");
        assert!(!s.is_current_format());
        drop(s);
        assert_eq!(header_magic(&path), b"CLF1");
        assert_eq!(std::fs::read(&path).unwrap(), before, "the file is byte for byte as it was");
        let s = Session::unlock_open(&path).unwrap();
        assert_eq!(s.items().filter(|i| matches!(i, Item::Page(_))).count(), 1);
    }

    #[test]
    fn changing_credentials_or_security_also_upgrades_an_old_vault() {
        let path = temp_vault();
        drop(Session::create_legacy(&path, "Correct-Horse-Battery-9", None, FAST).unwrap());
        let mut s = Session::unlock(&path, "Correct-Horse-Battery-9", None).unwrap();
        s.change_credentials("Correct-Horse-Battery-9", None, "Another-Long-Password-1", None, FAST).unwrap();
        assert!(s.is_current_format());
        drop(s);
        assert_eq!(header_magic(&path), b"NOTA");
        assert!(Session::unlock(&path, "Another-Long-Password-1", None).is_ok());

        let path = temp_vault();
        drop(Session::create_legacy_open(&path).unwrap());
        let mut s = Session::unlock_open(&path).unwrap();
        s.make_protected("Correct-Horse-Battery-9", None, FAST).unwrap();
        drop(s);
        assert_eq!(header_magic(&path), b"NOTA");
        assert!(Session::unlock(&path, "Correct-Horse-Battery-9", None).is_ok());
    }

    #[test]
    fn a_copy_of_an_old_vault_still_verifies_against_its_session() {
        let path = temp_vault();
        let mut s = Session::create_legacy_open(&path).unwrap();
        s.put(page("copy me")).unwrap();
        let copy = path.with_file_name("legacy-copy.nota");
        s.backup_to(&copy).unwrap();
        s.verify_copy(&copy).unwrap();
        s.upgrade_format("", None).unwrap();
        assert!(s.verify_copy(&copy).is_err(), "the old copy is not a copy of the upgraded vault");
        let upgraded = path.with_file_name("upgraded-copy.nota");
        s.backup_to(&upgraded).unwrap();
        s.verify_copy(&upgraded).unwrap();
    }

    // Real files made by the code of V1.3.0 (the vault format did not change between V1.3.0 and the commit
    // they were made at), kept as test data in `test-data/v1.3.0`. They prove that vaults written before
    // the Notaya format open and upgrade, not just vaults the current code made to look old.
    const FIXTURE_OPEN: &[u8] = include_bytes!("../../test-data/v1.3.0/open.vault");
    const FIXTURE_PROTECTED: &[u8] = include_bytes!("../../test-data/v1.3.0/protected.vault");
    const FIXTURE_KEYFILE: &[u8] = include_bytes!("../../test-data/v1.3.0/keyfile.vault");
    const PAGE_ID: &str = "00000000000000000000000000000001";

    fn fixture(bytes: &[u8]) -> std::path::PathBuf {
        let path = temp_vault();
        std::fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn a_vault_made_by_v1_3_0_opens_and_upgrades() {
        let path = fixture(FIXTURE_OPEN);
        assert_eq!(header_magic(&path), b"CLF1");
        let mut s = Session::unlock_any(&path, "", None).unwrap();
        assert!(matches!(s.get(PAGE_ID), Some(Item::Page(p)) if p.text == "open fixture page"));
        assert_eq!(s.get_attachment("4af92f06f769c38a27e8123570ed4ab3").unwrap().1, b"open-pixels".to_vec());
        assert!(s.upgrade_format("", None).unwrap());
        drop(s);
        assert_eq!(header_magic(&path), b"NOTA");
        let s = Session::unlock_any(&path, "", None).unwrap();
        assert!(matches!(s.get(PAGE_ID), Some(Item::Page(p)) if p.text == "open fixture page"));
        assert_eq!(s.items().count(), 3, "settings, page and sticky");
        assert_eq!(s.get_attachment("4af92f06f769c38a27e8123570ed4ab3").unwrap().1, b"open-pixels".to_vec());
    }

    #[test]
    fn a_protected_vault_made_by_v1_3_0_opens_and_upgrades_with_its_password() {
        let path = fixture(FIXTURE_PROTECTED);
        assert!(matches!(Session::unlock(&path, "Wrong-Fixture-Pass", None), Err(Error::BadCredentials)));
        let mut s = Session::unlock(&path, "Fixture-Pass-1", None).unwrap();
        assert!(matches!(s.get(PAGE_ID), Some(Item::Page(p)) if p.text == "protected fixture page"));
        assert!(s.upgrade_format("Fixture-Pass-1", None).unwrap());
        drop(s);
        assert_eq!(header_magic(&path), b"NOTA");
        let s = Session::unlock(&path, "Fixture-Pass-1", None).unwrap();
        assert_eq!(s.get_attachment("96eb9b45f4e2f18bd93eab13c15910ed").unwrap().1, b"protected-pixels".to_vec());
        assert!(matches!(Session::unlock(&path, "Wrong-Fixture-Pass", None), Err(Error::BadCredentials)));
    }

    #[test]
    fn a_keyfile_vault_made_by_v1_3_0_upgrades_and_still_needs_its_keyfile() {
        let path = fixture(FIXTURE_KEYFILE);
        let kf = [7u8; 32];
        assert!(Session::unlock(&path, "Fixture-Pass-2", None).is_err());
        let mut s = Session::unlock(&path, "Fixture-Pass-2", Some(&kf)).unwrap();
        assert!(s.upgrade_format("Fixture-Pass-2", Some(&kf)).unwrap());
        drop(s);
        assert!(Session::unlock(&path, "Fixture-Pass-2", None).is_err());
        let s = Session::unlock(&path, "Fixture-Pass-2", Some(&kf)).unwrap();
        assert!(matches!(s.get(PAGE_ID), Some(Item::Page(p)) if p.text == "keyfile fixture page"));
    }

    #[test]
    fn query_cache_clears_across_writes_rekey_rollback_and_clear() {
        use crate::database::types::{Database, View, ViewKind};

        let path = temp_vault();
        let mut s = Session::create_open(&path).unwrap();
        let db_id = new_id();
        let view_id = new_id();
        let mut db = Database::new(db_id.clone(), new_id(), "Cache DB".into(), 0);
        db.views = vec![View { id: view_id.clone(), name: "Table".into(), kind: ViewKind::Table, ..View::default() }];
        s.put(Item::Database(db.clone())).unwrap();

        let seed_cache = |session: &Session| -> String {
            let mut cache = session.query_cache.borrow_mut();
            let entry = cache.query(session.items(), &db, &view_id, db.views[0].clone(), (1000, 0));
            entry.metadata(&db, "").generation
        };

        let gen1 = seed_cache(&s);
        let p = page("new-page");
        let pid = p.id().to_string();
        s.put_many(vec![p]).unwrap();
        assert!(s.query_cache.borrow_mut().get(&db_id, &view_id, &gen1).is_none(), "put_many must clear query_cache");

        let gen2 = seed_cache(&s);
        s.remove_many(&[pid]).unwrap();
        assert!(s.query_cache.borrow_mut().get(&db_id, &view_id, &gen2).is_none(), "remove_many must clear query_cache");

        let gen3 = seed_cache(&s);
        s.make_protected("Secret-Pass-1", None, FAST).unwrap();
        assert!(s.query_cache.borrow_mut().get(&db_id, &view_id, &gen3).is_none(), "rekey must clear query_cache");

        let gen4 = seed_cache(&s);
        let err = s.atomic_action(false, |sess| {
            sess.put(page("rolled-back"))?;
            Err::<(), Error>(Error::invalid("forced rollback"))
        });
        assert!(err.is_err());
        assert!(s.query_cache.borrow_mut().get(&db_id, &view_id, &gen4).is_none(), "atomic_action rollback must clear query_cache");

        let gen5 = seed_cache(&s);
        s.clear_query_cache();
        assert!(s.query_cache.borrow_mut().get(&db_id, &view_id, &gen5).is_none(), "clear_query_cache must invalidate");
    }
}
