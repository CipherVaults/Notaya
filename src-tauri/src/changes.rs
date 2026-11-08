//! Session-only coordination. The dispatcher supplies Actor; action parameters never do.
use std::collections::{BTreeMap, HashMap};
use std::time::{Duration, Instant};

use crate::actions::Actor;
use crate::error::{Error, Result};
use serde::Serialize;

#[derive(Clone, Default, Serialize)]
pub struct ItemsChanged {
    pub ids: Vec<String>,
    pub kinds: Vec<String>,
    pub tree: bool,
    pub by: String,
    pub revs: BTreeMap<String, u64>,
}

struct Claim {
    owner: String,
    until: Instant,
    explicit: bool,
}

#[derive(Default)]
pub struct Coordination {
    leases: HashMap<String, HashMap<String, Option<Instant>>>,
    claims: HashMap<String, Claim>,
}

impl Coordination {
    pub fn editing(&mut self, id: &str, window: &str, active: bool) {
        self.leases.entry(id.into()).or_default().insert(
            window.into(),
            if active {
                None
            } else {
                Some(Instant::now() + Duration::from_secs(3))
            },
        );
    }

    pub fn forget_window(&mut self, window: &str) {
        self.leases.retain(|_, windows| {
            windows.remove(window);
            !windows.is_empty()
        });
    }

    pub fn check(&self, actor: &Actor, related: impl Fn(&str) -> bool) -> Result<()> {
        let Actor::Ai { pairing_id, .. } = actor else {
            return Ok(());
        };
        let now = Instant::now();
        if self.leases.iter().any(|(id, windows)| {
            related(id) && windows.values().any(|until| until.is_none_or(|t| t > now))
        }) {
            return Err(Error::invalid(
                "person_editing: This item is being edited; read it again in a moment.",
            ));
        }
        if self.claims.iter().any(|(id, c)| {
            c.owner != *pairing_id && c.until > now && related(id)
        }) {
            return Err(Error::invalid(
                "claimed_by_other_ai: Another AI is working on this item.",
            ));
        }
        Ok(())
    }

    pub fn claim(&mut self, actor: &Actor, ids: &[String], automatic: bool) -> Result<()> {
        let Actor::Ai { pairing_id, .. } = actor else {
            return Err(Error::invalid("Only an AI connection can claim items."));
        };
        let now = Instant::now();
        self.claims.retain(|_, c| c.until > now);
        // Check the whole batch before changing any claim, including callers that
        // use this primitive rather than Session's container-aware entry point.
        for id in ids {
            self.check(actor, |other| other == id)?;
        }
        for id in ids {
            let duration = if automatic { 120 } else { 1800 };
            let until = now + Duration::from_secs(duration);
            self.claims
                .entry(id.clone())
                .and_modify(|c| {
                    // Activity refreshes an explicit claim without shortening it.
                    c.explicit |= !automatic;
                    c.until = now + Duration::from_secs(if c.explicit { 1800 } else { duration });
                })
                .or_insert(Claim {
                    owner: pairing_id.clone(),
                    until,
                    explicit: !automatic,
                });
        }
        Ok(())
    }

    pub fn touch(&mut self, actor: &Actor, related: impl Fn(&str) -> bool) {
        let Actor::Ai { pairing_id, .. } = actor else {
            return;
        };
        let now = Instant::now();
        for (id, claim) in &mut self.claims {
            if claim.owner == *pairing_id && claim.until > now && related(id) {
                claim.until = now + Duration::from_secs(if claim.explicit { 1800 } else { 120 });
            }
        }
    }

    /// Used by release, Stop and disconnect. Dropping Session clears everything at lock.
    pub fn release(&mut self, actor: &Actor, ids: Option<&[String]>) {
        if let Actor::Ai { pairing_id, .. } = actor {
            self.claims
                .retain(|id, c| c.owner != *pairing_id || ids.is_some_and(|ids| !ids.contains(id)));
        }
    }
}

pub fn by(actor: &Actor) -> String {
    match actor {
        Actor::Person => "person".into(),
        Actor::Automation => "automation".into(),
        Actor::Ai { .. } => "ai".into(),
    }
}

