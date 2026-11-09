//! One AEAD anchor commits the activity head and a SHA-256 membership ledger.
//! The ledger uses the existing hash-chain construction, not a commutative hash
//! accumulator: replaying its authenticated add/remove deltas defines the exact
//! expected history set. Saves append O(1) records per snapshot; only unlock and
//! explicit verification replay the small, content-free certificates.
use super::{activity::{canonical, Head}, history::VersionMeta, parse_id, RowId, Session, Store, Table};
use crate::{error::{Error, Result}, model::{Item, SETTINGS_ID}};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use zeroize::{Zeroize, ZeroizeOnDrop};

pub(super) const ID: RowId = [0; 16];
const ZERO: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub(super) struct Anchor {
    pub activity: Head,
    pub activity_rows: u64,
    tail: String,
    root: String,
    certificates: u64,
    members: u64,
}
impl Anchor {
    pub(super) fn certificates(&self) -> u64 { self.certificates }
}
impl Default for Anchor {
    fn default() -> Self {
        Self { activity: Head::default(), activity_rows: 0, tail: String::new(), root: ZERO.into(), certificates: 0, members: 0 }
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct Binding {
    page: String,
    body_hash: String,
}
pub(super) type Delta = (String, Option<Binding>);
pub(super) fn addition(meta: &VersionMeta) -> Delta {
    (meta.version_id.clone(), Some(Binding { page: meta.page_id.clone(), body_hash: meta.body_hash.clone() }))
}

#[derive(Serialize, Deserialize)]
struct Certificate {
    previous: String,
    previous_hash: String,
    sequence: u64,
    changes: Vec<Delta>,
}
fn hash(certificate: &Certificate) -> Result<String> {
    let mut hash = Sha256::new();
    hash.update(b"notaya/history-membership/v1\0");
    hash.update(&*canonical(certificate)?);
    Ok(hex::encode(hash.finalize()))
}

pub(super) struct Update {
    pub next: Anchor,
    rows: Vec<(RowId, Vec<u8>)>,
    sealed: Vec<u8>,
}
impl Update {
    pub fn write(&self, store: &Store) -> Result<()> {
        for (id, bytes) in &self.rows { store.put(Table::Anchor, id, bytes)?; }
        store.put(Table::Anchor, &ID, &self.sealed)
    }
}

impl Session {
    pub(super) fn refresh_anchor_head(&self, mut anchor: Anchor, head: &Head, store: &Store) -> Result<Anchor> {
        anchor.activity = head.clone();
        anchor.activity_rows = head.rows()?;
        store.put(Table::Anchor, &ID, &self.seal_json(Table::Anchor, &ID, &anchor)?)?;
        Ok(anchor)
    }
    pub(super) fn prepare_anchor(&self, activity: &Head, changes: Vec<Delta>) -> Result<Update> {
        let mut next = self.anchor.clone();
        next.activity = activity.clone();
        next.activity_rows = activity.rows()?;
        let mut rows = vec![];
        if !changes.is_empty() {
            for (_, binding) in &changes {
                next.members = if binding.is_some() { next.members.checked_add(1) } else { next.members.checked_sub(1) }.ok_or(Error::Corrupt)?;
            }
            let mut id = crate::crypto::random_bytes::<16>();
            id[0] = 2; // 0 is the anchor; 2 certificates; 3 undo bodies.
            let certificate = Certificate {
                previous: next.tail.clone(), previous_hash: next.root.clone(),
                sequence: next.certificates.checked_add(1).ok_or(Error::Corrupt)?, changes,
            };
            next.tail = hex::encode(id);
            next.root = hash(&certificate)?;
            next.certificates = certificate.sequence;
            rows.push((id, self.seal_json(Table::Anchor, &id, &certificate)?));
        }
        let sealed = self.seal_json(Table::Anchor, &ID, &next)?;
        Ok(Update { next, rows, sealed })
    }

    /// Never trust absence alone as a migration signal. The latch is inside the
    /// ordinary authenticated settings item, which 1.5.1 can read and ignore.
    pub(super) fn load_anchor(&mut self) -> Result<()> {
        let Some(Item::Settings(settings)) = self.get(SETTINGS_ID) else { return Err(Error::Corrupt); };
        let required = settings.journals_anchored;
        let stored: Option<Anchor> = self.read_json(Table::Anchor, &ID)?;
        if let Some(anchor) = stored {
            self.anchor = anchor;
            self.check_anchor()?;
            return Ok(());
        }
        if required || !self.store.ids(Table::Anchor)?.is_empty() {
            return Err(Error::invalid("The journal anchor is missing. The log or history may have been changed."));
        }
        // Genuine legacy: fully verify the surviving old state before blessing it.
        self.verify_activity_chain()?;
        self.verify_history_bodies()?;
        let update = self.prepare_anchor(&self.activity_head,
            self.history_index.values().flatten().map(addition).collect())?;
        let mut settings = self.settings();
        settings.journals_anchored = true;
        let item = Item::Settings(settings);
        let (id, bytes) = self.seal_item(&item)?;
        self.store.transaction(|s| {
            update.write(s)?;
            s.put(Table::Records, &id, &bytes)
        })?;
        self.anchor = update.next;
        self.items.insert(SETTINGS_ID.into(), item);
        self.changed();
        Ok(())
    }

    pub(super) fn check_anchor(&self) -> Result<()> {
        let disk: Anchor = self.read_json(Table::Anchor, &ID)?.ok_or_else(|| Error::invalid("The journal anchor is missing. The log or history may have been changed."))?;
        if canonical(&disk)?.as_slice() != canonical(&self.anchor)?.as_slice()
            || canonical(&disk.activity)?.as_slice() != canonical(&self.activity_head)?.as_slice()
            || disk.activity_rows != self.activity_head.rows()?
        { return Err(Error::Corrupt); }
        if self.store.ids(Table::Activity)?.len() as u64 != disk.activity_rows { return Err(Error::Corrupt); }
        let mut tail = disk.tail.clone();
        let mut root = disk.root.clone();
        let mut certificates = vec![];
        for sequence in (1..=disk.certificates).rev() {
            let id = parse_id(&tail).map_err(|_| Error::Corrupt)?;
            if id[0] != 2 { return Err(Error::Corrupt); }
            let certificate: Certificate = self.read_json(Table::Anchor, &id)?.ok_or(Error::Corrupt)?;
            if certificate.sequence != sequence || hash(&certificate)? != root { return Err(Error::Corrupt); }
            tail = certificate.previous.clone();
            root = certificate.previous_hash.clone();
            certificates.push(certificate);
        }
        if !tail.is_empty() || root != ZERO { return Err(Error::Corrupt); }
        let mut expected = BTreeMap::new();
        for certificate in certificates.into_iter().rev() {
            for (id, binding) in certificate.changes {
                match binding {
                    Some(binding) => { if expected.insert(id, binding).is_some() { return Err(Error::Corrupt); } }
                    None => { if expected.remove(&id).is_none() { return Err(Error::Corrupt); } }
                }
            }
        }
        // Re-read metadata, rather than accepting the same possibly stale RAM
        // index that supplied the expected count. Bodies remain lazy at unlock.
        let mut actual = BTreeMap::new();
        for (id, bytes) in self.store.history_meta()? {
            let meta: VersionMeta = self.open_json(Table::History, &id, &bytes)?;
            if meta.version_id != hex::encode(id) || id[0] != 0 { return Err(Error::Corrupt); }
            let (id, binding) = addition(&meta);
            if actual.insert(id, binding.ok_or(Error::Corrupt)?).is_some() { return Err(Error::Corrupt); }
        }
        if expected != actual || expected.len() as u64 != disk.members { return Err(Error::Corrupt); }
        // Check existence of BOTH halves using ids only; never fetch a body here.
        let mut ids = self.store.ids(Table::History)?;
        ids.sort();
        let mut expected_ids = Vec::with_capacity(expected.len() * 2);
        for id in expected.keys() {
            expected_ids.push(parse_id(id).map_err(|_| Error::Corrupt)?);
            expected_ids.push(super::history::body_id(id).map_err(|_| Error::Corrupt)?);
        }
        expected_ids.sort();
        if ids != expected_ids { return Err(Error::Corrupt); }
        Ok(())
    }
}
