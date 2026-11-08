//! Small encrypted metadata at unlock; encrypted bodies are fetched on demand.
use super::{parse_id, RowId, Session, Table};
use crate::{
    actions::Actor,
    error::{Error, Result},
    model::{new_id, now_ms, Item},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

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
            Item::Page(p) => Some(Self {
                title: p.title.clone(),
                content: p.content.clone(),
                text: p.text.clone(),
                icon: p.icon.clone(),
                cover: p.cover.clone(),
            }),
            Item::Row(p) => Some(Self {
                title: p.title.clone(),
                content: p.content.clone(),
                text: p.text.clone(),
                icon: p.icon.clone(),
                cover: p.cover.clone(),
            }),
            _ => None,
        }
    }

    pub fn apply(&self, item: &mut Item) -> Result<()> {
        match item {
            Item::Page(p) => {
                p.title = self.title.clone();
                p.content = self.content.clone();
                p.text = self.text.clone();
                p.icon = self.icon.clone();
                p.cover = self.cover.clone();
                p.updated = now_ms();
            }
            Item::Row(p) => {
                p.title = self.title.clone();
                p.content = self.content.clone();
                p.text = self.text.clone();
                p.icon = self.icon.clone();
                p.cover = self.cover.clone();
                p.updated = now_ms();
            }
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
    if row[0] != 0 {
        return Err(Error::NotFound);
    }
    row[0] = 1;
    Ok(row)
}

/// Extract only existing clatt/pic identifiers. No URL is ever fetched.
fn pictures(body: &VersionBody) -> Vec<String> {
    let source = format!("{} {} {}", body.content, body.cover, body.icon);
    let mut ids = HashSet::new();
    for marker in [
        "clatt://localhost/",
        "http://clatt.localhost/",
        "https://clatt.localhost/",
        "pic:",
    ] {
        for tail in source.split(marker).skip(1) {
            let id: String = tail
                .chars()
                .take_while(|c| c.is_ascii_hexdigit())
                .take(32)
                .collect();
            if id.len() == 32 {
                ids.insert(id);
            }
        }
    }
    let mut ids: Vec<_> = ids.into_iter().collect();
    ids.sort();
    ids
}

impl Session {
    pub fn history_list(&self, page_id: &str) -> Result<Vec<VersionMeta>> {
        if !matches!(self.get(page_id), Some(Item::Page(_) | Item::Row(_))) {
            return Err(Error::NotFound);
        }
        let mut out = self.history_index.get(page_id).cloned().unwrap_or_default();
        out.sort_by_key(|m| {
            (
                std::cmp::Reverse(m.at),
                std::cmp::Reverse(m.version_id.clone()),
            )
        });
        Ok(out)
    }

    pub fn history_get(&self, page_id: &str, version: &str) -> Result<VersionBody> {
        let meta = self
            .history_index
            .get(page_id)
            .and_then(|v| v.iter().find(|m| m.version_id == version))
            .ok_or(Error::NotFound)?;
        let body: VersionBody = self
            .read_json(Table::History, &body_id(version)?)?
            .ok_or(Error::Corrupt)?;
        let bytes = Zeroizing::new(serde_json::to_vec(&body).map_err(|_| Error::Corrupt)?);
        if hex::encode(Sha256::digest(&*bytes)) != meta.body_hash {
            return Err(Error::Corrupt);
        }
        Ok(body)
    }

    pub(super) fn prepare_snapshot(
        &self,
        item: &Item,
        actor: &Actor,
        before_ai: bool,
        at: i64,
    ) -> Result<Option<Snapshot>> {
        let Some(body) = VersionBody::from_item(item) else {
            return Ok(None);
        };
        let mut id = crate::crypto::random_bytes::<16>();
        id[0] = 0;
        let version_id = hex::encode(id);
        let bytes = Zeroizing::new(serde_json::to_vec(&body).map_err(|_| Error::Corrupt)?);
        let mut meta = VersionMeta {
            page_id: item.id().into(),
            version_id: version_id.clone(),
            at,
            actor: super::activity::actor_label(actor),
            change_set: self.change_set.clone().unwrap_or_else(new_id),
            title: body.title.clone(),
            words: body.text.split_whitespace().count(),
            picture_ids: pictures(&body),
            before_ai,
            bytes: bytes.len() as u64,
            body_hash: hex::encode(Sha256::digest(&bytes)),
        };
        let body_row = body_id(&version_id)?;
        let sealed_body = self.seal_json(Table::History, &body_row, &body)?;
        let mut sealed_meta = self.seal_json(Table::History, &id, &meta)?;
        // Count both padded ciphertext rows toward the cap. Re-seal metadata
        // until the decimal size field no longer changes its padding bucket.
        loop {
            let bytes = (sealed_body.len() + sealed_meta.len()) as u64;
            if meta.bytes == bytes {
                break;
            }
            meta.bytes = bytes;
            sealed_meta = self.seal_json(Table::History, &id, &meta)?;
        }
        Ok(Some(Snapshot {
            rows: vec![(id, sealed_meta), (body_row, sealed_body)],
            meta,
        }))
    }

    pub(super) fn page_changed(old: Option<&Item>, new: &Item) -> bool {
        VersionBody::from_item(new)
            .is_some_and(|body| old.and_then(VersionBody::from_item).as_ref() != Some(&body))
    }

    pub(super) fn history_due(&self, id: &str, at: i64) -> bool {
        self.history_index
            .get(id)
            .and_then(|v| {
                v.iter()
                    .filter(|m| m.actor == "person")
                    .max_by_key(|m| m.at)
            })
            .is_none_or(|m| at.saturating_sub(m.at) >= 10 * MINUTE)
    }

    /// Used after flushing on leaving a page and before restoring a version.
    pub fn snapshot_page(&mut self, id: &str) -> Result<()> {
        let item = self.get(id).ok_or(Error::NotFound)?;
        let Some(body) = VersionBody::from_item(item) else {
            return Err(Error::NotFound);
        };
        let bytes = Zeroizing::new(serde_json::to_vec(&body).map_err(|_| Error::Corrupt)?);
        let hash = hex::encode(Sha256::digest(&*bytes));
        if self
            .history_index
            .get(id)
            .and_then(|v| v.last())
            .is_some_and(|m| m.body_hash == hash)
        {
            return Ok(());
        }
        let actor = self.actor.clone().unwrap_or(Actor::Person);
        let snapshot = self
            .prepare_snapshot(item, &actor, matches!(actor, Actor::Ai { .. }), now_ms())?
            .ok_or(Error::NotFound)?;
        self.store.transaction(|s| {
            for (id, bytes) in &snapshot.rows {
                s.put(Table::History, id, bytes)?;
            }
            Ok(())
        })?;
        self.history_index
            .entry(id.into())
            .or_default()
            .push(snapshot.meta);
        self.changed();
        Ok(())
    }

    pub fn history_restore(&mut self, actor: &Actor, page_id: &str, version: &str) -> Result<()> {
        self.check_write(actor, page_id, None)?;
        if self.in_bin(page_id) {
            return Err(Error::invalid("Restore this page from the bin first."));
        }
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
            let mut ordered: Vec<_> = versions.iter().collect();
            ordered.sort_by_key(|m| (std::cmp::Reverse(m.at), &m.version_id));
            let mut hours = HashSet::new();
            let mut days = HashSet::new();
            for (n, m) in ordered.into_iter().enumerate() {
                total += m.bytes;
                let age = at.saturating_sub(m.at);
                let protected = n < 20 || (m.before_ai && age <= 90 * DAY);
                let retain = protected
                    || age <= 7 * DAY
                    || (age <= 30 * DAY && hours.insert(m.at / (60 * MINUTE)))
                    || (age > 30 * DAY && age <= 365 * DAY && days.insert(m.at / DAY));
                if retain {
                    keep.insert(m.version_id.clone());
                }
                if !protected {
                    candidates.push((m.at, m.version_id.clone(), m.bytes));
                }
            }
        }
        candidates.sort();
        for (_, id, bytes) in &candidates {
            if !keep.contains(id) {
                total = total.saturating_sub(*bytes);
            }
        }
        for (_, id, bytes) in candidates {
            if total <= CAP {
                break;
            }
            if keep.remove(&id) {
                total = total.saturating_sub(bytes);
            }
        }
        let remove: Vec<_> = self
            .history_index
            .values()
            .flatten()
            .filter(|m| !keep.contains(&m.version_id))
            .map(|m| m.version_id.clone())
            .collect();
        self.store.transaction(|s| {
            for id in &remove {
                s.delete(Table::History, &parse_id(id)?)?;
                s.delete(Table::History, &body_id(id)?)?;
            }
            Ok(())
        })?;
        for versions in self.history_index.values_mut() {
            versions.retain(|m| keep.contains(&m.version_id));
        }
        if !remove.is_empty() {
            self.changed();
        }
        Ok(remove.len())
    }

    pub(super) fn load_history(&mut self) -> Result<()> {
        for (id, sealed) in self.store.history_meta()? {
            let meta: VersionMeta = self.open_json(Table::History, &id, &sealed)?;
            if meta.version_id != hex::encode(id) {
                return Err(Error::Corrupt);
            }
            self.history_index
                .entry(meta.page_id.clone())
                .or_default()
                .push(meta);
        }
        for versions in self.history_index.values_mut() {
            versions.sort_by_key(|m| (m.at, m.version_id.clone()));
        }
        Ok(())
    }
}