pub(crate) fn claim_core(
    s: &mut crate::vault::Session,
    actor: &Actor,
    ids: Vec<String>,
) -> Result<()> {
    if ids.len() > 10_000 {
        return Err(Error::invalid("Too many items at once."));
    }
    s.claim(actor, &ids)
}

pub(crate) fn release_core(
    s: &mut crate::vault::Session,
    actor: &Actor,
    ids: Option<Vec<String>>,
) -> Result<()> {
    s.coordination.release(actor, ids.as_deref());
    Ok(())
}

#[derive(Serialize)]
pub struct Revisioned<T: Serialize> {
    #[serde(flatten)]
    pub item: T,
    pub rev: u64,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::commands::{notes, stickies, tools};
    use crate::model::{Item, Page, Sticky, new_id};
    use crate::portable::Paths;
    use crate::vault::Session;
    use serde_json::{Value, json};
    use std::sync::{Arc, Mutex};

    const NB: &str = "20000000000000000000000000000001";
    const SEC: &str = "20000000000000000000000000000002";
    const PAGE: &str = "20000000000000000000000000000003";
    const STICKY: &str = "20000000000000000000000000000004";
    const DB: &str = "20000000000000000000000000000005";
    const ROW: &str = "20000000000000000000000000000006";

    fn ai(n: &str) -> Actor {
        Actor::Ai {
            pairing_id: n.into(),
            label: n.into(),
        }
    }
    pub(crate) fn fixture() -> (Session, Paths) {
        let root = std::path::PathBuf::from(
            std::env::var_os("NOTAYA_HOME").unwrap_or_else(|| "D:\\Notaya Tests\\v1-6-C2".into()),
        )
        .join(format!("c2-{}", new_id()));
        let paths = Paths::at(&root);
        std::fs::create_dir_all(&paths.data).unwrap();
        let mut s = Session::create(
            &paths.vault,
            "throwaway",
            None,
            crate::crypto::KdfParams {
                m_kib: 64,
                t: 1,
                p: 1,
            },
        )
        .unwrap();
        for value in [
            json!({"kind":"notebook","id":NB,"name":"exact notebook","color":"","icon":"","order":0,"created":1,"updated":1}),
            json!({"kind":"section","id":SEC,"notebookId":NB,"name":"exact section","color":"","order":0,"created":1,"updated":1}),
            json!({"kind":"page","id":PAGE,"sectionId":SEC,"title":"exact page","content":"","text":"exact text","order":0,"favorite":true,"trashedAt":null,"created":1,"updated":1}),
            json!({"kind":"sticky","id":STICKY,"content":"","text":"exact sticky","color":"yellow","x":0,"y":0,"w":320,"h":320,"onTop":true,"open":true,"created":1,"updated":1}),
        ] {
            s.put(serde_json::from_value(value).unwrap()).unwrap();
        }
        s.put(Item::Database(crate::database::types::Database::new(
            DB.into(),
            SEC.into(),
            "exact database".into(),
            1,
        )))
        .unwrap();
        s.put(Item::Row(crate::database::types::Row::new(
            ROW.into(),
            DB,
            "exact row".into(),
            1,
        )))
        .unwrap();
        s.drain_changes();
        (s, paths)
    }
    fn save(s: &mut Session, actor: &Actor, text: &str) -> Result<crate::model::PageMeta> {
        notes::save_page_core(
            s,
            actor,
            PAGE.into(),
            "exact page".into(),
            text.into(),
            text.into(),
        )
    }

