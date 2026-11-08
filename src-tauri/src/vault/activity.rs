//! Encrypted, append-only activity and undo. Only the encrypted head is loaded
//! at unlock. Entry hashes cover canonical JSON, including encrypted undo data.
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
}
impl Journal {
    pub(super) fn write(&self, store: &Store) -> Result<()> {
        for snapshot in &self.snapshots {
            for (id, bytes) in &snapshot.rows {
                store.put(Table::History, id, bytes)?;
            }
        }
        if let Some((id, entry, head, _)) = &self.activity {
            store.put(Table::Activity, id, entry)?;
            store.put(Table::Activity, &HEAD, head)?;
        }
        Ok(())
    }
}

fn canonical<T: Serialize>(value: &T) -> Result<Zeroizing<Vec<u8>>> {
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
                    .transpose()?;
                undo.push(Previous {
                    id: id.into(),
                    before,
                    version,
                    after_hash: new.map(item_hash).transpose()?,
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
                    actor: crate::changes::by(actor),
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
        Ok(journal)
    }

    pub(super) fn accept_journal(&mut self, journal: Journal) {
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
        let mut id = self.activity_head.last_id.clone();
        for seq in (self.activity_head.first_seq..=self.activity_head.last_seq).rev() {
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
    pub(super) fn undo_attachment_ids(&self) -> Result<HashSet<String>> {
        let mut referenced = HashSet::new();
        self.walk_entries(|entry| {
            for previous in &entry.undo {
                if self.get(&previous.id).is_none() {
                    continue;
                }
                if previous.version.as_ref().is_some_and(|version| {
                    !self
                        .history_index
                        .get(&previous.id)
                        .is_some_and(|metas| metas.iter().any(|m| m.version_id == *version))
                }) {
                    continue;
                }
                if let Some(before) = &previous.before {
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
        self.activity_verify()?;
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
                actor: crate::changes::by(actor),
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
        self.store.transaction(|s| {
            s.put(Table::Activity, &id, &bytes)?;
            s.put(Table::Activity, &HEAD, &head)
        })?;
        self.activity_head = next;
        self.changed();
        Ok(())
    }

    pub fn clear_activity(&mut self, actor: &Actor, confirmation: &str) -> Result<()> {
        if !matches!(actor, Actor::Person) || confirmation != "CLEAR" {
            return Err(Error::invalid("Type CLEAR to clear the activity log."));
        }
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
        self.store.transaction(|s| {
            for id in s.ids(Table::Activity)? {
                s.delete(Table::Activity, &id)?;
            }
            s.put(Table::Activity, &id, &bytes)?;
            s.put(Table::Activity, &HEAD, &head)
        })?;
        self.activity_head = next;
        self.changed();
        Ok(())
    }

    pub fn prune_activity(&mut self, at: i64) -> Result<usize> {
        self.activity_verify()?;
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
        self.store.transaction(|s| {
            for e in &entries[..remove] {
                s.delete(Table::Activity, &super::parse_id(&e.id)?)?;
            }
            s.put(Table::Activity, &id, &bytes)?;
            s.put(Table::Activity, &HEAD, &sealed_head)
        })?;
        self.activity_head = next;
        self.changed();
        Ok(remove)
    }

    pub fn activity_undo(&mut self, actor: &Actor, change_set: &str) -> Result<UndoSummary> {
        self.activity_verify()?;
        let entries = self.entries()?;
        let selected: Vec<_> = entries
            .iter()
            .filter(|e| e.meta.change_set == change_set)
            .collect();
        if selected.is_empty() {
            return Err(Error::NotFound);
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
            let mut item = if let Some(before) = &undo.before {
                serde_json::from_str::<Item>(before).map_err(|_| Error::Corrupt)?
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
        let restored = restore.len();
        self.run_as(actor, |s| s.put_many(restore))?;
        Ok(UndoSummary { restored, skipped })
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
        s.put(Item::Row(crate::database::types::Row::new(
            row_id.clone(),
            "db",
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
}