pub(super) type Index = HashMap<String, Vec<VersionMeta>>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KdfParams;
    fn fixture() -> (Session, std::path::PathBuf, String) {
        let dir = std::path::PathBuf::from(
            std::env::var_os("NOTAYA_HOME").expect("throwaway NOTAYA_HOME required"),
        )
        .join(format!("c3-history-{}", new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("vault.nota");
        let mut s = Session::create(
            &path,
            "throwaway",
            None,
            KdfParams {
                m_kib: 64,
                t: 1,
                p: 1,
            },
        )
        .unwrap();
        let id = new_id();
        s.put(Item::Page(serde_json::from_value(serde_json::json!({"id":id,"sectionId":"","title":"Page","content":"original","text":"original","order":0,"favorite":false,"trashedAt":null,"created":1,"updated":1})).unwrap())).unwrap();
        (s, path, id)
    }
    fn edit(s: &mut Session, id: &str, text: &str, actor: &Actor) {
        let mut item = s.get(id).unwrap().clone();
        let mut body = VersionBody::from_item(&item).unwrap();
        body.content = text.into();
        body.text = text.into();
        body.apply(&mut item).unwrap();
        s.run_as(actor, |s| s.put(item)).unwrap();
    }
    fn ai() -> Actor {
        Actor::Ai {
            pairing_id: "one".into(),
            label: "Test AI".into(),
        }
    }
    fn insert(s: &mut Session, id: &str, at: i64, before_ai: bool, bytes: Option<u64>) -> String {
        let mut snap = s
            .prepare_snapshot(s.get(id).unwrap(), &ai(), before_ai, at)
            .unwrap()
            .unwrap();
        if let Some(bytes) = bytes {
            snap.meta.bytes = bytes;
            snap.rows[0].1 = s
                .seal_json(Table::History, &snap.rows[0].0, &snap.meta)
                .unwrap();
        }
        for (id, bytes) in &snap.rows {
            s.store.put(Table::History, id, bytes).unwrap();
        }
        let version = snap.meta.version_id.clone();
        s.history_index
            .entry(id.into())
            .or_default()
            .push(snap.meta);
        version
    }

    #[test]
    fn snapshots_coalesce_person_edits_capture_leave_and_keep_ai_before_after() {
        let (mut s, _, id) = fixture();
        edit(&mut s, &id, "person one", &Actor::Person);
        edit(&mut s, &id, "person two", &Actor::Person);
        assert_eq!(s.history_list(&id).unwrap().len(), 1);
        s.snapshot_page(&id).unwrap();
        assert_eq!(s.history_list(&id).unwrap().len(), 2);
        s.snapshot_page(&id).unwrap();
        assert_eq!(s.history_list(&id).unwrap().len(), 2);
        for meta in s.history_index.get_mut(&id).unwrap() {
            meta.at = now_ms() - 10 * MINUTE - 1;
        }
        edit(&mut s, &id, "person three", &Actor::Person);
        assert_eq!(s.history_list(&id).unwrap().len(), 3);
        let activity = s.activity_list(Some("person"), None, 100).unwrap();
        assert_eq!(
            activity
                .iter()
                .filter(|e| e.items.iter().any(|i| i.id == id))
                .count(),
            2,
            "create plus one editing burst"
        );
        edit(&mut s, &id, "AI version", &ai());
        let versions = s.history_list(&id).unwrap();
        assert_eq!(versions.len(), 5);
        let before = versions.iter().find(|v| v.before_ai).unwrap();
        assert_eq!(
            s.history_get(&id, &before.version_id).unwrap().text,
            "person three"
        );
        assert!(versions.iter().any(|v| v.actor == "ai:Test AI"
            && !v.before_ai
            && s.history_get(&id, &v.version_id).unwrap().text == "AI version"));
        let target = versions
            .iter()
            .find(|v| v.title == "Page" && v.actor == "person")
            .unwrap()
            .version_id
            .clone();
        s.history_restore(&Actor::Person, &id, &target).unwrap();
        assert!(s.history_list(&id).unwrap().iter().any(|v| s
            .history_get(&id, &v.version_id)
            .unwrap()
            .text
            == "AI version"));
    }

    #[test]
    fn retention_keeps_recent_hourly_daily_minimum_and_ai_before_versions() {
        let (mut s, _, id) = fixture();
        let at = now_ms();
        // More than twenty newer versions keep the minimum from masking the policy.
        for n in 0..22 {
            insert(&mut s, &id, at - n * MINUTE, false, None);
        }
        let recent_a = insert(&mut s, &id, at - 6 * DAY, false, None);
        let recent_b = insert(&mut s, &id, at - 6 * DAY - MINUTE, false, None);
        let hour_a = insert(
            &mut s,
            &id,
            (at - 10 * DAY) / (60 * MINUTE) * (60 * MINUTE) + MINUTE,
            false,
            None,
        );
        let hour_b = insert(
            &mut s,
            &id,
            (at - 10 * DAY) / (60 * MINUTE) * (60 * MINUTE),
            false,
            None,
        );
        let daily = insert(&mut s, &id, at - 200 * DAY, false, None);
        let ai_before = insert(&mut s, &id, at - 89 * DAY, true, None);
        let expired = insert(&mut s, &id, at - 366 * DAY, false, None);
        s.thin_history(at).unwrap();
        let kept: HashSet<_> = s
            .history_list(&id)
            .unwrap()
            .iter()
            .map(|m| m.version_id.clone())
            .collect();
        for version in [recent_a, recent_b, hour_a, daily, ai_before] {
            assert!(kept.contains(&version));
        }
        assert!(!kept.contains(&hour_b));
        assert!(!kept.contains(&expired));
    }

    #[test]
    fn history_cap_thins_oldest_unprotected_but_preserves_twenty_and_ai_before() {
        let (mut s, _, id) = fixture();
        let at = now_ms();
        let protected = insert(&mut s, &id, at - 80 * DAY, true, Some(CAP));
        let oldest = insert(&mut s, &id, at - 5 * DAY, false, Some(CAP));
        for n in 0..21 {
            insert(&mut s, &id, at - n * MINUTE, false, None);
        }
        s.thin_history(at).unwrap();
        let kept = s.history_list(&id).unwrap();
        assert!(kept.iter().any(|v| v.version_id == protected));
        assert!(!kept.iter().any(|v| v.version_id == oldest));
        assert!(kept.len() >= 21);
    }

    #[test]
    fn gc_keeps_history_only_pictures_including_bin_then_permanent_delete_removes_history() {
        let (mut s, path, id) = fixture();
        let pic = s.put_attachment("image/png", b"throwaway pixels").unwrap();
        edit(
            &mut s,
            &id,
            &format!("http://clatt.localhost/{pic}"),
            &Actor::Person,
        );
        s.snapshot_page(&id).unwrap();
        edit(&mut s, &id, "removed body sentinel", &ai());
        assert_eq!(s.collect_garbage().unwrap(), 0);
        assert!(s.get_attachment(&pic).is_ok());
        let mut item = s.get(&id).unwrap().clone();
        if let Item::Page(p) = &mut item {
            p.trashed_at = Some(now_ms());
        }
        s.put(item).unwrap();
        assert!(!s.history_list(&id).unwrap().is_empty());
        crate::commands::notes::empty_trash_core(&mut s, &Actor::Person).unwrap();
        assert!(s.history_index.get(&id).is_none());
        s.collect_garbage().unwrap();
        assert!(s.get_attachment(&pic).is_err());
        assert!(s.store.ids(Table::History).unwrap().is_empty());
        drop(s);
        let s = Session::unlock(&path, "throwaway", None).unwrap();
        assert!(s.history_index.is_empty());
        for (row, sealed) in s.store.all(Table::Activity).unwrap() {
            let plain = crate::crypto::open(
                &s.dek,
                &super::super::aad(s.format, &s.vault_id, Table::Activity, &row),
                &sealed,
            )
            .unwrap();
            assert!(!String::from_utf8_lossy(&plain).contains("removed body sentinel"));
        }
    }

    #[test]
    fn permanent_delete_removes_page_and_row_history_and_undo_bodies() {
        for row in [false, true] {
            let (mut s, path, kept) = fixture();
            let kept_history = s.store.all(Table::History).unwrap();
            let id = new_id();
            let mut item = if row {
                Item::Row(crate::database::types::Row::new(id.clone(), "db", "Row".into(), 1))
            } else {
                let Item::Page(mut page) = s.get(&kept).unwrap().clone() else { panic!() };
                page.id = id.clone();
                Item::Page(page)
            };
            let mut body = VersionBody::from_item(&item).unwrap();
            body.content = "deleted undo body sentinel".into();
            body.text = body.content.clone();
            body.apply(&mut item).unwrap();
            s.put(item).unwrap();
            edit(&mut s, &id, "deleted AI body sentinel", &ai());
            let change_set = s.activity_list(Some("ai:Test AI"), None, 1).unwrap()[0].change_set.clone();
            let versions = s.history_list(&id).unwrap();
            assert!(versions.iter().any(|v| v.before_ai));
            assert!(versions.iter().any(|v| s.history_get(&id, &v.version_id).unwrap().text == "deleted undo body sentinel"));
            let kept_versions = serde_json::to_value(s.history_list(&kept).unwrap()).unwrap();
            // A refused live delete must preserve the encrypted undo records too,
            // including bodies that are not returned in activity-list metadata.
            let history = s.store.all(Table::History).unwrap();
            let activity = s.store.all(Table::Activity).unwrap();
            let error = crate::commands::notes::delete_page_core(&mut s, &Actor::Person, id.clone()).err().unwrap();
            assert_eq!(error.to_string(), "Only items in the bin can be permanently deleted.");
            assert_eq!(s.store.all(Table::History).unwrap(), history);
            assert_eq!(s.store.all(Table::Activity).unwrap(), activity);
            crate::commands::notes::trash_page_core(&mut s, &Actor::Person, id.clone()).unwrap();
            // Include any versions taken by the bin operation in the deletion check.
            let versions = s.history_list(&id).unwrap();
            crate::commands::notes::delete_page_core(&mut s, &Actor::Person, id.clone()).unwrap();
            assert!(s.get(&id).is_none());
            assert!(!s.history_index.contains_key(&id));
            for version in &versions {
                assert!(s.store.get(Table::History, &parse_id(&version.version_id).unwrap()).unwrap().is_none());
                assert!(s.store.get(Table::History, &body_id(&version.version_id).unwrap()).unwrap().is_none());
            }
            assert_eq!(s.store.all(Table::History).unwrap(), kept_history);
            assert_eq!(serde_json::to_value(s.history_list(&kept).unwrap()).unwrap(), kept_versions);
            drop(s);
            let mut s = Session::unlock(&path, "throwaway", None).unwrap();
            assert!(!s.history_index.contains_key(&id));
            assert_eq!(s.store.all(Table::History).unwrap(), kept_history);
            for (row, bytes) in s.store.all(Table::Activity).unwrap() {
                let plain = crate::crypto::open(&s.dek,
                    &super::super::aad(s.format, &s.vault_id, Table::Activity, &row), &bytes).unwrap();
                let plain = String::from_utf8_lossy(&plain);
                assert!(!plain.contains("deleted undo body sentinel"));
                assert!(!plain.contains("deleted AI body sentinel"));
            }
            let summary = s.activity_undo(&Actor::Person, &change_set).unwrap();
            assert_eq!((summary.restored, summary.skipped), (0, 1));
            assert!(s.get(&id).is_none(), "undo cannot recreate a permanently deleted item");
            s.activity_verify().unwrap();
        }
    }

    #[test]
    fn thousand_large_versions_unlock_with_only_metadata_and_lazy_body_reads() {
        let (mut s, path, id) = fixture();
        let mut item = s.get(&id).unwrap().clone();
        if let Item::Page(p) = &mut item {
            p.content = "x".repeat(8192);
            p.text = "y".repeat(8192);
        }
        s.items.insert(id.clone(), item);
        let mut snapshots = vec![];
        for n in 0..1000 {
            snapshots.push(
                s.prepare_snapshot(s.get(&id).unwrap(), &ai(), false, n)
                    .unwrap()
                    .unwrap(),
            );
        }
        s.store
            .transaction(|store| {
                for snap in &snapshots {
                    for (id, bytes) in &snap.rows {
                        store.put(Table::History, id, bytes)?;
                    }
                }
                Ok(())
            })
            .unwrap();
        let body_bytes: u64 = snapshots.iter().map(|snap| snap.meta.bytes).sum();
        let version = snapshots[0].meta.version_id.clone();
        s.store
            .put(
                Table::History,
                &body_id(&version).unwrap(),
                b"damaged lazy body",
            )
            .unwrap();
        drop(s);
        let s = Session::unlock(&path, "throwaway", None).expect("unlock reads only metadata");
        let index_bytes = serde_json::to_vec(&s.history_index).unwrap().len() as u64;
        assert_eq!(s.history_list(&id).unwrap().len(), 1001);
        assert!(
            index_bytes < body_bytes / 10,
            "{index_bytes} metadata bytes for {body_bytes} body bytes"
        );
        assert!(s.history_get(&id, &version).is_err());
        assert!(s.verify_attachments().is_err());
    }
}