    #[test]
    fn permanent_delete_requires_supported_bin_item_and_refuses_stale_restore_action() {
        let (mut s, paths) = fixture();
        s.put(Item::Settings(s.settings())).unwrap();
        s.drain_changes();
        for id in [NB, SEC, PAGE, STICKY, DB, ROW, crate::model::SETTINGS_ID] {
            let before = serde_json::to_value(s.get(id).unwrap()).unwrap();
            let rev = s.rev(id);
            let error = notes::delete_page_core(&mut s, &Actor::Person, id.into()).err().unwrap();
            assert_eq!(error.to_string(), "Only items in the bin can be permanently deleted.");
            assert_eq!(serde_json::to_value(s.get(id).unwrap()).unwrap(), before);
            assert_eq!(s.rev(id), rev);
            assert!(s.drain_changes().is_none());
        }
        // The restore wins the session lock before a stale Trash click arrives.
        notes::delete_notebook_core(&mut s, &Actor::Person, &paths, NB.into()).unwrap();
        notes::restore_page_core(&mut s, &Actor::Person, NB.into()).unwrap();
        assert!(notes::delete_page_core(&mut s, &Actor::Person, NB.into()).is_err());
        for id in [NB, SEC, PAGE, DB, ROW] { assert!(s.get(id).is_some() && !s.in_bin(id)); }
        // Effective bin membership is sufficient for children of binned containers.
        notes::delete_section_core(&mut s, &Actor::Person, &paths, SEC.into()).unwrap();
        notes::delete_page_core(&mut s, &Actor::Person, PAGE.into()).unwrap();
        assert!(s.get(PAGE).is_none());
        assert!(s.get(SEC).is_some() && s.in_bin(SEC));
    }

    #[test]
    fn coordination_errors_and_signals_never_include_names_or_item_text() {
        let (mut s, _) = fixture();
        let a = ai("Ahmed exact page exact text");
        let b = ai("Mohamed");
        s.coordination.editing(PAGE, "main", true);
        let error = save(&mut s, &a, "ignored").err().unwrap();
        assert_eq!(serde_json::to_value(error).unwrap(),
            "person_editing: This item is being edited; read it again in a moment.");
        s.coordination.forget_window("main");
        s.claim(&a, &[PAGE.into()]).unwrap();
        let error = save(&mut s, &b, "ignored").err().unwrap();
        assert_eq!(serde_json::to_value(error).unwrap(),
            "claimed_by_other_ai: Another AI is working on this item.");
        save(&mut s, &a, "exact text").unwrap();
        let event = serde_json::to_value(s.drain_changes().unwrap()).unwrap();
        assert_eq!(event["by"], "ai");
        let serialized = event.to_string();
        for private in ["Ahmed", "Mohamed", "exact page", "exact text", "exact notebook"] {
            assert!(!serialized.contains(private));
        }
        save(&mut s, &Actor::Person, "person still wins").unwrap();
    }

    #[test]
    fn child_page_and_database_restore_reparent_without_reviving_binned_page() {
        let (mut s, paths) = fixture();
        let Item::Page(mut child) = s.get(PAGE).unwrap().clone() else { unreachable!() };
        child.id = new_id();
        child.parent = PAGE.into();
        let child_id = child.id.clone();
        s.put(Item::Page(child)).unwrap();
        let Item::Database(mut db) = s.get(DB).unwrap().clone() else { unreachable!() };
        db.parent = PAGE.into();
        db.locked = true;
        s.put(Item::Database(db)).unwrap();
        // Existing Undo: the child was trashed, then its parent was trashed.
        notes::trash_page_core(&mut s, &Actor::Person, child_id.clone()).unwrap();
        notes::trash_page_core(&mut s, &Actor::Person, DB.into()).unwrap();
        notes::trash_page_core(&mut s, &Actor::Person, PAGE.into()).unwrap();
        notes::restore_page_core(&mut s, &Actor::Person, child_id.clone()).unwrap();
        notes::restore_bin_item_core(&mut s, &Actor::Person, DB.into(), false).unwrap();
        assert!(s.in_bin(PAGE));
        assert!(!s.in_bin(&child_id) && !s.in_bin(DB));
        assert!(matches!(s.get(&child_id), Some(Item::Page(p)) if p.parent.is_empty()));
        assert!(matches!(s.get(DB), Some(Item::Database(d)) if d.parent.is_empty() && d.locked));
        // Container confirmation still applies to every restore entry point.
        notes::delete_section_core(&mut s, &Actor::Person, &paths, SEC.into()).unwrap();
        assert!(notes::restore_page_core(&mut s, &Actor::Person, child_id.clone()).is_err());
        assert!(notes::restore_bin_item_core(&mut s, &Actor::Person, DB.into(), false).is_err());
        notes::restore_bin_item_core(&mut s, &Actor::Person, DB.into(), true).unwrap();
        assert!(!s.in_bin(DB) && s.in_bin(PAGE));
    }

