//! Encrypted activity and undo. Activity entries stay lazy at unlock; the head
//! is compared with the separate anchor. Hashes cover canonical JSON and undo
//! references. Permanent deletion replaces undo with content-free tombstones
//! and re-links the chain in the deletion transaction.
use super::{
    history::{Snapshot, VersionBody, DAY},
    RowId, Session, Store, Table,
};
use crate::{
    actions::Actor,
    error::{Error, Result},
    model::{new_id, now_ms, Item},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

const HEAD: RowId = [0; 16];
const ZERO: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const MAX_ENTRIES: u64 = 100_000;

/// Journals display trusted pairing labels; content-free change signals use
/// `changes::by` instead and never carry these labels.
pub(super) fn actor_label(actor: &Actor) -> String {
    match actor {
        Actor::Person => "person".into(),
        Actor::Automation => "automation".into(),
        Actor::Ai { label, .. } => format!("ai:{label}"),
    }
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub(super) struct Head {
    last_seq: u64,
    last_hash: String,
    last_id: String,
    first_seq: u64,
    base_hash: String,
}

impl Default for Head {
    fn default() -> Self {
        Self {
            last_seq: 0,
            last_hash: ZERO.into(),
            last_id: String::new(),
            first_seq: 1,
            base_hash: ZERO.into(),
        }
    }
}
impl Head {
    pub(super) fn rows(&self) -> Result<u64> {
        if self.first_seq == 0 || self.first_seq > self.last_seq.saturating_add(1) { return Err(Error::Corrupt); }
        Ok(self.last_seq.saturating_add(1) - self.first_seq + u64::from(self.last_seq > 0))
    }
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct ActivityItem {
    pub id: String,
    pub kind: String,
    pub title_at_the_time: String,
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct ActivityMeta {
    pub seq: u64,
    pub at: i64,
    pub actor: String,
    pub change_set: String,
    pub action: String,
    pub items: Vec<ActivityItem>,
    #[zeroize(skip)]
    pub counts: BTreeMap<String, u64>,
    pub undoable: bool,
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
struct Previous {
    id: String,
    // Page bodies live only in history. Permanent deletion therefore removes
    // them from undo too; activity retains titles but no second page body.
    before: Option<String>,
    version: Option<String>,
    after_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    body_hash: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    purged: bool,
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
struct Entry {
    id: String,
    prev_id: String,
    meta: ActivityMeta,
    owner: Option<String>,
    undo: Vec<Previous>,
    prev_hash: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    hash: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoSummary {
    pub restored: usize,
    pub skipped: usize,
}

pub(super) struct Journal {
    snapshots: Vec<Snapshot>,
    activity: Option<(RowId, Vec<u8>, Vec<u8>, Head)>,
    bursts: Vec<(String, i64)>,
    bodies: Vec<(RowId, Vec<u8>)>,
    pub(super) anchor: super::anchor::Update,
}
impl Journal {
    pub(super) fn history_additions(&self) -> impl Iterator<Item = super::anchor::Delta> + '_ {
        self.snapshots.iter().map(|snapshot| super::anchor::addition(&snapshot.meta))
    }
    pub(super) fn picture_ids(&self) -> impl Iterator<Item = &String> {
        self.snapshots.iter().flat_map(|snapshot| &snapshot.meta.picture_ids)
    }
    pub(super) fn head<'a>(&'a self, current: &'a Head) -> &'a Head {
        self.activity.as_ref().map(|(_, _, _, head)| head).unwrap_or(current)
    }
    pub(super) fn write(&self, store: &Store) -> Result<()> {
        for (id, bytes) in &self.bodies { store.put(Table::Anchor, id, bytes)?; }
        for snapshot in &self.snapshots {
            for (id, bytes) in &snapshot.rows {
                store.put(Table::History, id, bytes)?;
            }
        }
        if let Some((id, entry, head, _)) = &self.activity {
            store.put(Table::Activity, id, entry)?;
            store.put(Table::Activity, &HEAD, head)?;
        }
        self.anchor.write(store)
    }
}

pub(super) fn canonical<T: Serialize>(value: &T) -> Result<Zeroizing<Vec<u8>>> {
    fn sort(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                let mut pairs: Vec<_> = std::mem::take(map).into_iter().collect();
                pairs.sort_by(|a, b| a.0.cmp(&b.0));
                for (key, mut value) in pairs {
                    sort(&mut value);
                    map.insert(key, value);
                }
            }
            serde_json::Value::Array(values) => {
                for value in values {
                    sort(value);
                }
            }
            _ => {}
        }
    }
    fn wipe(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::String(text) => text.zeroize(),
            serde_json::Value::Object(map) => {
                for (mut key, mut value) in std::mem::take(map) {
                    key.zeroize();
                    wipe(&mut value);
                }
            }
            serde_json::Value::Array(values) => {
                for value in values {
                    wipe(value);
                }
            }
            _ => {}
        }
    }
    let mut value = serde_json::to_value(value).map_err(|_| Error::Corrupt)?;
    sort(&mut value);
    let bytes = serde_json::to_vec(&value)
        .map(Zeroizing::new)
        .map_err(|_| Error::Corrupt);
    wipe(&mut value);
    bytes
}

fn item_hash(item: &Item) -> Result<String> {
    Ok(hex::encode(Sha256::digest(&*canonical(item)?)))
}
fn entry_hash(entry: &Entry) -> Result<String> {
    let mut unsigned = entry.clone();
    unsigned.hash.clear();
    let mut hash = Sha256::new();
    hash.update(hex::decode(&entry.prev_hash).map_err(|_| Error::Corrupt)?);
    hash.update(&*canonical(&unsigned)?);
    Ok(hex::encode(hash.finalize()))
}

fn description(item: &Item) -> ActivityItem {
    let (kind, title) = match item {
        Item::Notebook(x) => ("notebook", x.name.as_str()),
        Item::Section(x) => ("section", x.name.as_str()),
        Item::Page(x) => ("page", x.title.as_str()),
        Item::Row(x) => ("row", x.title.as_str()),
        Item::Database(x) => ("database", x.title.as_str()),
        Item::Sticky(_) => ("sticky", "Sticky note"),
        Item::Settings(_) => ("settings", "Settings"),
    };
    ActivityItem {
        id: item.id().into(),
        kind: kind.into(),
        title_at_the_time: title.into(),
    }
}

fn page_body_only(old: &Item, new: &Item) -> bool {
    let mut old = old.clone();
    if let Some(body) = VersionBody::from_item(new) {
        if body.apply(&mut old).is_err() {
            return false;
        }
        // updated timestamps are not structural changes.
        match (&mut old, new) {
            (Item::Page(a), Item::Page(b)) => a.updated = b.updated,
            (Item::Row(a), Item::Row(b)) => a.updated = b.updated,
            _ => return false,
        }
        return matches!((item_hash(&old), item_hash(new)), (Ok(a), Ok(b)) if a == b);
    }
    false
}

impl Session {
    pub(super) fn maintain_journals(&mut self) -> Result<()> {
        let at = now_ms();
        // The first settings write creates the initial anchor in its own data
        // transaction. There is no existing journal to maintain yet.
        if self.items.is_empty() { self.journal_maintenance_at = at; return Ok(()); }
        if at.saturating_sub(self.journal_maintenance_at) >= DAY
            || self
                .activity_head
                .last_seq
                .saturating_sub(self.activity_head.first_seq)
                >= MAX_ENTRIES - 1
        {
            self.thin_history(at)?;
            self.prune_activity(at)?;
            self.journal_maintenance_at = at;
        }
        Ok(())
    }
    pub(super) fn load_activity_head(&mut self) -> Result<()> {
        let head: Option<Head> = self.read_json(Table::Activity, &HEAD)?;
        if head.is_none() && !self.store.ids(Table::Activity)?.is_empty() {
            return Err(Error::Corrupt);
        }
        self.activity_head = head.unwrap_or_default();
        Ok(())
    }

    fn prepare_entry(
        &self,
        entry: &mut Entry,
        mut head: Head,
    ) -> Result<(RowId, Vec<u8>, Vec<u8>, Head)> {
        entry.meta.seq = head.last_seq.checked_add(1).ok_or(Error::Corrupt)?;
        let mut id = crate::crypto::random_bytes::<16>();
        if id == HEAD {
            id[0] = 1;
        }
        entry.id = hex::encode(id);
        entry.prev_id = head.last_id.clone();
        entry.prev_hash = head.last_hash.clone();
        entry.hash = entry_hash(entry)?;
        head.last_seq = entry.meta.seq;
        head.last_hash = entry.hash.clone();
        head.last_id = entry.id.clone();
        Ok((
            id,
            self.seal_json(Table::Activity, &id, entry)?,
            self.seal_json(Table::Activity, &HEAD, &head)?,
            head,
        ))
    }

    pub(super) fn prepare_journal(
        &self,
        actor: &Actor,
        items: &[Item],
        removed: &[String],
    ) -> Result<Journal> {
        let at = now_ms();
        let origin = self.request_ai.as_ref().unwrap_or(actor);
        let mut journal = Journal {
            snapshots: vec![],
            activity: None,
            bursts: vec![],
            bodies: vec![],
            anchor: self.prepare_anchor(&self.activity_head, vec![])?,
        };
        let mut undo = vec![];
        let mut described = vec![];
        let mut counts = BTreeMap::new();
        for (id, new) in items
            .iter()
            .map(|i| (i.id(), Some(i)))
            .chain(removed.iter().map(|id| (id.as_str(), None)))
        {
            let old = self.get(id);
            if old.is_none() && new.is_none() {
                continue;
            }
            let ai = matches!(origin, Actor::Ai { .. });
            let mut version = None;
            if let Some(old) = old {
                if new.is_some()
                    && (ai
                        || (new.is_some_and(|new| Self::page_changed(Some(old), new))
                            && self.history_due(id, at)))
                {
                    if let Some(snapshot) = self.prepare_snapshot(old, actor, ai, at)? {
                        version = Some(snapshot.meta.version_id.clone());
                        journal.snapshots.push(snapshot);
                    }
                }
            }
            if let Some(new) = new {
                if ai || old.is_none() {
                    if let Some(snapshot) = self.prepare_snapshot(new, actor, false, at)? {
                        journal.snapshots.push(snapshot);
                    }
                }
            }
            // Person page saves are represented by the first line in a ten-minute
            // editing burst. Structural changes always get a new line.
            let body_only = old
                .zip(new)
                .is_some_and(|(old, new)| page_body_only(old, new));
            if matches!(actor, Actor::Person) && body_only {
                if self
                    .person_bursts
                    .get(id)
                    .is_some_and(|last| at.saturating_sub(*last) < 10 * super::history::MINUTE)
                {
                    continue;
                }
                journal.bursts.push((id.into(), at));
            }
            let action = match (old, new) {
                (None, _) => "created",
                (_, None) => "deleted",
                (Some(a), Some(b))
                    if Self::own_trash(a).is_none() && Self::own_trash(b).is_some() =>
                {
                    "binned"
                }
                (Some(a), Some(b))
                    if Self::own_trash(a).is_some() && Self::own_trash(b).is_none() =>
                {
                    "restored"
                }
                _ => "changed",
            };
            *counts.entry(action.into()).or_insert(0) += 1;
            described.push(description(new.or(old).ok_or(Error::Corrupt)?));
            if ai && new.is_some() {
                let before = old
                    .map(|item| {
                        let mut item = item.clone();
                        match &mut item {
                            Item::Page(p) => {
                                p.content.clear();
                                p.text.clear();
                            }
                            Item::Row(p) => {
                                p.content.clear();
                                p.text.clear();
                            }
                            Item::Settings(settings) => {
                                settings.replaced_edits.clear();
                            }
                            _ => {}
                        }
                        serde_json::to_string(&item).map_err(|_| Error::Corrupt)
                    })
                    .transpose()?.map(Zeroizing::new);
                // All non-page values (sticky bodies, row cells, templates)
                // live behind a purgeable encrypted reference, never inline in
                // the immutable activity entry. Page bodies already use history.
                let mut body = None;
                let mut body_hash = None;
                if let Some(before) = &before {
                    let mut row = crate::crypto::random_bytes::<16>();
                    row[0] = 3;
                    body = Some(hex::encode(row));
                    body_hash = Some(hex::encode(Sha256::digest(before.as_bytes())));
                    journal.bodies.push((row, self.seal_json(Table::Anchor, &row, &**before)?));
                }
                undo.push(Previous {
                    id: id.into(),
                    before: None,
                    version,
                    after_hash: new.map(item_hash).transpose()?,
                    body, body_hash, purged: false,
                });
            }
        }
        if !described.is_empty() {
            let mut entry = Entry {
                id: String::new(),
                prev_id: String::new(),
                meta: ActivityMeta {
                    seq: 0,
                    at,
                    actor: actor_label(actor),
                    change_set: self.change_set.clone().unwrap_or_else(new_id),
                    action: self.action_name.unwrap_or("items.changed").into(),
                    items: described,
                    counts,
                    undoable: !undo.is_empty(),
                },
                owner: match origin {
                    Actor::Ai { pairing_id, .. } => Some(pairing_id.clone()),
                    _ => None,
                },
                undo,
                prev_hash: String::new(),
                hash: String::new(),
            };
            journal.activity = Some(self.prepare_entry(&mut entry, self.activity_head.clone())?);
        }
        let head = journal.activity.as_ref().map(|(_, _, _, head)| head).unwrap_or(&self.activity_head);
        journal.anchor = self.prepare_anchor(head, journal.snapshots.iter().map(|s| super::anchor::addition(&s.meta)).collect())?;
        Ok(journal)
    }

    pub(super) fn accept_journal(&mut self, journal: Journal) {
        self.anchor = journal.anchor.next;
        for snapshot in journal.snapshots {
            self.history_index
                .entry(snapshot.meta.page_id.clone())
                .or_default()
                .push(snapshot.meta);
        }
        if let Some((_, _, _, head)) = journal.activity {
            self.activity_head = head;
        }
        self.person_bursts.extend(journal.bursts);
    }

    /// Random row ids and encrypted backwards links conceal sequence/order on
    /// disk while keeping newest-first pagination independent of log size.
    fn walk_entries(&self, mut visit: impl FnMut(&Entry) -> Result<bool>) -> Result<()> {
        self.walk_entries_at(&self.activity_head, &mut visit)
    }
    fn walk_entries_at(&self, head: &Head, mut visit: impl FnMut(&Entry) -> Result<bool>) -> Result<()> {
        let mut id = head.last_id.clone();
        for seq in (head.first_seq..=head.last_seq).rev() {
            let row = super::parse_id(&id).map_err(|_| Error::Corrupt)?;
            let entry: Entry = self
                .read_json(Table::Activity, &row)?
                .ok_or(Error::Corrupt)?;
            if entry.meta.seq != seq || entry.id != id {
                return Err(Error::Corrupt);
            }
            id = entry.prev_id.clone();
            if !visit(&entry)? {
                break;
            }
        }
        Ok(())
    }

    fn previous_body(&self, previous: &Previous) -> Result<Option<Zeroizing<String>>> {
        if previous.purged { return Ok(None); }
        if let Some(id) = &previous.body {
            let row = super::parse_id(id).map_err(|_| Error::Corrupt)?;
            if row[0] != 3 || previous.before.is_some() { return Err(Error::Corrupt); }
            let body: String = self.read_json(Table::Anchor, &row)?.ok_or(Error::Corrupt)?;
            let body = Zeroizing::new(body);
            if previous.body_hash.as_ref() != Some(&hex::encode(Sha256::digest(body.as_bytes()))) { return Err(Error::Corrupt); }
            return Ok(Some(body));
        }
        if previous.body_hash.is_some() { return Err(Error::Corrupt); }
        Ok(previous.before.clone().map(Zeroizing::new))
    }

    fn undo_body_rows(&self, entries: &[Entry]) -> Result<Vec<RowId>> {
        entries.iter().flat_map(|e| &e.undo).filter_map(|p| p.body.as_ref())
            .map(|id| super::parse_id(id).map_err(|_| Error::Corrupt)).collect()
    }

    fn verify_undo_bodies(&self) -> Result<()> {
        let mut expected = HashSet::new();
        self.walk_entries(|entry| {
            for previous in &entry.undo {
                if previous.purged && (previous.before.is_some() || previous.body.is_some() || previous.version.is_some()) { return Err(Error::Corrupt); }
                if let Some(body) = self.previous_body(previous)? {
                    let item: Item = serde_json::from_str(&body).map_err(|_| Error::Corrupt)?;
                    if item.id() != previous.id { return Err(Error::Corrupt); }
                }
                if let Some(id) = &previous.body {
                    if !expected.insert(super::parse_id(id).map_err(|_| Error::Corrupt)?) { return Err(Error::Corrupt); }
                }
            }
            Ok(true)
        })?;
        let ids = self.store.ids(Table::Anchor)?;
        let actual: HashSet<_> = ids.iter().copied().filter(|id| id[0] == 3).collect();
        if expected != actual || ids.len() as u64 != 1 + self.anchor.certificates() + actual.len() as u64 { return Err(Error::Corrupt); }
        Ok(())
    }

    pub(super) fn verify_legacy_undo(&self) -> Result<()> {
        self.walk_entries(|entry| {
            for previous in &entry.undo {
                if let Some(body) = self.previous_body(previous)? {
                    let item: Item = serde_json::from_str(&body).map_err(|_| Error::Corrupt)?;
                    if item.id() != previous.id { return Err(Error::Corrupt); }
                }
            }
            Ok(true)
        })
    }

    /// Called INSIDE the same transaction as the permanent deletion. Removes
    /// both new indirect bodies and legacy inline values, then re-links the
    /// affected suffix. Titles/actors/times/actions and row ids are retained.
    pub(super) fn purge_undo_in_store(&self, head: &Head, ids: &HashSet<&str>, templates: &BTreeMap<String, HashSet<String>>) -> Result<Head> {
        let mut entries = vec![];
        self.walk_entries_at(head, |e| { entries.push(e.clone()); Ok(true) })?;
        entries.reverse();
        let mut previous_hash = head.base_hash.clone();
        let mut bodies = Vec::new();
        for entry in &mut entries {
            let old_hash = entry.hash.clone();
            for previous in &mut entry.undo {
                let mut purge = ids.contains(previous.id.as_str());
                if let Some(removed) = templates.get(&previous.id) {
                    if let Some(body) = self.previous_body(previous)? {
                        let item: Item = serde_json::from_str(&body).map_err(|_| Error::Corrupt)?;
                        if let Item::Database(db) = item { purge |= db.templates.iter().any(|t| removed.contains(&t.id)); }
                    }
                }
                if purge {
                    if let Some(body) = &previous.body { bodies.push(super::parse_id(body)?); }
                    previous.before.zeroize();
                    previous.body = None;
                    previous.body_hash = None;
                    previous.version = None;
                    previous.after_hash = None;
                    previous.purged = true;
                }
            }
            entry.meta.undoable = entry.undo.iter().any(|p| !p.purged);
            entry.prev_hash = previous_hash;
            entry.hash = entry_hash(entry)?;
            previous_hash = entry.hash.clone();
            if entry.hash != old_hash {
                let id = super::parse_id(&entry.id)?;
                self.store.put(Table::Activity, &id, &self.seal_json(Table::Activity, &id, entry)?)?;
            }
        }
        self.store.delete_many(Table::Anchor, &bodies)?;
        let mut next = head.clone();
        next.last_hash = previous_hash;
        self.store.put(Table::Activity, &HEAD, &self.seal_json(Table::Activity, &HEAD, &next)?)?;
        Ok(next)
    }

    fn entries(&self) -> Result<Vec<Entry>> {
        let mut out = vec![];
        self.walk_entries(|entry| {
            out.push(entry.clone());
            Ok(true)
        })?;
        out.reverse();
        Ok(out)
    }

    pub fn activity_list(
        &self,
        actor: Option<&str>,
        before: Option<u64>,
        limit: usize,
    ) -> Result<Vec<ActivityMeta>> {
        let mut out = vec![];
        self.walk_entries(|entry| {
            if before.is_none_or(|end| entry.meta.seq < end)
                && actor.is_none_or(|who| entry.meta.actor == who)
            {
                out.push(entry.meta.clone());
            }
            Ok(out.len() < limit.clamp(1, 500))
        })?;
        Ok(out)
    }

    pub fn activity_verify(&self) -> Result<()> {
        self.check_anchor()?;
        self.verify_activity_chain()?;
        self.verify_history_bodies()?;
        self.verify_undo_bodies()
    }

    pub(super) fn verify_activity_chain(&self) -> Result<()> {
        let head: Head = self.read_json(Table::Activity, &HEAD)?.unwrap_or_default();
        if canonical(&head)?.as_slice() != canonical(&self.activity_head)?.as_slice() {
            return Err(Error::Corrupt);
        }
        if head.first_seq == 0 || head.first_seq > head.last_seq.saturating_add(1) {
            return Err(Error::Corrupt);
        }
        let mut expected = head.last_hash.clone();
        let mut count = 0;
        self.walk_entries(|entry| {
            if entry.hash != expected || entry.hash != entry_hash(entry)? {
                return Err(Error::Corrupt);
            }
            expected = entry.prev_hash.clone();
            count += 1;
            Ok(true)
        })?;
        if expected != head.base_hash
            || self.store.ids(Table::Activity)?.len() != count + usize::from(head.last_seq > 0)
        {
            return Err(Error::Corrupt);
        }
        Ok(())
    }

    /// Non-page previous values can contain pictures and file cells too. Read
    /// them lazily during cleanup, never into the unlock-time index.
    pub(super) fn undo_attachment_ids(&self, head: &Head) -> Result<HashSet<String>> {
        let mut referenced = HashSet::new();
        self.walk_entries_at(head, |entry| {
            for previous in &entry.undo {
                if self.get(&previous.id).is_none() {
                    continue;
                }
                if let Some(version) = &previous.version {
                    // GC can run inside the data transaction, before its new
                    // snapshots have been accepted into the RAM index.
                    let row = super::parse_id(version).map_err(|_| Error::Corrupt)?;
                    if self.store.get(Table::History, &row)?.is_none() {
                        continue;
                    }
                }
                if let Some(before) = self.previous_body(previous)? {
                    for token in before
                        .split(|c: char| !c.is_ascii_hexdigit())
                        .filter(|s| s.len() == 32)
                    {
                        referenced.insert(token.to_ascii_lowercase());
                    }
                }
            }
            Ok(true)
        })?;
        Ok(referenced)
    }

    pub(super) fn verify_journals(&self) -> Result<()> {
        self.activity_verify()
    }

    pub(super) fn verify_history_bodies(&self) -> Result<()> {
        for versions in self.history_index.values() {
            for meta in versions {
                self.history_get(&meta.page_id, &meta.version_id)?;
            }
        }
        if self.store.ids(Table::History)?.len()
            != self
                .history_index
                .values()
                .map(|v| v.len() * 2)
                .sum::<usize>()
        {
            return Err(Error::Corrupt);
        }
        Ok(())
    }

    /// The trusted dispatcher records one summary on connection close, not one
    /// entry per read. No action exposes arbitrary log writing to the pipe.
    pub fn record_ai_reads(&mut self, actor: &Actor, count: u64) -> Result<()> {
        if !matches!(actor, Actor::Ai { .. }) {
            return Err(Error::invalid("An AI connection is required."));
        }
        self.maintain_journals()?;
        self.append_marker(actor, "ai.reads", BTreeMap::from([("reads".into(), count)]))
    }

    pub fn record_automation_run(&mut self) -> Result<()> {
        self.maintain_journals()?;
        self.append_marker(
            &Actor::Automation,
            "automation.run",
            BTreeMap::from([("runs".into(), 1)]),
        )
    }

    fn append_marker(
        &mut self,
        actor: &Actor,
        action: &str,
        counts: BTreeMap<String, u64>,
    ) -> Result<()> {
        let mut entry = Entry {
            id: String::new(),
            prev_id: String::new(),
            meta: ActivityMeta {
                seq: 0,
                at: now_ms(),
                actor: actor_label(actor),
                change_set: self.change_set.clone().unwrap_or_else(new_id),
                action: action.into(),
                items: vec![],
                counts,
                undoable: false,
            },
            owner: None,
            undo: vec![],
            prev_hash: String::new(),
            hash: String::new(),
        };
        let (id, bytes, head, next) = self.prepare_entry(&mut entry, self.activity_head.clone())?;
        let anchor = self.prepare_anchor(&next, vec![])?;
        self.store.transaction(|s| {
            s.put(Table::Activity, &id, &bytes)?;
            s.put(Table::Activity, &HEAD, &head)?;
            anchor.write(s)
        })?;
        self.anchor = anchor.next;
        self.activity_head = next;
        self.changed();
        Ok(())
    }

    pub(super) fn prepare_thinning(&self, removed: &[String]) -> Result<Journal> {
        let mut entry = Entry {
            id: String::new(), prev_id: String::new(), owner: None, undo: vec![],
            prev_hash: String::new(), hash: String::new(),
            meta: ActivityMeta {
                seq: 0, at: now_ms(), actor: "person".into(), change_set: new_id(),
                action: "history.thinned".into(), items: vec![],
                counts: BTreeMap::from([("thinned".into(), removed.len() as u64)]), undoable: false,
            },
        };
        let activity = self.prepare_entry(&mut entry, self.activity_head.clone())?;
        let anchor = self.prepare_anchor(&activity.3, removed.iter().map(|id| (id.clone(), None)).collect())?;
        Ok(Journal { snapshots: vec![], activity: Some(activity), bursts: vec![], bodies: vec![], anchor })
    }

    pub fn clear_activity(&mut self, actor: &Actor, confirmation: &str) -> Result<()> {
        if !matches!(actor, Actor::Person) || confirmation != "CLEAR" {
            return Err(Error::invalid("Type CLEAR to clear the activity log."));
        }
        self.activity_verify()?;
        let mut entry = Entry {
            id: String::new(),
            prev_id: String::new(),
            meta: ActivityMeta {
                seq: 0,
                at: now_ms(),
                actor: "person".into(),
                change_set: new_id(),
                action: "log.cleared".into(),
                items: vec![],
                counts: BTreeMap::new(),
                undoable: false,
            },
            owner: None,
            undo: vec![],
            prev_hash: String::new(),
            hash: String::new(),
        };
        let (id, bytes, head, next) = self.prepare_entry(&mut entry, Head::default())?;
        let anchor = self.prepare_anchor(&next, vec![])?;
        let bodies = self.undo_body_rows(&self.entries()?)?;
        self.store.transaction(|s| {
            for id in &bodies { s.delete(Table::Anchor, id)?; }
            for id in s.ids(Table::Activity)? {
                s.delete(Table::Activity, &id)?;
            }
            s.put(Table::Activity, &id, &bytes)?;
            s.put(Table::Activity, &HEAD, &head)?;
            anchor.write(s)
        })?;
        self.anchor = anchor.next;
        self.activity_head = next;
        self.changed();
        Ok(())
    }

    pub fn prune_activity(&mut self, at: i64) -> Result<usize> {
        self.check_anchor()?;
        self.verify_activity_chain()?;
        let entries = self.entries()?;
        let remove = entries
            .iter()
            .take_while(|e| {
                e.meta.at < at - 365 * DAY
                    || self.activity_head.last_seq.saturating_sub(e.meta.seq) >= MAX_ENTRIES - 2
            })
            .count();
        if remove == 0 {
            return Ok(0);
        }
        let mut head = self.activity_head.clone();
        head.first_seq = entries[remove - 1].meta.seq + 1;
        head.base_hash = entries[remove - 1].hash.clone();
        let mut entry = Entry {
            id: String::new(),
            prev_id: String::new(),
            meta: ActivityMeta {
                seq: 0,
                at,
                actor: "person".into(),
                change_set: new_id(),
                action: "log.pruned".into(),
                items: vec![],
                counts: BTreeMap::from([("pruned".into(), remove as u64)]),
                undoable: false,
            },
            owner: None,
            undo: vec![],
            prev_hash: String::new(),
            hash: String::new(),
        };
        let (id, bytes, sealed_head, next) = self.prepare_entry(&mut entry, head)?;
        let anchor = self.prepare_anchor(&next, vec![])?;
        let bodies = self.undo_body_rows(&entries[..remove])?;
        self.store.transaction(|s| {
            for id in &bodies { s.delete(Table::Anchor, id)?; }
            for e in &entries[..remove] {
                s.delete(Table::Activity, &super::parse_id(&e.id)?)?;
            }
            s.put(Table::Activity, &id, &bytes)?;
            s.put(Table::Activity, &HEAD, &sealed_head)?;
            anchor.write(s)
        })?;
        self.anchor = anchor.next;
        self.activity_head = next;
        self.changed();
        Ok(remove)
    }

    pub fn activity_undo(&mut self, actor: &Actor, change_set: &str) -> Result<UndoSummary> {
        let (restore, skipped) = self.activity_undo_plan(actor, change_set)?;
        self.ai_write_targets(actor, &restore)?;
        let restored = restore.len();
        self.run_as(actor, |s| s.put_many(restore))?;
        Ok(UndoSummary { restored, skipped })
    }

    /// Returns all surviving item IDs referenced in the given change set without
    /// applying the UI's 500-entry list clamp.
    pub(crate) fn activity_change_set_targets(&self, change_set: &str) -> Result<Vec<String>> {
        self.activity_verify()?;
        let entries = self.entries()?;
        let mut out = Vec::new();
        for e in entries.iter().filter(|e| e.meta.change_set == change_set) {
            for item in &e.meta.items {
                if self.get(&item.id).is_some() {
                    out.push(item.id.clone());
                }
            }
        }
        Ok(out)
    }

    pub(crate) fn activity_undo_plan(&self, actor: &Actor, change_set: &str) -> Result<(Vec<Item>, usize)> {
        self.activity_verify()?;
        let entries = self.entries()?;
        let selected: Vec<_> = entries
            .iter()
            .filter(|e| e.meta.change_set == change_set)
            .collect();
        if selected.is_empty() {
            return Err(Error::NotFound);
        }
        if selected.iter().any(|entry| entry.undo.iter().any(|previous| previous.purged)) {
            return Err(Error::invalid("This change includes permanently deleted content. It cannot be undone."));
        }
        if !selected.iter().any(|e| e.owner.is_some())
            || selected.iter().filter(|e| !e.undo.is_empty()).any(|e| {
                e.owner.is_none()
                    || match actor {
                        Actor::Person => false,
                        Actor::Ai { pairing_id, .. } => e.owner.as_ref() != Some(pairing_id),
                        _ => true,
                    }
            })
        {
            return Err(Error::invalid("Only the AI's own changes can be undone."));
        }
        let mut previous: BTreeMap<String, Previous> = BTreeMap::new();
        for entry in &selected {
            for undo in &entry.undo {
                if let Some(prior) = previous.get_mut(&undo.id) {
                    prior.after_hash = undo.after_hash.clone();
                } else {
                    previous.insert(undo.id.clone(), undo.clone());
                }
            }
        }
        let last_seq = selected.last().ok_or(Error::NotFound)?.meta.seq;
        let touched: HashSet<_> = entries
            .iter()
            .filter(|e| e.meta.seq > last_seq && e.meta.actor == "person")
            .flat_map(|e| e.meta.items.iter().map(|i| i.id.as_str()))
            .collect();
        let mut restore = vec![];
        let mut skipped = 0;
        for (id, undo) in previous {
            let Some(current) = self.get(&id) else {
                skipped += 1;
                continue;
            };
            if touched.iter().any(|other| self.is_within(other, &id))
                || undo.after_hash.as_ref() != Some(&item_hash(current)?)
                || self.check_write(actor, &id, None).is_err()
            {
                skipped += 1;
                continue;
            }
            let mut item = if let Some(before) = self.previous_body(&undo)? {
                serde_json::from_str::<Item>(&before).map_err(|_| Error::Corrupt)?
            } else {
                let mut item = current.clone();
                match &mut item {
                    Item::Notebook(x) => x.trashed_at = Some(now_ms()),
                    Item::Section(x) => x.trashed_at = Some(now_ms()),
                    Item::Page(x) => x.trashed_at = Some(now_ms()),
                    Item::Row(x) => x.trashed_at = Some(now_ms()),
                    Item::Database(x) => x.trashed_at = Some(now_ms()),
                    Item::Sticky(x) => {
                        x.trashed_at = Some(now_ms());
                        x.open = false;
                    }
                    Item::Settings(_) => {
                        skipped += 1;
                        continue;
                    }
                }
                item
            };
            if let Some(version) = &undo.version {
                match self.history_get(&id, version) {
                    Ok(body) => body.apply(&mut item)?,
                    Err(Error::NotFound) => {
                        skipped += 1;
                        continue;
                    }
                    Err(e) => return Err(e),
                }
            }
            if let (Item::Settings(before), Item::Settings(current)) = (&item, current) {
                // Undo can never restore AI permissions, security, confirmations,
                // or future owner-only fields through an old settings snapshot.
                let mut settings = current.clone();
                settings.theme = before.theme.clone();
                settings.accent = before.accent.clone();
                settings.font = before.font.clone();
                settings.font_size = before.font_size;
                settings.editor_width = before.editor_width.clone();
                settings.density = before.density.clone();
                settings.backdrop = before.backdrop.clone();
                settings.unlock_background = before.unlock_background.clone();
                settings.sticky_default_color = before.sticky_default_color.clone();
                item = Item::Settings(settings);
            }
            restore.push(item);
        }
        Ok((restore, skipped))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KdfParams;
    use std::path::PathBuf;
    pub(super) const FAST: KdfParams = KdfParams {
        m_kib: 64,
        t: 1,
        p: 1,
    };
    pub(super) fn fixture() -> (Session, PathBuf) {
        let dir = PathBuf::from(
            std::env::var_os("NOTAYA_HOME").expect("tests require a throwaway NOTAYA_HOME"),
        )
        .join(format!("c3-{}", new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("vault.nota");
        (
            Session::create(&path, "throwaway", None, FAST).unwrap(),
            path,
        )
    }
    fn ai(id: &str) -> Actor {
        Actor::Ai {
            pairing_id: id.into(),
            label: "Test AI".into(),
        }
    }
    fn page(text: &str) -> Item {
        Item::Page(serde_json::from_value(serde_json::json!({"id": new_id(), "sectionId":"", "title":"Test page", "content":text, "text":text, "order":0, "favorite":false, "trashedAt":null, "created":1, "updated":1})).unwrap())
    }
    fn edit(s: &mut Session, id: &str, text: &str, actor: &Actor) {
        let Item::Page(mut p) = s.get(id).unwrap().clone() else {
            panic!()
        };
        p.content = text.into();
        p.text = text.into();
        p.updated += 1;
        s.run_as(actor, |s| s.put(Item::Page(p))).unwrap();
    }

    #[test]
    fn thousand_entries_verify_and_reads_are_one_connection_summary() {
        let (mut s, _) = fixture();
        for _ in 0..1000 {
            s.record_ai_reads(&ai("one"), 37).unwrap();
        }
        s.activity_verify().unwrap();
        let list = s.activity_list(Some("ai:Test AI"), None, 20).unwrap();
        assert_eq!(list.len(), 20);
        assert_eq!(list[0].counts["reads"], 37);
        assert!(list.windows(2).all(|pair| pair[0].seq > pair[1].seq));
        assert!(s.record_ai_reads(&Actor::Person, 1).is_err());
    }

    #[test]
    fn removed_swapped_edited_and_truncated_entries_or_heads_are_detected() {
        for mode in 0..7 {
            let (mut s, _) = fixture();
            for _ in 0..4 {
                s.record_ai_reads(&ai("one"), 1).unwrap();
            }
            let rows: BTreeMap<_, _> = s
                .entries()
                .unwrap()
                .iter()
                .map(|e| (e.meta.seq, super::super::parse_id(&e.id).unwrap()))
                .collect();
            match mode {
                0 => s.store.delete(Table::Activity, &rows[&2]).unwrap(),
                1 => {
                    let a = s.store.get(Table::Activity, &rows[&2]).unwrap().unwrap();
                    let b = s.store.get(Table::Activity, &rows[&3]).unwrap().unwrap();
                    s.store.put(Table::Activity, &rows[&2], &b).unwrap();
                    s.store.put(Table::Activity, &rows[&3], &a).unwrap();
                }
                2 => {
                    let mut e: Entry = s.read_json(Table::Activity, &rows[&2]).unwrap().unwrap();
                    e.meta.at += 1;
                    let sealed = s.seal_json(Table::Activity, &rows[&2], &e).unwrap();
                    s.store.put(Table::Activity, &rows[&2], &sealed).unwrap();
                }
                3 => s
                    .store
                    .delete(Table::Activity, &rows[&s.activity_head.last_seq])
                    .unwrap(),
                4 => {
                    let mut head = s.activity_head.clone();
                    head.last_hash = ZERO.into();
                    let sealed = s.seal_json(Table::Activity, &HEAD, &head).unwrap();
                    s.store.put(Table::Activity, &HEAD, &sealed).unwrap();
                }
                5 => s.store.delete(Table::Activity, &HEAD).unwrap(),
                _ => s.store.put(Table::Activity, &rows[&3], b"damaged").unwrap(),
            }
            assert!(s.activity_verify().is_err(), "tamper mode {mode}");
        }
    }

    #[test]
    fn pruning_and_person_only_typed_clear_leave_valid_anchors() {
        let (mut s, _) = fixture();
        for _ in 0..4 {
            s.record_ai_reads(&ai("one"), 1).unwrap();
        }
        assert!(s.prune_activity(now_ms() + 366 * DAY).unwrap() >= 5);
        s.activity_verify().unwrap();
        assert_eq!(
            s.activity_list(None, None, 10).unwrap()[0].action,
            "log.pruned"
        );
        assert!(s.clear_activity(&ai("one"), "CLEAR").is_err());
        assert!(s.clear_activity(&Actor::Person, "clear").is_err());
        s.clear_activity(&Actor::Person, "CLEAR").unwrap();
        assert_eq!(s.activity_list(None, None, 10).unwrap().len(), 1);
        assert_eq!(
            s.activity_list(None, None, 10).unwrap()[0].action,
            "log.cleared"
        );
        s.record_ai_reads(&ai("one"), 1).unwrap();
        s.activity_verify().unwrap();
    }

    #[test]
    fn mixed_request_undo_restores_before_values_and_bins_created_items() {
        let (mut s, _) = fixture();
        let original = page("before body");
        let id = original.id().to_string();
        s.put(original).unwrap();
        let binned = page("bin body");
        let bin_id = binned.id().to_string();
        s.put(binned).unwrap();
        let row_id = new_id();
        let db_id = new_id();
        s.put(Item::Database(crate::database::types::Database::new(db_id.clone(), String::new(), "Test DB".into(), 1))).unwrap();
        s.put(Item::Row(crate::database::types::Row::new(
            row_id.clone(),
            &db_id,
            "Row".into(),
            1,
        )))
        .unwrap();
        let created = page("AI created");
        let created_id = created.id().to_string();
        s.run_as(&ai("one"), |s| {
            edit(s, &id, "AI body", &ai("one"));
            let Item::Page(mut p) = s.get(&bin_id).unwrap().clone() else {
                panic!()
            };
            p.trashed_at = Some(now_ms());
            let Item::Row(mut row) = s.get(&row_id).unwrap().clone() else {
                panic!()
            };
            row.cells = "{\"property\":42}".into();
            let mut settings = s.settings();
            settings.theme = "black".into();
            s.put_many(vec![
                created,
                Item::Page(p),
                Item::Row(row),
                Item::Settings(settings),
            ])
        })
        .unwrap();
        let change_set = s.activity_list(Some("ai:Test AI"), None, 1).unwrap()[0]
            .change_set
            .clone();
        assert!(s.activity_undo(&ai("other"), &change_set).is_err());
        let summary = s.activity_undo(&Actor::Person, &change_set).unwrap();
        assert_eq!((summary.restored, summary.skipped), (5, 0));
        assert!(matches!(s.get(&id), Some(Item::Page(p)) if p.text == "before body"));
        assert!(!s.in_bin(&bin_id));
        assert!(s.in_bin(&created_id));
        assert!(matches!(s.get(&row_id), Some(Item::Row(row)) if row.cells == "{}"));
        assert_eq!(s.settings().theme, "sunrise");
        s.activity_verify().unwrap();
    }

    #[test]
    fn undo_skips_later_person_changes_even_after_reopen() {
        let (mut s, path) = fixture();
        let p = page("before");
        let id = p.id().to_string();
        s.put(p).unwrap();
        edit(&mut s, &id, "AI", &ai("one"));
        let set = s.activity_list(Some("ai:Test AI"), None, 1).unwrap()[0]
            .change_set
            .clone();
        edit(&mut s, &id, "person", &Actor::Person);
        drop(s);
        let mut s = Session::unlock(&path, "throwaway", None).unwrap();
        let result = s.activity_undo(&ai("one"), &set).unwrap();
        assert_eq!((result.restored, result.skipped), (0, 1));
        assert!(matches!(s.get(&id), Some(Item::Page(p)) if p.text == "person"));
        let person = s.activity_list(Some("person"), None, 1).unwrap()[0]
            .change_set
            .clone();
        assert!(s.activity_undo(&ai("one"), &person).is_err());
    }

    #[test]
    fn undo_keeps_non_page_pictures_and_never_changes_owner_security_fields() {
        let (mut s, _) = fixture();
        let picture = s.put_attachment("image/png", b"throwaway picture").unwrap();
        let id = new_id();
        s.put(serde_json::from_value(serde_json::json!({"kind":"sticky","id":id,"content":format!("http://clatt.localhost/{picture}"),"text":"","color":"yellow","x":0,"y":0,"w":320,"h":320,"onTop":true,"open":false,"created":1,"updated":1})).unwrap()).unwrap();
        s.run_action(&ai("one"), "batch.run", |s| {
            let Item::Sticky(mut sticky) = s.get(&id).unwrap().clone() else {
                panic!()
            };
            sticky.content.clear();
            let mut settings = s.settings();
            settings.theme = "black".into();
            // Even a trusted caller incorrectly supplying a security change
            // cannot make undo lower security through the settings snapshot.
            settings.idle_lock_minutes = 30;
            s.put_many(vec![Item::Sticky(sticky), Item::Settings(settings)])
        })
        .unwrap();
        let entry = s
            .activity_list(Some("ai:Test AI"), None, 1)
            .unwrap()
            .remove(0);
        assert_eq!(entry.action, "batch.run");
        s.collect_garbage().unwrap();
        assert!(s.get_attachment(&picture).is_ok());
        let summary = s.activity_undo(&ai("one"), &entry.change_set).unwrap();
        assert_eq!(summary.restored, 2);
        assert!(
            matches!(s.get(&id), Some(Item::Sticky(sticky)) if sticky.content.contains(&picture))
        );
        assert_eq!(s.settings().theme, "sunrise");
        assert_eq!(s.settings().idle_lock_minutes, 30);
    }

    #[test]
    fn automation_writes_in_an_ai_request_share_undo_and_change_set() {
        let (mut s, _) = fixture();
        let original = page("before");
        let id = original.id().to_string();
        s.put(original).unwrap();
        s.run_action(&ai("one"), "batch.run", |s| {
            s.record_automation_run()?;
            s.run_as(&Actor::Automation, |s| {
                let Item::Page(mut page) = s.get(&id).unwrap().clone() else {
                    panic!()
                };
                page.text = "automatic".into();
                s.put(Item::Page(page))
            })
        })
        .unwrap();
        let entry = s
            .activity_list(Some("automation"), None, 1)
            .unwrap()
            .remove(0);
        assert!(entry.undoable);
        assert_eq!(
            s.activity_undo(&ai("one"), &entry.change_set)
                .unwrap()
                .restored,
            1
        );
        assert!(matches!(s.get(&id), Some(Item::Page(page)) if page.text == "before"));
    }

    #[test]
    fn journal_failure_rolls_back_records_history_revisions_and_memory() {
        let (mut s, path) = fixture();
        let p = page("before");
        let id = p.id().to_string();
        s.put(p).unwrap();
        let before = s.history_list(&id).unwrap().len();
        let revision = s.rev(&id);
        rusqlite::Connection::open(&path).unwrap().execute_batch("CREATE TRIGGER block_journal BEFORE INSERT ON activity BEGIN SELECT RAISE(ABORT, 'fixture'); END;").unwrap();
        let Item::Page(mut p) = s.get(&id).unwrap().clone() else {
            panic!()
        };
        p.text = "must not survive".into();
        assert!(s.run_as(&ai("one"), |s| s.put(Item::Page(p))).is_err());
        assert_eq!(s.rev(&id), revision);
        assert_eq!(s.history_list(&id).unwrap().len(), before);
        drop(s);
        let s = Session::unlock(&path, "throwaway", None).unwrap();
        assert!(matches!(s.get(&id), Some(Item::Page(p)) if p.text == "before"));
        s.activity_verify().unwrap();
    }

    #[test]
    fn failed_or_panicking_requests_cannot_leak_actor_identity_into_the_next_write() {
        let (mut s, _) = fixture();
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _: Result<()> = s.run_action(&ai("one"), "batch.run", |_| panic!("fixture panic"));
        }));
        assert!(panic.is_err());
        assert!(
            s.actor.is_none()
                && s.request_ai.is_none()
                && s.change_set.is_none()
                && s.action_name.is_none()
        );
        s.put(page("person")).unwrap();
        let entry = s.activity_list(None, None, 1).unwrap().remove(0);
        assert_eq!(entry.actor, "person");
        assert!(!entry.undoable);
        let denied: Result<()> = s.run_action(&ai("one"), "page.write", |_| Err(Error::NotFound));
        assert!(denied.is_err());
        assert!(
            s.actor.is_none()
                && s.request_ai.is_none()
                && s.change_set.is_none()
                && s.action_name.is_none()
        );
    }

    #[test]
    fn rekey_security_switches_password_change_format_upgrade_and_backup_cover_journals() {
        for legacy in [false, true] {
            let (session, path) = fixture();
            drop(session);
            let path = if legacy {
                path.with_file_name("legacy.clf")
            } else {
                path
            };
            let mut s = if legacy {
                Session::create_legacy(&path, "throwaway", None, FAST).unwrap()
            } else {
                Session::unlock(&path, "throwaway", None).unwrap()
            };
            let p = page("before");
            let id = p.id().to_string();
            s.put(p).unwrap();
            edit(&mut s, &id, "after AI", &ai("one"));
            let version = s
                .history_list(&id)
                .unwrap()
                .into_iter()
                .find(|v| v.before_ai)
                .unwrap()
                .version_id
                .clone();
            let check = |s: &Session| {
                s.activity_verify().unwrap();
                assert_eq!(s.history_get(&id, &version).unwrap().text, "before");
            };
            if legacy {
                assert!(s.upgrade_format("throwaway", None).unwrap());
                check(&s);
            }
            s.make_open("throwaway", None).unwrap();
            check(&s);
            drop(s);
            let mut s = Session::unlock_open(&path).unwrap();
            check(&s);
            s.make_protected("first password", None, FAST).unwrap();
            check(&s);
            s.change_credentials("first password", None, "second password", None, FAST)
                .unwrap();
            check(&s);
            drop(s);
            let s = Session::unlock(&path, "second password", None).unwrap();
            check(&s);
            let copy = path.with_file_name("copy.nota");
            s.backup_to(&copy).unwrap();
            s.verify_copy(&copy).unwrap();
            let copy_session = Session::unlock(&copy, "second password", None).unwrap();
            check(&copy_session);
            copy_session.verify_attachments().unwrap();
            drop(copy_session);
            rusqlite::Connection::open(&copy)
                .unwrap()
                .execute(
                    "DELETE FROM history WHERE id=?1",
                    rusqlite::params![&super::super::history::body_id(&version).unwrap()[..]],
                )
                .unwrap();
            assert!(s.verify_copy(&copy).is_err());
        }
    }

    #[test]
    fn corrupt_journal_rolls_back_rekey_of_all_earlier_tables() {
        let (mut s, _) = fixture();
        let p = page("original");
        let id = p.id().to_string();
        s.put(p).unwrap();
        let row = super::super::parse_id(&s.entries().unwrap()[0].id).unwrap();
        let activity = s.store.get(Table::Activity, &row).unwrap().unwrap();
        let header = s.store.read_header().unwrap().to_bytes();
        let records = s.store.all(Table::Records).unwrap();
        let history = s.store.all(Table::History).unwrap();
        s.store.put(Table::Activity, &row, b"damaged").unwrap();
        assert!(s.make_open("throwaway", None).is_err());
        assert_eq!(header, s.store.read_header().unwrap().to_bytes());
        assert_eq!(records, s.store.all(Table::Records).unwrap());
        assert_eq!(history, s.store.all(Table::History).unwrap());
        s.store.put(Table::Activity, &row, &activity).unwrap();
        s.activity_verify().unwrap();
        s.verify_journals().unwrap();
    }

    #[test]
    fn legacy_inline_sticky_undo_is_purged_without_losing_activity_metadata() {
        let (mut s, path) = fixture();
        let marker = Zeroizing::new(format!("legacy{}", new_id()));
        let id = new_id();
        s.put(serde_json::from_value(serde_json::json!({"kind":"sticky","id":id,"content":&*marker,"text":&*marker,"color":"yellow","x":0,"y":0,"w":320,"h":320,"onTop":false,"open":false,"created":1,"updated":1})).unwrap()).unwrap();
        let mut changed = s.get(&id).unwrap().clone();
        if let Item::Sticky(p) = &mut changed { p.content = "after".into(); p.text = p.content.clone(); }
        s.run_as(&ai("one"), |s| s.put(changed)).unwrap();
        let mut entry = s.entries().unwrap().last().unwrap().clone();
        let body = s.previous_body(&entry.undo[0]).unwrap().unwrap();
        let body_row = super::super::parse_id(entry.undo[0].body.as_ref().unwrap()).unwrap();
        // Synthesize the exact old inline encoding, with a legitimate hash/head.
        entry.undo[0].before = Some(body.to_string());
        entry.undo[0].body = None; entry.undo[0].body_hash = None;
        let legacy = serde_json::to_value(&entry).unwrap();
        let fields = legacy["undo"][0].as_object().unwrap();
        assert_eq!(fields.len(), 4, "old entry hashes must not gain default fields");
        assert!(fields.contains_key("before") && fields.contains_key("version")
            && fields.contains_key("id") && fields.contains_key("after_hash"));
        entry.hash = entry_hash(&entry).unwrap();
        let mut head = s.activity_head.clone(); head.last_hash = entry.hash.clone();
        let anchor = s.store.transaction(|store| {
            store.delete(Table::Anchor, &body_row)?;
            let row = super::super::parse_id(&entry.id)?;
            store.put(Table::Activity, &row, &s.seal_json(Table::Activity, &row, &entry)?)?;
            store.put(Table::Activity, &HEAD, &s.seal_json(Table::Activity, &HEAD, &head)?)?;
            s.refresh_anchor_head(s.anchor.clone(), &head, store)
        }).unwrap();
        s.activity_head = head; s.anchor = anchor; s.activity_verify().unwrap();
        // An actual pre-SF3 file has no latch field or anchor table. Keep the
        // inline ciphertext/hash just constructed and exercise silent migration.
        let mut settings = serde_json::to_value(Item::Settings(s.settings())).unwrap();
        settings.as_object_mut().unwrap().remove("journalsAnchored");
        let settings_id = super::super::parse_id(crate::model::SETTINGS_ID).unwrap();
        s.store.put(Table::Records, &settings_id,
            &s.seal_json(Table::Records, &settings_id, &settings).unwrap()).unwrap();
        drop(s);
        rusqlite::Connection::open(&path).unwrap().execute_batch("DROP TABLE journal_anchor").unwrap();
        let mut s = Session::unlock(&path, "throwaway", None).unwrap();
        assert!(s.settings().journals_anchored);
        crate::commands::stickies::delete_sticky_core(&mut s, &Actor::Person, id.clone()).unwrap();
        crate::commands::notes::test_delete_page_core(&mut s, &Actor::Person, id.clone()).unwrap();
        for table in [Table::Records, Table::Attachments, Table::History, Table::Activity, Table::Anchor] {
            for (row, bytes) in s.store.all(table).unwrap() {
                let plain = crate::crypto::open(&s.dek, &super::super::aad(s.format, &s.vault_id, table, &row), &bytes).unwrap();
                assert!(!plain.windows(marker.len()).any(|w| w == marker.as_bytes()));
            }
        }
        assert!(s.activity_list(None, None, 500).unwrap().iter().any(|e| e.items.iter().any(|i| i.id == id)));
        s.activity_verify().unwrap();
    }
}
