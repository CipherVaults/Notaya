//! Small encrypted metadata at unlock; encrypted bodies are fetched on demand.
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::{Zeroize, ZeroizeOnDrop};
use super::{Session, Table, RowId, parse_id};
use crate::{actions::Actor, error::{Error, Result}, model::{Item, now_ms, new_id}};

pub const MINUTE: i64 = 60_000;
pub const DAY: i64 = 24 * 60 * MINUTE;
const CAP: u64 = 500 * 1024 * 1024;

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct VersionMeta {
    pub page_id: String,
    pub version_id: String,
    pub at: i64,
    pub actor: String,
    pub change_set: String,
    pub title: String,
    pub words: usize,
    pub picture_ids: Vec<String>,
    pub before_ai: bool,
    pub bytes: u64,
    pub body_hash: String,
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VersionBody {
    pub title: String,
    pub content: String,
    pub text: String,
    pub icon: String,
    pub cover: String,
}

impl VersionBody {
    pub fn from_item(item: &Item) -> Option<Self> {
        match item {
            Item::Page(p) => Some(Self { title: p.title.clone(), content: p.content.clone(), text: p.text.clone(), icon: p.icon.clone(), cover: p.cover.clone() }),
            Item::Row(p) => Some(Self { title: p.title.clone(), content: p.content.clone(), text: p.text.clone(), icon: p.icon.clone(), cover: p.cover.clone() }),
            _ => None,
        }
    }

    pub fn apply(&self, item: &mut Item) -> Result<()> {
        match item {
            Item::Page(p) => { p.title = self.title.clone(); p.content = self.content.clone(); p.text = self.text.clone(); p.icon = self.icon.clone(); p.cover = self.cover.clone(); p.updated = now_ms(); },
            Item::Row(p) => { p.title = self.title.clone(); p.content = self.content.clone(); p.text = self.text.clone(); p.icon = self.icon.clone(); p.cover = self.cover.clone(); p.updated = now_ms(); },
            _ => return Err(Error::NotFound),
        }
        Ok(())
    }
}

pub(super) struct Snapshot {
    pub meta: VersionMeta,
    pub rows: Vec<(RowId, Vec<u8>)>,
}

pub(super) fn body_id(id: &str) -> Result<RowId> {
    let mut row = parse_id(id)?;
    if row[0] != 0 { return Err(Error::NotFound); }
    row[0] = 1;
    Ok(row)
}

/// Extract only existing clatt/pic identifiers. No URL is ever fetched.
fn pictures(body: &VersionBody) -> Vec<String> {
    let source = format!("{} {} {}", body.content, body.cover, body.icon);
    let mut ids = HashSet::new();
    for marker in ["clatt://localhost/", "http://clatt.localhost/", "https://clatt.localhost/", "pic:"] {
        for tail in source.split(marker).skip(1) {
            let id: String = tail.chars().take_while(|c| c.is_ascii_hexdigit()).take(32).collect();
            if id.len() == 32 { ids.insert(id); }
        }
    }
    let mut ids: Vec<_> = ids.into_iter().collect(); ids.sort(); ids
}

impl Session {
    pub fn history_list(&self, page_id: &str) -> Result<Vec<VersionMeta>> {
        if VersionBody::from_item(self.get(page_id).ok_or(Error::NotFound)?).is_none() { return Err(Error::NotFound); }
        let mut out = self.history_index.get(page_id).cloned().unwrap_or_default();
        out.sort_by_key(|m| (std::cmp::Reverse(m.at), std::cmp::Reverse(m.version_id.clone())));
        Ok(out)
    }

    pub fn history_get(&self, page_id: &str, version: &str) -> Result<VersionBody> {
        let meta = self.history_index.get(page_id).and_then(|v| v.iter().find(|m| m.version_id == version)).ok_or(Error::NotFound)?;
        let body: VersionBody = self.read_json(Table::History, &body_id(version)?)?.ok_or(Error::Corrupt)?;
        if hex::encode(Sha256::digest(serde_json::to_vec(&body).map_err(|_| Error::Corrupt)?)) != meta.body_hash { return Err(Error::Corrupt); }
        Ok(body)
    }

    pub(super) fn prepare_snapshot(&self, item: &Item, actor: &Actor, before_ai: bool, at: i64) -> Result<Option<Snapshot>> {
        let Some(body) = VersionBody::from_item(item) else { return Ok(None) };
        let mut id = crate::crypto::random_bytes::<16>(); id[0] = 0;
        let version_id = hex::encode(id);
        let bytes = serde_json::to_vec(&body).map_err(|_| Error::Corrupt)?;
        let meta = VersionMeta { page_id: item.id().into(), version_id: version_id.clone(), at,
            actor: crate::changes::by(actor), change_set: self.change_set.clone().unwrap_or_else(new_id),
            title: body.title.clone(), words: body.text.split_whitespace().count(), picture_ids: pictures(&body),
            before_ai, bytes: bytes.len() as u64, body_hash: hex::encode(Sha256::digest(&bytes)) };
        Ok(Some(Snapshot { rows: vec![(id, self.seal_json(Table::History, &id, &meta)?),
            (body_id(&version_id)?, self.seal_json(Table::History, &body_id(&version_id)?, &body)?)], meta }))
    }

    pub(super) fn page_changed(old: Option<&Item>, new: &Item) -> bool {
        VersionBody::from_item(new).is_some_and(|body| old.and_then(VersionBody::from_item).as_ref() != Some(&body))
    }

    pub(super) fn history_due(&self, id: &str, at: i64) -> bool {
        self.history_index.get(id).and_then(|v| v.iter().max_by_key(|m| m.at))
            .is_none_or(|m| at.saturating_sub(m.at) >= 10 * MINUTE)
    }

    /// Used after flushing on leaving a page and before restoring a version.
    pub fn snapshot_page(&mut self, id: &str) -> Result<()> {
        let item = self.get(id).ok_or(Error::NotFound)?;
        let Some(body) = VersionBody::from_item(item) else { return Err(Error::NotFound) };
        let hash = hex::encode(Sha256::digest(serde_json::to_vec(&body).map_err(|_| Error::Corrupt)?));
        if self.history_index.get(id).and_then(|v| v.last()).is_some_and(|m| m.body_hash == hash) { return Ok(()); }
        let snapshot = self.prepare_snapshot(item, &Actor::Person, false, now_ms())?.ok_or(Error::NotFound)?;
        self.store.transaction(|s| { for (id, bytes) in &snapshot.rows { s.put(Table::History, id, bytes)?; } Ok(()) })?;
        self.history_index.entry(id.into()).or_default().push(snapshot.meta);
        self.changed();
        Ok(())
    }

    pub fn history_restore(&mut self, actor: &Actor, page_id: &str, version: &str) -> Result<()> {
        self.check_write(actor, page_id, None)?;
        if self.in_bin(page_id) { return Err(Error::invalid("Restore this page from the bin first.")); }
        let body = self.history_get(page_id, version)?;
        self.snapshot_page(page_id)?;
        let mut item = self.get(page_id).ok_or(Error::NotFound)?.clone();
        body.apply(&mut item)?;
        self.run_as(actor, |s| s.put(item))
    }

    /// Retention operates on metadata only. Protected minima may exceed the
    /// soft cap; preserving undo and the latest 20 wins over the size target.
    pub fn thin_history(&mut self, at: i64) -> Result<usize> {
        let mut keep = HashSet::new();
        let mut candidates = Vec::new();
        let mut total = 0u64;
        for versions in self.history_index.values() {
            let mut ordered: Vec<_> = versions.iter().collect(); ordered.sort_by_key(|m| (std::cmp::Reverse(m.at), &m.version_id));
            let mut hours = HashSet::new(); let mut days = HashSet::new();
            for (n, m) in ordered.into_iter().enumerate() {
                total += m.bytes;
                let age = at.saturating_sub(m.at);
                let protected = n < 20 || (m.before_ai && age <= 90 * DAY);
                let retain = protected || age <= 7 * DAY || (age <= 30 * DAY && hours.insert(m.at / (60 * MINUTE))) || (age <= 365 * DAY && days.insert(m.at / DAY));
                if retain { keep.insert(m.version_id.clone()); }
                if !protected { candidates.push((m.at, m.version_id.clone(), m.bytes)); }
            }
        }
        candidates.sort();
        for (_, id, bytes) in &candidates { if !keep.contains(id) { total = total.saturating_sub(*bytes); } }
        for (_, id, bytes) in candidates { if total <= CAP { break } if keep.remove(&id) { total = total.saturating_sub(bytes); } }
        let remove: Vec<_> = self.history_index.values().flatten().filter(|m| !keep.contains(&m.version_id)).map(|m| m.version_id.clone()).collect();
        self.store.transaction(|s| { for id in &remove { s.delete(Table::History, &parse_id(id)?)?; s.delete(Table::History, &body_id(id)?)?; } Ok(()) })?;
        for versions in self.history_index.values_mut() { versions.retain(|m| keep.contains(&m.version_id)); }
        if !remove.is_empty() { self.changed(); }
        Ok(remove.len())
    }

    pub(super) fn load_history(&mut self) -> Result<()> {
        for (id, sealed) in self.store.history_meta()? {
            let meta: VersionMeta = self.open_json(Table::History, &id, &sealed)?;
            if meta.version_id != hex::encode(id) { return Err(Error::Corrupt); }
            self.history_index.entry(meta.page_id.clone()).or_default().push(meta);
        }
        for versions in self.history_index.values_mut() { versions.sort_by_key(|m| (m.at, m.version_id.clone())); }
        Ok(())
    }
}

pub(super) type Index = HashMap<String, Vec<VersionMeta>>;