    #[test]
    fn revisions_bump_for_put_batch_and_remove_and_reset_on_unlock() {
        let (mut s, paths) = fixture();
        let item = s.get(PAGE).unwrap().clone();
        let first = s.rev(PAGE);
        s.put(item.clone()).unwrap();
        assert_eq!(s.rev(PAGE), first + 1);
        s.put_many(vec![item.clone(), s.get(STICKY).unwrap().clone()])
            .unwrap();
        assert_eq!(s.rev(PAGE), first + 2);
        s.remove_many(&[PAGE.into()]).unwrap();
        assert_eq!(s.rev(PAGE), first + 3);
        let change = s.drain_changes().unwrap();
        assert_eq!(change.ids.len(), 2);
        assert_eq!(change.revs[PAGE], first + 3);
        assert!(s.drain_changes().is_none());
        drop(s);
        let s = Session::unlock(&paths.vault, "throwaway", None).unwrap();
        assert_eq!(s.rev(STICKY), 0);
        assert!(s.revs.is_empty());
    }

    #[test]
    fn leases_are_per_window_cover_containers_and_expire_after_save() {
        let (mut s, _) = fixture();
        s.coordination.editing(PAGE, "main", true);
        assert!(
            save(&mut s, &ai("Codex"), "ai")
                .err()
                .unwrap()
                .to_string()
                .starts_with("person_editing:")
        );
        assert!(s.check_write(&ai("Codex"), NB, None).is_err());
        save(&mut s, &Actor::Person, "person").unwrap();
        s.coordination.editing(PAGE, "peek", true);
        s.coordination.editing(PAGE, "main", false);
        s.coordination.forget_window("main");
        assert!(save(&mut s, &ai("Codex"), "ai").is_err());
        s.coordination.forget_window("peek");
        s.coordination.editing(PAGE, "main", false);
        assert!(save(&mut s, &ai("Codex"), "ai").is_err());
        s.coordination
            .leases
            .get_mut(PAGE)
            .unwrap()
            .insert("main".into(), Some(Instant::now() - Duration::from_secs(1)));
        save(&mut s, &ai("Codex"), "ai").unwrap();
    }

    #[test]
    fn claims_reserve_descendants_and_release_stop_disconnect_and_timeout() {
        let (mut s, _) = fixture();
        let a = ai("Claude");
        let b = ai("Codex");
        s.claim(&a, &[NB.into()]).unwrap();
        assert!(
            save(&mut s, &b, "ai")
                .err()
                .unwrap()
                .to_string()
                .starts_with("claimed_by_other_ai:")
        );
        save(&mut s, &Actor::Person, "person").unwrap();
        s.coordination.release(&a, Some(&[NB.into()]));
        save(&mut s, &b, "ai").unwrap();
        assert!(s.check_write(&a, PAGE, None).is_err());
        let claim = s.coordination.claims.get(PAGE).unwrap();
        assert!(
            !claim.explicit
                && claim.until.duration_since(Instant::now()) <= Duration::from_secs(120)
        );
        // Stop and disconnect both call release(actor, None).
        s.coordination.release(&b, None);
        save(&mut s, &a, "ai 2").unwrap();
        s.coordination.release(&a, None);
        save(&mut s, &b, "ai 3").unwrap();
        s.coordination.claims.get_mut(PAGE).unwrap().until =
            Instant::now() - Duration::from_secs(1);
        save(&mut s, &a, "ai 4").unwrap();
        s.claim(&a, &[PAGE.into()]).unwrap();
        assert!(s.coordination.claims.get(PAGE).unwrap().explicit);
    }

    #[test]
    fn container_claim_activity_renews_the_claim_and_conflicts_are_atomic() {
        let (mut s, _) = fixture();
        let a = ai("Claude");
        let b = ai("Codex");
        s.claim(&a, &[NB.into()]).unwrap();
        s.coordination.claims.get_mut(NB).unwrap().until = Instant::now() + Duration::from_secs(1);
        save(&mut s, &a, "child activity").unwrap();
        assert!(s.coordination.claims[NB].until > Instant::now() + Duration::from_secs(1790));
        assert!(s.claim(&b, &[STICKY.into(), PAGE.into()]).is_err());
        assert!(!s.coordination.claims.contains_key(STICKY));
        assert!(
            s.coordination
                .claim(&b, &[STICKY.into(), NB.into()], false)
                .is_err()
        );
        assert!(!s.coordination.claims.contains_key(STICKY));
        s.coordination.release(&a, None);
        s.claim(&b, &[NB.into()]).unwrap();
        s.coordination.claims.get_mut(NB).unwrap().until = Instant::now() - Duration::from_secs(1);
        s.claim(&a, &[NB.into()]).unwrap();
    }

    #[test]
    fn leased_destination_and_batch_conflict_never_partially_write() {
        let (mut s, _) = fixture();
        s.coordination.editing(SEC, "main", true);
        let child = match s.get(PAGE).unwrap() {
            Item::Page(p) => {
                let mut p = p.clone();
                p.id = new_id();
                Item::Page(p)
            }
            _ => unreachable!(),
        };
        let id = child.id().to_string();
        assert!(s.run_as(&ai("Codex"), |s| s.put(child)).is_err());
        assert!(s.get(&id).is_none());
        let before = s.rev(STICKY);
        let batch = vec![s.get(STICKY).unwrap().clone(), s.get(PAGE).unwrap().clone()];
        assert!(s.run_as(&ai("Codex"), |s| s.put_many(batch)).is_err());
        assert_eq!(s.rev(STICKY), before);
        assert!(s.drain_changes().is_none());
    }

    #[test]
    fn restore_section_asks_before_restoring_its_binned_notebook() {
        let (mut s, paths) = fixture();
        notes::delete_section_core(&mut s, &Actor::Person, &paths, SEC.into()).unwrap();
        notes::delete_notebook_core(&mut s, &Actor::Person, &paths, NB.into()).unwrap();
        assert!(notes::restore_bin_item_core(&mut s, &Actor::Person, SEC.into(), false).is_err());
        assert!(s.in_bin(SEC) && s.in_bin(NB));
        notes::restore_bin_item_core(&mut s, &Actor::Person, SEC.into(), true).unwrap();
        assert!(!s.in_bin(SEC) && !s.in_bin(NB) && !s.in_bin(PAGE));
    }

    #[test]
    fn failed_safety_backup_keeps_notebook_and_section_live() {
        let (mut s, paths) = fixture();
        std::fs::write(&paths.backups, b"throwaway blocker").unwrap();
        assert!(notes::delete_notebook_core(&mut s, &Actor::Person, &paths, NB.into()).is_err());
        assert!(notes::delete_section_core(&mut s, &Actor::Person, &paths, SEC.into()).is_err());
        assert!(!s.in_bin(NB) && !s.in_bin(SEC) && !s.in_bin(PAGE));
    }

    #[test]
    fn pending_person_saves_land_in_the_bin_and_replaced_pictures_survive_until_delete() {
        let (mut s, paths) = fixture();
        let base = s.rev(PAGE);
        let pic = s
            .put_attachment("image/png", b"throwaway previous picture")
            .unwrap();
        save(&mut s, &ai("Codex"), &pic).unwrap();
        s.person_save(PAGE, Some(base), |s| save(s, &Actor::Person, "person text"))
            .unwrap();
        assert_eq!(s.collect_garbage().unwrap(), 0);
        assert!(s.get_attachment(&pic).is_ok());
        notes::delete_notebook_core(&mut s, &Actor::Person, &paths, NB.into()).unwrap();
        save(&mut s, &Actor::Person, "last pending typing").unwrap();
        assert!(save(&mut s, &ai("Codex"), "binned edit").is_err());
        assert!(matches!(s.get(PAGE), Some(Item::Page(p)) if p.text == "last pending typing"));
        notes::empty_trash_core(&mut s, &Actor::Person).unwrap();
        assert!(s.settings().replaced_edits.is_empty());
        assert!(s.get_attachment(&pic).is_err());
    }

    #[test]
    fn stale_sticky_and_database_saves_also_keep_the_previous_ai_value() {
        let (mut s, _) = fixture();
        let base = s.rev(STICKY);
        stickies::save_sticky_core(
            &mut s,
            &ai("Codex"),
            STICKY.into(),
            "AI sticky".into(),
            "AI sticky".into(),
        )
        .unwrap();
        s.person_save(STICKY, Some(base), |s| {
            stickies::save_sticky_core(
                s,
                &Actor::Person,
                STICKY.into(),
                "person sticky".into(),
                "person sticky".into(),
            )
        })
        .unwrap();
        let base = s.rev(DB);
        let Item::Database(mut db) = s.get(DB).unwrap().clone() else {
            unreachable!()
        };
        db.title = "AI database".into();
        s.run_as(&ai("Claude"), |s| s.put(Item::Database(db.clone())))
            .unwrap();
        db.title = "person database".into();
        s.person_save(DB, Some(base), |s| s.put(Item::Database(db)))
            .unwrap();
        let settings = s.settings();
        assert_eq!(settings.replaced_edits.len(), 2);
        assert!(
            settings
                .replaced_edits
                .iter()
                .any(|edit| edit.id == STICKY && edit.previous.contains("AI sticky"))
        );
        assert!(
            settings
                .replaced_edits
                .iter()
                .any(|edit| edit.id == DB && edit.previous.contains("AI database"))
        );
    }

    #[test]
    fn expected_revision_refuses_ai_and_stale_person_preserves_previous_value() {
        let (mut s, paths) = fixture();
        let base = s.rev(PAGE);
        save(&mut s, &ai("Codex"), "AI version").unwrap();
        // A person's metadata command between the AI write and their delayed
        // body save must not erase the fact that an AI version is being replaced.
        notes::set_page_favorite_core(&mut s, &Actor::Person, PAGE.into(), false).unwrap();
        assert!(
            s.write_as(&ai("Codex"), &[PAGE.into()], Some(base), |s| save(
                s,
                &ai("Codex"),
                "wrong"
            ))
            .is_err()
        );
        s.person_save(PAGE, Some(base), |s| {
            save(s, &Actor::Person, "person version")
        })
        .unwrap();
        let settings = s.settings();
        assert_eq!(settings.replaced_edits.len(), 1);
        let previous: Value = serde_json::from_str(&settings.replaced_edits[0].previous).unwrap();
        assert_eq!(previous["text"], "AI version");
        assert_eq!(settings.replaced_edits[0].status, "replaced by your edit");
        drop(s);
        let mut s = Session::unlock(&paths.vault, "throwaway", None).unwrap();
        assert_eq!(s.settings().replaced_edits.len(), 1);
        notes::trash_page_core(&mut s, &Actor::Person, PAGE.into()).unwrap();
        notes::delete_page_core(&mut s, &Actor::Person, PAGE.into()).unwrap();
        assert!(s.settings().replaced_edits.is_empty());
    }

    #[test]
    fn bin_containers_keep_children_pictures_and_checked_backups_and_restore_as_unit() {
        let (mut s, paths) = fixture();
        let pic = s.put_attachment("image/png", b"throwaway picture").unwrap();
        let mut page: Page = match s.get(PAGE).unwrap() {
            Item::Page(p) => p.clone(),
            _ => unreachable!(),
        };
        page.content = pic.clone();
        s.put(Item::Page(page)).unwrap();
        notes::delete_notebook_core(&mut s, &Actor::Person, &paths, NB.into()).unwrap();
        assert!(s.get(PAGE).is_some() && s.get(DB).is_some() && s.get(ROW).is_some());
        assert!(s.in_bin(PAGE) && s.in_bin(ROW));
        assert_eq!(s.collect_garbage().unwrap(), 0);
        assert!(s.get_attachment(&pic).is_ok());
        assert!(std::fs::read_dir(&paths.backups).unwrap().any(|p| {
            p.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("vault-predelete-")
        }));
        let tree =
            serde_json::to_value(notes::list_notes_core(&s, &Actor::Person).unwrap()).unwrap();
        assert_eq!(tree["notebooks"].as_array().unwrap().len(), 0);
        assert_eq!(tree["pages"].as_array().unwrap().len(), 0);
        assert_eq!(tree["databases"].as_array().unwrap().len(), 0);
        let hits =
            serde_json::to_value(tools::search_core(&s, &Actor::Person, "exact".into()).unwrap())
                .unwrap();
        assert!(
            hits.as_array()
                .unwrap()
                .iter()
                .all(|hit| hit["kind"] == "sticky")
        );
        notes::restore_page_core(&mut s, &Actor::Person, NB.into()).unwrap();
        assert!(!s.in_bin(PAGE) && !s.in_bin(ROW));
        notes::delete_section_core(&mut s, &Actor::Person, &paths, SEC.into()).unwrap();
        notes::delete_notebook_core(&mut s, &Actor::Person, &paths, NB.into()).unwrap();
        assert!(notes::restore_page_core(&mut s, &Actor::Person, SEC.into()).is_err());
        notes::restore_page_core(&mut s, &Actor::Person, NB.into()).unwrap();
        notes::restore_page_core(&mut s, &Actor::Person, SEC.into()).unwrap();
        assert!(!s.in_bin(PAGE));
        notes::delete_notebook_core(&mut s, &Actor::Person, &paths, NB.into()).unwrap();
        notes::empty_trash_core(&mut s, &Actor::Person).unwrap();
        for id in [NB, SEC, PAGE, DB, ROW] {
            assert!(s.get(id).is_none());
        }
        assert!(s.get_attachment(&pic).is_err());
    }

    #[test]
    fn old_records_default_to_live_and_sticky_bin_closes_and_restores() {
        let (mut s, _) = fixture();
        for id in [NB, SEC, STICKY] {
            assert!(!s.in_bin(id));
        }
        stickies::delete_sticky_core(&mut s, &Actor::Person, STICKY.into()).unwrap();
        let st: Sticky = match s.get(STICKY).unwrap() {
            Item::Sticky(st) => st.clone(),
            _ => unreachable!(),
        };
        assert!(!st.open && st.trashed_at.is_some());
        assert!(
            stickies::list_stickies_core(&s, &Actor::Person)
                .unwrap()
                .is_empty()
        );
        notes::restore_page_core(&mut s, &Actor::Person, STICKY.into()).unwrap();
        assert!(!s.in_bin(STICKY));
        stickies::delete_stickies_core(&mut s, &Actor::Person, vec![STICKY.into()]).unwrap();
        notes::empty_trash_core(&mut s, &Actor::Person).unwrap();
        assert!(s.get(STICKY).is_none());
    }

    #[test]
    fn with_mut_emits_one_batch_and_lock_drops_coordination() {
        let (mut s, paths) = fixture();
        s.claim(&ai("Codex"), &[PAGE.into()]).unwrap();
        let state = crate::state::AppState::new(paths);
        let events = Arc::new(Mutex::new(Vec::new()));
        let emitted = events.clone();
        state.bind_events(move |changes| { emitted.lock().unwrap().push(changes); Ok(()) }, |_| {});
        state.set_session(s);
        state
            .with_mut(|s| {
                save(s, &Actor::Person, "first")?;
                save(s, &Actor::Person, "second")?;
                Ok(())
            })
            .unwrap();
        let batch = events.lock().unwrap();
        assert_eq!(batch.len(), 1);
        assert_eq!(batch[0].ids, vec![PAGE]);
        assert_eq!(batch[0].by, "person");
        assert!(batch[0].tree);
        drop(batch);
        state.with_mut(|_| Ok(())).unwrap();
        assert_eq!(events.lock().unwrap().len(), 1);
        let old = state.take_session().unwrap();
        let paths = &state.paths;
        drop(old);
        state.set_session(Session::unlock(&paths.vault, "throwaway", None).unwrap());
        state
            .with_mut(|s| save(s, &ai("Claude"), "after unlock"))
            .unwrap();
    }
}
