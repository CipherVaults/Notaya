//! "Import pages…": Markdown, HTML, Word and PDF files, a folder of them, and ZIP files (for
//! example a Notion export) become ONE new notebook (see `crate::importers`). A ZIP of several
//! notebooks made by Notaya's own export becomes one new notebook for each; the summary lists
//! them all, and the app opens the first.
//!
//! The UI sends only which kind of source to import ("files" or "folder"). The dialog is opened
//! here, the files are read and converted here, and the UI gets back a summary. Nothing is ever
//! fetched from the network, links on disk are not followed, and the vault is changed in two
//! steps: the pictures are stored first, then the notebook, its sections and pages are saved in
//! one transaction (all or nothing). If anything fails, the pictures stored for this import are
//! taken back and nothing else has changed.
//!
//! The person can cancel from the dialog (`import_cancel`), and locking the vault cancels too.
//! The import also stops by itself after 15 minutes. Both are noticed inside the converters'
//! loops (see `importers::Control`), so a cancel takes effect within moments, and ends like any
//! other failure: the pictures stored so far are taken back and nothing is added.
//! While the pictures wait to be attached to the new pages, the vault's cleanup
//! (`collect_garbage`) leaves them alone.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::Deserialize;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::error::{Error, Result};
use crate::commands::notion_db;
use crate::importers::plan::{self, ImportSummary};
use crate::importers::source::Vfs;
use crate::importers::{Control, MAX_TIME};
use crate::model::{now_ms, Item};
use crate::state::AppState;
use crate::vault::Session;

#[cfg(test)]
#[path = "../importers/n1_tests.rs"]
mod n1_tests;

/// Most files one pick may hold.
const MAX_PICKED: usize = 5_000;
/// Conversion nests function calls (documents inside documents), so it runs on a thread with a
/// big stack instead of the 2 MB default.
const STACK: usize = 128 * 1024 * 1024;

/// The cancel flag of the import that is running (there is at most one).
static RUNNING: Mutex<Option<Arc<AtomicBool>>> = Mutex::new(None);

/// Registers the running import; unregisters it when dropped.
struct Running(Arc<AtomicBool>);

impl Running {
    fn start() -> Result<Running> {
        let mut slot = RUNNING.lock().unwrap_or_else(|e| e.into_inner());
        if slot.is_some() {
            return Err(Error::invalid("Another import is still running."));
        }
        let flag = Arc::new(AtomicBool::new(false));
        *slot = Some(flag.clone());
        Ok(Running(flag))
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        *RUNNING.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
}

/// Asks the import in progress, if there is one, to stop. True when there was one.
pub fn cancel_running() -> bool {
    match RUNNING.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
        Some(flag) => {
            flag.store(true, Ordering::SeqCst);
            true
        }
        None => false,
    }
}

/// The Cancel button of the import dialog.
#[tauri::command]
pub fn import_cancel() -> bool {
    cancel_running()
}

/// Said when the vault was locked while pictures were being stored.
const LOCKED: &str = "The notes were locked while the import was running, so it was stopped. Nothing was added.";

#[derive(Deserialize)]
pub struct ImportRequest {
    /// "files" (Markdown, HTML, Word, PDF and ZIP files, any mix) or "folder".
    source: String,
}

/// What the import needs from the vault. `AppState` is the real one; tests use a plain session.
pub trait VaultAccess: Sync {
    fn with<T>(&self, f: impl FnOnce(&Session) -> Result<T>) -> Result<T>;
    fn with_mut<T>(&self, f: impl FnOnce(&mut Session) -> Result<T>) -> Result<T>;
}

impl VaultAccess for AppState {
    fn with<T>(&self, f: impl FnOnce(&Session) -> Result<T>) -> Result<T> {
        AppState::with(self, f)
    }
    fn with_mut<T>(&self, f: impl FnOnce(&mut Session) -> Result<T>) -> Result<T> {
        AppState::with_mut(self, f)
    }
}

/// Refuses Notaya's own files: a `.nota` or `.clf` file, anything inside the data folder, and
/// (for a folder) a folder that holds the data folder, which would pull in its web-view files.
pub fn check_sources(paths: &[PathBuf], data_dir: &Path, folder: bool) -> Result<()> {
    if paths.is_empty() || paths.len() > MAX_PICKED {
        return Err(Error::invalid(format!("Choose between 1 and {MAX_PICKED} files.")));
    }
    let data = crate::export::resolve(data_dir);
    for p in paths {
        if crate::filenames::path_has_vault_ext(p) {
            return Err(Error::invalid(
                "That is a Notaya notes file (.nota or .clf). It cannot be imported here; use Restore from a backup on the lock screen instead.",
            ));
        }
        let real = crate::export::resolve(p);
        if real.starts_with(&data) || (folder && data.starts_with(&real)) {
            return Err(Error::invalid(
                "That is, or contains, the folder where Notaya keeps your notes and backups. Choose another folder or other files.",
            ));
        }
    }
    Ok(())
}

/// The name the new notebook is given: the folder, or the one file chosen.
fn name_hint(paths: &[PathBuf], folder: bool) -> Option<String> {
    let first = paths.first()?;
    if folder {
        first.file_name().map(|n| n.to_string_lossy().into_owned())
    } else if paths.len() == 1 {
        first.file_stem().map(|n| n.to_string_lossy().into_owned())
    } else {
        None
    }
}

fn next_notebook_order(s: &Session) -> i64 {
    s.items()
        .filter_map(|i| match i {
            Item::Notebook(n) => Some(n.order),
            _ => None,
        })
        .max()
        .map_or(0, |m| m + 1)
}

/// Runs `f` on a thread with a big stack (it may borrow from the caller).
fn on_big_stack<T: Send>(f: impl FnOnce() -> T + Send) -> Result<T> {
    std::thread::scope(|scope| {
        let handle = std::thread::Builder::new()
            .stack_size(STACK)
            .spawn_scoped(scope, f)
            .map_err(|e| Error::Other(format!("The import could not start ({e}).")))?;
        handle.join().map_err(|_| Error::Other("The import stopped unexpectedly. Nothing was imported.".into()))
    })
}

/// The whole import of the chosen paths into the vault. Nothing is written when it fails, is
/// cancelled through `ctl` or runs out of time.
pub fn import_from<V: VaultAccess>(vault: &V, paths: &[PathBuf], folder: bool, data_dir: &Path, now: i64, ctl: &Control) -> Result<ImportSummary> {
    import_from_with(vault, paths, folder, data_dir, now, ctl, &|_| Ok(()))
}

/// Like `import_from`; `before_first_database` runs (inside the saving step) when the import makes the
/// vault's first database from the tables of a Notion export, so the app can make its safety backup.
pub fn import_from_with<V: VaultAccess>(
    vault: &V,
    paths: &[PathBuf],
    folder: bool,
    data_dir: &Path,
    now: i64,
    ctl: &Control,
    before_first_database: &(dyn Fn(&Session) -> Result<()> + Sync),
) -> Result<ImportSummary> {
    check_sources(paths, data_dir, folder)?;
    let hint = name_hint(paths, folder);
    on_big_stack(|| -> Result<ImportSummary> {
        let mut vfs = if folder { Vfs::from_folder(&paths[0]) } else { Vfs::from_files(paths) }.map_err(Error::invalid)?;
        // A picture is never read from Notaya's own data folder, whatever a document points at.
        vfs.protect(&crate::export::resolve(data_dir));
        let mut plan = plan::build(&vfs, ctl, hint.as_deref(), now).map_err(Error::invalid)?;
        // Pictures first: each is stored on its own, so the vault is not held while files are read.
        let mut stored: Vec<String> = Vec::new();
        let result = (|| -> Result<ImportSummary> {
            let urls = plan
                .store_images(&vfs, ctl, &mut |mime, bytes| {
                    let last = stored.last().cloned();
                    let id = vault
                        .with_mut(|s| {
                            // The pictures stored so far are held in this unlocked session. If a
                            // lock came in between they are not, and the import must not go on.
                            if last.is_some_and(|l| !s.holds(std::slice::from_ref(&l))) {
                                return Err(Error::Other(LOCKED.into()));
                            }
                            let id = s.put_attachment(mime, bytes)?;
                            s.hold_attachment(&id);
                            Ok(id)
                        })
                        .map_err(|e| e.to_string())?;
                    stored.push(id.clone());
                    Ok(id)
                })
                .map_err(Error::Other)?;
            let order = vault.with(|s| Ok(next_notebook_order(s)))?;
            // The tables of a Notion export: each becomes a database (the pages importer leaves them out), and the
            // row pages are folded into the rows.
            let (tables, _, table_notes) = notion_db::read_tables(&vfs, &plan.csv_paths);
            let row_pages = notion_db::prepare_pages(&tables, &mut plan.pages);
            let (mut items, mut summary) = plan.items(ctl, &urls, now, order).map_err(Error::invalid)?;
            // Pending CSV entries are replaced with final outcomes, never counted as left out.
            summary.skipped.retain(|k| !plan.csv_paths.contains(&k.item));
            summary.notes.extend(table_notes);
            let mut imported_csv = Vec::new();
            // Last chance to stop: after this the notebooks are saved in one step.
            plan::check(ctl).map_err(Error::invalid)?;
            // The notebooks, sections and pages in one transaction; the pictures are then in saved
            // pages and no longer need holding.
            vault.with_mut(|s| {
                if !s.holds(&stored) {
                    return Err(Error::Other(LOCKED.into()));
                }
                if !tables.is_empty() {
                    let made = notion_db::import_databases(&tables, &row_pages, &items, &plan.csv_section_ids(&items), s, now, before_first_database)?;
                    if made.databases > 0 {
                        drop_row_pages(&mut items, &made.removed_pages, &plan.generated_page_ids());
                        summary.pages = items.iter().filter(|i| matches!(i, Item::Page(_))).count();
                    }
                    imported_csv = made.covered;
                    let rows = made.items.iter().filter(|i| matches!(i, Item::Row(_))).count();
                    summary.notes.push(format!("Imported {} databases and {rows} rows.", made.databases));
                    summary.notes.extend(made.notes);
                    items.extend(made.items);
                }
                // Importing the same export twice makes a second notebook; say so, so it is not mistaken for an update.
                let existing: Vec<String> = s.items().filter_map(|i| if let Item::Notebook(n) = i { Some(n.name.trim().to_lowercase()) } else { None }).collect();
                for it in &items {
                    if let Item::Notebook(n) = it {
                        if existing.contains(&n.name.trim().to_lowercase()) {
                            summary.notes.push(format!("A notebook named \"{}\" already exists. This import added another one next to it, so everything in it is now twice in your notes. Delete one of them if this was the same export.", n.name));
                        }
                    }
                }
                s.put_many(items)?;
                s.release_attachments(&stored);
                Ok(())
            })?;
            let failed_csv: Vec<_> = plan.csv_paths.iter().filter(|p| !imported_csv.contains(p)).collect();
            let room = plan::MAX_LISTED.saturating_sub(summary.skipped.len());
            for path in failed_csv.iter().take(room) {
                summary.skipped.push(crate::importers::Skipped {
                    item: (*path).clone(),
                    reason: "The database view could not be imported; see the import notes.".into(),
                });
            }
            if failed_csv.len() > room {
                summary.notes.push(format!("{} more database view files were not imported than the list shows.", failed_csv.len() - room));
            }
            Ok(summary)
        })();
        if result.is_err() && !stored.is_empty() {
            // Take back exactly the pictures this import stored. (If the vault was locked
            // meanwhile this cannot run; they are then ordinary unused pictures that the next
            // cleanup removes.)
            let _ = vault.with_mut(|s| {
                let r = s.delete_attachments(&stored);
                s.release_attachments(&stored);
                r
            });
        }
        result
    })?
}

/// Takes out the pages explicitly moved into rows, then only empty generated folder parents.
/// An empty source document is still a genuine page and must survive ancestor cleanup.
fn drop_row_pages(items: &mut Vec<Item>, gone: &std::collections::HashSet<String>, generated: &std::collections::HashSet<String>) {
    use std::collections::HashSet;
    let mut held: HashSet<String> = items
        .iter()
        .filter_map(|i| match i {
            Item::Page(p) if gone.contains(&p.id) && !p.parent.is_empty() => Some(p.parent.clone()),
            _ => None,
        })
        .collect();
    items.retain(|i| !matches!(i, Item::Page(p) if gone.contains(&p.id)));
    for _ in 0..8 {
        let parents: HashSet<String> = items.iter().filter_map(|i| if let Item::Page(p) = i { Some(p.parent.clone()) } else { None }).filter(|p| !p.is_empty()).collect();
        let empty: HashSet<String> = items
            .iter()
            .filter_map(|i| match i {
                Item::Page(p) if generated.contains(&p.id) && held.contains(&p.id) && p.text.trim().is_empty() && !parents.contains(&p.id) => Some(p.id.clone()),
                _ => None,
            })
            .collect();
        if empty.is_empty() {
            break;
        }
        held = items.iter().filter_map(|i| match i { Item::Page(p) if empty.contains(&p.id) && !p.parent.is_empty() => Some(p.parent.clone()), _ => None }).collect();
        items.retain(|i| !matches!(i, Item::Page(p) if empty.contains(&p.id)));
    }
}

/// Asks what to import, then imports it as a new notebook. `None` if the dialog was cancelled.
/// Runs on a worker thread: the dialog waits for the person, and a big import takes a while.
#[tauri::command(async)]
pub fn import_notes(app: AppHandle, state: State<'_, AppState>, request: ImportRequest) -> Result<Option<ImportSummary>> {
    import_notes_person(app, state, request)
}

fn import_notes_person(app: AppHandle, state: State<'_, AppState>, request: ImportRequest) -> Result<Option<ImportSummary>> {
    // Registered before the dialog opens, so a Cancel (or a lock) during the pick is seen too.
    let running = Running::start()?;
    let (paths, folder): (Vec<PathBuf>, bool) = match request.source.as_str() {
        "files" => {
            let picked = app
                .dialog()
                .file()
                .set_title("Import pages")
                .add_filter("Documents and ZIP files", &["md", "markdown", "txt", "html", "htm", "docx", "pdf", "zip"])
                .blocking_pick_files();
            let Some(picked) = picked else { return Ok(None) };
            (picked.into_iter().filter_map(|f| f.into_path().ok()).collect(), false)
        }
        "folder" => {
            let Some(dir) = app.dialog().file().set_title("Choose a folder to import").blocking_pick_folder().and_then(|f| f.into_path().ok()) else {
                return Ok(None);
            };
            (vec![dir], true)
        }
        _ => return Err(Error::invalid("Unknown import choice.")),
    };
    if paths.is_empty() || running.0.load(Ordering::SeqCst) {
        return Ok(None);
    }
    let ctl = Control::new(MAX_TIME, running.0.clone());
    let safety = |s: &Session| crate::backup::make_safety_backup(&state.paths, s).map(|_| ()).map_err(|e| crate::backup::safety_failure(&e));
    match import_from_with(&*state, &paths, folder, &state.paths.data, now_ms(), &ctl, &safety) {
        Ok(summary) => Ok(Some(summary)),
        // The person pressed Cancel: nothing happened, and that is not an error. A lock that
        // stopped the import is told as such.
        Err(e) if e.to_string() == plan::CANCELLED => {
            if state.is_unlocked() {
                Ok(None)
            } else {
                Err(Error::Other(LOCKED.into()))
            }
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::types::{Database, PropKind, Row};
    use crate::importers::zip;
    use serde_json::Value;
    use crate::model::{new_id, Page};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::sync::Mutex;

    struct Test {
        session: Mutex<Session>,
    }

    impl VaultAccess for Test {
        fn with<T>(&self, f: impl FnOnce(&Session) -> Result<T>) -> Result<T> {
            f(&self.session.lock().unwrap())
        }
        fn with_mut<T>(&self, f: impl FnOnce(&mut Session) -> Result<T>) -> Result<T> {
            f(&mut self.session.lock().unwrap())
        }
    }

    fn temp() -> PathBuf {
        let d = std::env::temp_dir().join(format!("notaya-import-cmd-{}", new_id()));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn vault(dir: &Path) -> Test {
        std::fs::create_dir_all(dir).unwrap();
        Test { session: Mutex::new(Session::create_open(&dir.join("vault.nota")).unwrap()) }
    }

    const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\0\x01\0\0\0\x01";

    fn pages(v: &Test) -> Vec<Page> {
        v.with(|s| Ok(s.items().filter_map(|i| if let Item::Page(p) = i { Some(p.clone()) } else { None }).collect())).unwrap()
    }

    fn count(v: &Test) -> usize {
        v.with(|s| Ok(s.items().count())).unwrap()
    }

    #[test]
    fn the_tables_of_a_notion_export_become_databases() {
        let root = temp();
        let src = root.join("export");
        let id = "98b9830964384ed4b08e1c3de9ace4e9";
        std::fs::create_dir_all(src.join(format!("Space/Projects {id}"))).unwrap();
        std::fs::write(src.join(format!("Space/Projects {id}.csv")), "Name,Status\nOnly the view,Done\n").unwrap();
        std::fs::write(src.join(format!("Space/Projects {id}_all.csv")), "Name,Status,Budget\nAlpha,Done,10\nBeta,In progress,20\n\u{645}\u{631}\u{62d}\u{628}\u{627},Done,30\n").unwrap();
        std::fs::write(src.join(format!("Space/Projects {id}/Alpha.md")), "# Alpha\n\nStatus: Done\n").unwrap();
        std::fs::write(src.join("Space/Handbook.md"), "# Handbook\n\nHello\n").unwrap();
        let v = vault(&root.join("data"));
        let backed_up = std::sync::atomic::AtomicUsize::new(0);
        let backup = |_: &Session| -> Result<()> {
            backed_up.fetch_add(1, Ordering::SeqCst);
            Ok(())
        };
        let flag = Arc::new(AtomicBool::new(false));
        let summary = import_from_with(&v, &[src.clone()], true, &root.join("data"), 1, &flag_control(&flag), &backup).unwrap();
        let (dbs, rows): (Vec<_>, Vec<_>) = v
            .with(|s| {
                let d: Vec<_> = s.items().filter_map(|i| if let Item::Database(d) = i { Some((d.title.clone(), d.props.len())) } else { None }).collect();
                let r: Vec<_> = s.items().filter_map(|i| if let Item::Row(r) = i { Some(r.title.clone()) } else { None }).collect();
                Ok((d, r))
            })
            .unwrap();
        assert_eq!(dbs.len(), 1, "one database, from the _all file");
        assert_eq!(dbs[0].0, "Projects");
        assert!(dbs[0].1 >= 3, "a column for each header");
        assert_eq!(rows.len(), 3, "every row of the _all file, not the view's");
        assert!(rows.iter().any(|t| t == "\u{645}\u{631}\u{62d}\u{628}\u{627}"));
        assert_eq!(backed_up.load(Ordering::SeqCst), 1, "a backup before the first database");
        assert!(!summary.skipped.iter().any(|k| k.item.ends_with(".csv")), "{:?}", summary.skipped);
        assert!(summary.notes.iter().any(|n| n.contains("Database \"Projects\": 3 rows")), "{:?}", summary.notes);
        assert!(!pages(&v).is_empty());
    }

    #[test]
    fn notion_rows_become_database_rows_with_relations_status_and_numbers() {
        let root = temp();
        let src = root.join("export");
        let (pdb, tdb) = ("98b9830964384ed4b08e1c3de9ace4e9", "26d33f50cab44d2cb56b63471a0d5540");
        let (a, b, t1, t2) = ("40d90e048b60816bb6ebddfb2c2daf50", "40d90e048b60814b8896d436383181dd", "40d90e048b6081abaf32ea800580a648", "40d90e048b608125bfa4c8225a33e807");
        let space = src.join("Space");
        std::fs::create_dir_all(space.join(format!("Projects {pdb}"))).unwrap();
        std::fs::create_dir_all(space.join(format!("Tasks {tdb}"))).unwrap();
        let projects = format!(
            "Name,Status,Budget,Progress,Tags,Tasks,Priority\n\
             Alpha,In progress,\"$12,000.00\",60%,\"a, b\",\"Write it (Tasks%20{tdb}/Write%20it%20{t1}.md), Fix (Tasks%20{tdb}/Fix%20{t2}.md)\",High\n\
             Beta,Done,\"$3,000.00\",100%,a,,Low\n\
             Gamma,Planning,\"$1,000.00\",0%,b,,High\n\
             Delta,Blocked,$5.00,10%,\"a, b\",,Low\n"
        );
        std::fs::write(space.join(format!("Projects {pdb}_all.csv")), &projects).unwrap();
        std::fs::write(space.join(format!("Projects {pdb}.csv")), "Name\nAlpha\n").unwrap();
        let tasks = format!(
            "Name,Status,Project,Estimate\n\
             Write it,Done,\"Alpha (Projects%20{pdb}/Alpha%20{a}.md)\",3\n\
             Fix,Doing,\"Alpha (Projects%20{pdb}/Alpha%20{a}.md)\",2\n\
             Lonely,Doing,,1\n\
             Extra,Done,,1\n"
        );
        std::fs::write(space.join(format!("Tasks {tdb}_all.csv")), &tasks).unwrap();
        let row = |title: &str, _id: &str, props: &str, body: &str| format!("# {title}\n\n{props}\n\n{body}\n");
        std::fs::write(
            space.join(format!("Projects {pdb}/Alpha {a}.md")),
            row("Alpha", a, "Status: In progress\nBudget: $12,000.00\nProgress: 60%\nPriority: High", "The body of Alpha has a **bold** word."),
        )
        .unwrap();
        std::fs::write(space.join(format!("Projects {pdb}/Beta {b}.md")), row("Beta", b, "Status: Done\nPriority: Low", "")).unwrap();
        std::fs::write(space.join(format!("Tasks {tdb}/Write it {t1}.md")), row("Write it", t1, "Status: Done\nEstimate: 3", "Write the importer.")).unwrap();
        std::fs::write(space.join("Handbook.md"), "# Handbook\n\nStays a page.\n").unwrap();
        let v = vault(&root.join("data"));
        let flag = Arc::new(AtomicBool::new(false));
        let summary = import_from_with(&v, &[src.clone()], true, &root.join("data"), 1, &flag_control(&flag), &|_| Ok(())).unwrap();
        let (dbs, rows): (Vec<Database>, Vec<Row>) = v
            .with(|s| {
                let d = s.items().filter_map(|i| if let Item::Database(d) = i { Some(d.clone()) } else { None }).collect();
                let r = s.items().filter_map(|i| if let Item::Row(r) = i { Some(r.clone()) } else { None }).collect();
                Ok((d, r))
            })
            .unwrap();
        let p = dbs.iter().find(|d| d.title == "Projects").expect("Projects");
        let t = dbs.iter().find(|d| d.title == "Tasks").expect("Tasks");
        let prop = |d: &Database, n: &str| d.props.iter().find(|x| x.name == n).unwrap_or_else(|| panic!("no property {n}")).clone();
        let status = prop(p, "Status");
        assert_eq!(status.kind, PropKind::Status);
        assert!(status.options.iter().any(|o| o.name == "Done" && o.group == "complete"));
        assert!(status.options.iter().any(|o| o.name == "In progress" && o.group == "inProgress"));
        assert_eq!(prop(p, "Budget").kind, PropKind::Number);
        assert_eq!(prop(p, "Budget").number_format, "dollar");
        assert_eq!(prop(p, "Progress").number_format, "percent");
        assert_eq!(prop(p, "Tags").kind, PropKind::MultiSelect);
        assert_eq!(prop(p, "Priority").kind, PropKind::Select);
        let tasks_prop = prop(p, "Tasks");
        let project_prop = prop(t, "Project");
        assert_eq!(tasks_prop.kind, PropKind::Relation);
        assert_eq!(tasks_prop.relation_db, t.id);
        assert_eq!(project_prop.kind, PropKind::Relation);
        assert_eq!(project_prop.relation_db, p.id);
        assert_eq!(tasks_prop.relation_back, project_prop.id, "the two columns are one two-way relation");
        assert_eq!(project_prop.relation_back, tasks_prop.id);
        let cell = |r: &Row, pid: &str| -> Value { serde_json::from_str::<Value>(&r.cells).unwrap().get(pid).cloned().unwrap_or(Value::Null) };
        let alpha = rows.iter().find(|r| r.db == p.id && r.title == "Alpha").unwrap();
        assert_eq!(cell(alpha, &tasks_prop.id).as_array().map(|a| a.len()), Some(2));
        assert_eq!(cell(alpha, &prop(p, "Progress").id), serde_json::json!(0.6));
        assert_eq!(cell(alpha, &prop(p, "Budget").id), serde_json::json!(12000.0));
        let write = rows.iter().find(|r| r.db == t.id && r.title == "Write it").unwrap();
        assert_eq!(cell(write, &project_prop.id), serde_json::json!([alpha.id.clone()]));
        assert!(alpha.text.contains("The body of Alpha"), "{:?}", alpha.text);
        assert!(!alpha.text.contains("Status:"), "the property lines are columns now: {:?}", alpha.text);
        assert!(write.text.contains("Write the importer."));
        let titles: Vec<String> = pages(&v).iter().map(|p| p.title.clone()).collect();
        assert!(titles.iter().any(|t| t == "Handbook"), "{titles:?}");
        assert!(!titles.iter().any(|t| t == "Alpha" || t == "Beta" || t == "Write it" || t == "Projects" || t == "Tasks"), "no row pages and no empty folder pages: {titles:?}");
        assert!(summary.notes.iter().any(|n| n.contains("Relation")), "{:?}", summary.notes);
    }

    #[test]
    fn a_flat_notion_export_still_moves_row_pages_into_rows_and_links_relations() {
        let root = temp();
        let src = root.join("export");
        std::fs::create_dir_all(&src).unwrap();
        let (pdb, tdb) = ("98b9830964384ed4b08e1c3de9ace4e9", "26d33f50cab44d2cb56b63471a0d5540");
        let (a, t1, t2) = ("40d90e048b60816bb6ebddfb2c2daf50", "40d90e048b6081abaf32ea800580a648", "40d90e048b608125bfa4c8225a33e807");
        std::fs::write(
            src.join(format!("Projects {pdb}_all.csv")),
            format!("Name,Status,Tasks\nAlpha,Done,\"Write it (Write%20it%20{t1}.csv), Fix (Fix%20{t2}.csv)\"\nBeta,Done,\n"),
        )
        .unwrap();
        std::fs::write(
            src.join(format!("Tasks {tdb}_all.csv")),
            format!("Name,Status,Project\nWrite it,Done,\"Alpha (Alpha%20{a}.csv)\"\nFix,Doing,\"Alpha (Alpha%20{a}.csv)\"\n"),
        )
        .unwrap();
        std::fs::write(src.join(format!("Alpha {a}.md")), "# Alpha\n\nStatus: Done\nTasks: Write it, Fix\n\nBody of Alpha.\n").unwrap();
        std::fs::write(src.join(format!("Write it {t1}.md")), "# Write it\n\nStatus: Done\nProject: Alpha\n\nBody of the task.\n").unwrap();
        std::fs::write(src.join("Handbook.md"), "# Handbook\n\nStays a page.\n").unwrap();
        let v = vault(&root.join("data"));
        let flag = Arc::new(AtomicBool::new(false));
        import_from_with(&v, &[src.clone()], true, &root.join("data"), 1, &flag_control(&flag), &|_| Ok(())).unwrap();
        let (dbs, rows): (Vec<Database>, Vec<Row>) = v
            .with(|s| {
                let d = s.items().filter_map(|i| if let Item::Database(d) = i { Some(d.clone()) } else { None }).collect();
                let r = s.items().filter_map(|i| if let Item::Row(r) = i { Some(r.clone()) } else { None }).collect();
                Ok((d, r))
            })
            .unwrap();
        let p = dbs.iter().find(|d| d.title == "Projects").expect("Projects");
        let t = dbs.iter().find(|d| d.title == "Tasks").expect("Tasks");
        let tasks_prop = p.props.iter().find(|x| x.name == "Tasks").unwrap();
        assert_eq!(tasks_prop.kind, PropKind::Relation, "the .csv links are relation links");
        assert_eq!(tasks_prop.relation_db, t.id);
        let alpha = rows.iter().find(|r| r.db == p.id && r.title == "Alpha").unwrap();
        assert!(alpha.text.contains("Body of Alpha"), "{:?}", alpha.text);
        let write = rows.iter().find(|r| r.db == t.id && r.title == "Write it").unwrap();
        assert!(write.text.contains("Body of the task."), "{:?}", write.text);
        let titles: Vec<String> = pages(&v).iter().map(|p| p.title.clone()).collect();
        assert!(titles.iter().any(|t| t == "Handbook"), "{titles:?}");
        assert!(!titles.iter().any(|t| t == "Alpha" || t == "Write it"), "row pages are not kept as pages: {titles:?}");
    }

    #[test]
    fn markdown_csv_and_html_files_together_complete_each_other_into_database_rows() {
        let root = temp();
        let src = root.join("export");
        std::fs::create_dir_all(&src).unwrap();
        let (pdb, a, b) = ("98b9830964384ed4b08e1c3de9ace4e9", "40d90e048b60816bb6ebddfb2c2daf50", "40d90e048b60814b8896d436383181dd");
        std::fs::write(src.join(format!("Projects {pdb}_all.csv")), "Name,Status\nAlpha,Done\nBeta,Doing\n").unwrap();
        let html = |title: &str, status: &str, body: &str| {
            format!("<html><head><title>{title}</title></head><body><article class=\"page sans\"><header><h1 class=\"page-title\">{title}</h1><table class=\"properties\"><tbody><tr class=\"property-row\"><th>Status</th><td>{status}</td></tr></tbody></table></header><div class=\"page-body\">{body}</div></article></body></html>")
        };
        // Alpha is in both formats; Beta only as HTML; the database's own HTML page is only a view of the table.
        std::fs::write(src.join(format!("Alpha {a}.md")), "# Alpha\n\nStatus: Done\n\nplain markdown body\n").unwrap();
        std::fs::write(src.join(format!("Alpha {a}.html")), html("Alpha", "Done", "<p><mark class=\"highlight-blue_background\">coloured</mark> alpha body</p>")).unwrap();
        std::fs::write(src.join(format!("Beta {b}.html")), html("Beta", "Doing", "<p>beta html body</p>")).unwrap();
        std::fs::write(src.join(format!("Projects {pdb}.html")), "<html><head><title>Projects</title></head><body><article class=\"page sans\"><header><h1 class=\"page-title\">Projects</h1></header><div class=\"page-body\"><p>table view</p></div></article></body></html>").unwrap();
        let v = vault(&root.join("data"));
        let flag = Arc::new(AtomicBool::new(false));
        let summary = import_from_with(&v, &[src.clone()], true, &root.join("data"), 1, &flag_control(&flag), &|_| Ok(())).unwrap();
        let rows: Vec<Row> = v.with(|s| Ok(s.items().filter_map(|i| if let Item::Row(r) = i { Some(r.clone()) } else { None }).collect())).unwrap();
        let alpha = rows.iter().find(|r| r.title == "Alpha").expect("Alpha is a row");
        let beta = rows.iter().find(|r| r.title == "Beta").expect("Beta is a row");
        assert!(alpha.text.contains("coloured alpha body") && !alpha.text.contains("plain markdown") && !alpha.text.contains("Status:"), "{:?}", alpha.text);
        assert!(alpha.content.contains(crate::importers::doc::HIGHLIGHTS[5]), "the HTML formatting is kept");
        assert!(beta.text.contains("beta html body") && !beta.text.contains("Status"), "the properties table of an HTML-only row page is dropped: {:?}", beta.text);
        let titles: Vec<String> = pages(&v).iter().map(|p| p.title.clone()).collect();
        assert!(!titles.iter().any(|t| t == "Alpha" || t == "Beta" || t == "Projects"), "no row pages, no page for the database view: {titles:?}");
        assert!(summary.notes.iter().any(|n| n.contains("were in both exports")), "{:?}", summary.notes);
    }

    #[test]
    fn two_export_folders_picked_together_make_each_database_once() {
        let root = temp();
        let src = root.join("both");
        let (md, ht) = (src.join("Export md"), src.join("Export html"));
        std::fs::create_dir_all(md.join("Projects")).unwrap();
        std::fs::create_dir_all(ht.join("Projects")).unwrap();
        let (pdb, a) = ("98b9830964384ed4b08e1c3de9ace4e9", "40d90e048b60816bb6ebddfb2c2daf50");
        std::fs::write(md.join(format!("Projects {pdb}_all.csv")), "Name,Status\nAlpha,Done\n").unwrap();
        std::fs::write(ht.join(format!("Projects {pdb}.csv")), "Name,Status\nAlpha,Done\n").unwrap();
        std::fs::write(md.join(format!("Projects/Alpha {a}.md")), "# Alpha\n\nStatus: Done\n\nplain markdown body\n").unwrap();
        std::fs::write(
            ht.join(format!("Projects/Alpha {a}.html")),
            "<html><head><title>Alpha</title></head><body><article class=\"page sans\"><header><h1 class=\"page-title\">Alpha</h1></header><div class=\"page-body\"><p>alpha html body</p></div></article></body></html>",
        )
        .unwrap();
        std::fs::write(ht.join(format!("Projects {pdb}.html")), "<html><head><title>Projects</title></head><body><article class=\"page sans\"><header><h1 class=\"page-title\">Projects</h1></header><div class=\"page-body\"><p>view</p></div></article></body></html>").unwrap();
        let v = vault(&root.join("data"));
        let flag = Arc::new(AtomicBool::new(false));
        import_from_with(&v, &[src.clone()], true, &root.join("data"), 1, &flag_control(&flag), &|_| Ok(())).unwrap();
        let (dbs, rows): (usize, Vec<Row>) = v
            .with(|s| Ok((s.items().filter(|i| matches!(i, Item::Database(_))).count(), s.items().filter_map(|i| if let Item::Row(r) = i { Some(r.clone()) } else { None }).collect())))
            .unwrap();
        assert_eq!(dbs, 1, "one database, not one per export");
        assert_eq!(rows.len(), 1);
        assert!(rows[0].text.contains("alpha html body"), "{:?}", rows[0].text);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn a_folder_of_markdown_becomes_a_notebook_with_pictures() {
        let root = temp();
        let src = root.join("My notes");
        std::fs::create_dir_all(src.join("Work/img")).unwrap();
        std::fs::write(src.join("Work/plan.md"), "# Plan\n\nHello **world**\n\n![shot](img/a.png)\n").unwrap();
        std::fs::write(src.join("Work/img/a.png"), PNG).unwrap();
        std::fs::write(src.join("readme.md"), "Just text").unwrap();
        let v = vault(&root.join("data"));
        let summary = import_from(&v, &[src.clone()], true, &root.join("data"), 1_700_000_000_000, &Control::standard()).unwrap();
        assert_eq!(summary.notebook_name, "My notes");
        assert_eq!((summary.sections, summary.pages, summary.images), (2, 2, 1));
        let ps = pages(&v);
        let plan_page = ps.iter().find(|p| p.title == "Plan").unwrap();
        // The picture is stored and shown with the editor's address.
        let url = plan_page.content.split("http://clatt.localhost/").nth(1).unwrap().split('"').next().unwrap().to_string();
        let (mime, bytes) = v.with(|s| s.get_attachment(&url)).unwrap();
        assert_eq!((mime.as_str(), bytes.as_slice()), ("image/png", PNG));
        assert!(plan_page.text.contains("Hello world"));
        // The notebook, its sections and the pages are in the vault.
        v.with(|s| {
            let n = s.items().filter(|i| matches!(i, Item::Notebook(_))).count();
            let sec = s.items().filter(|i| matches!(i, Item::Section(_))).count();
            assert_eq!((n, sec), (1, 2));
            Ok(())
        })
        .unwrap();
        // Importing again makes a second notebook and leaves the first alone.
        import_from(&v, &[src], true, &root.join("data"), 1_700_000_000_001, &Control::standard()).unwrap();
        assert_eq!(pages(&v).len(), 4);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn a_mixed_pick_of_markdown_word_and_zip_files() {
        let root = temp();
        let picks = root.join("picked");
        std::fs::create_dir_all(&picks).unwrap();
        std::fs::write(picks.join("one.md"), "# One\n\nfirst").unwrap();
        std::fs::write(picks.join("two.docx"), crate::importers::docx::tests::sample_docx()).unwrap();
        let notion = zip::build(
            &[
                ("Export-1/Top 0123456789abcdef0123456789abcdef.md", b"# Top\n\n[Child](Top%200123456789abcdef0123456789abcdef/Child%2011111111111111111111111111111111.md)"),
                ("Export-1/Top 0123456789abcdef0123456789abcdef/Child 11111111111111111111111111111111.md", b"# Child\n\n![p](pic.png)"),
                ("Export-1/Top 0123456789abcdef0123456789abcdef/pic.png", PNG),
            ],
            true,
        );
        std::fs::write(picks.join("Export-abc.zip"), notion).unwrap();
        let v = vault(&root.join("data"));
        let paths = vec![picks.join("one.md"), picks.join("two.docx"), picks.join("Export-abc.zip")];
        let summary = import_from(&v, &paths, false, &root.join("data"), 1_700_000_000_000, &Control::standard()).unwrap();
        assert!(summary.notebook_name.starts_with("Imported 2023-11-"), "{}", summary.notebook_name);
        assert_eq!(summary.sections, 2, "the loose files, and the zip's own top folder");
        assert_eq!(summary.pages, 4, "{:?}", summary.skipped);
        assert_eq!(summary.images, 2, "one from the Word file, one from the zip");
        let ps = pages(&v);
        let top = ps.iter().find(|p| p.title == "Top").unwrap();
        let child = ps.iter().find(|p| p.title == "Child").unwrap();
        assert_eq!(child.parent, top.id, "Notion's folder holds the sub-page");
        assert!(top.content.contains(&format!("\"pageId\":\"{}\"", child.id)), "a link to the sub-page is a page link: {}", top.content);
        assert!(ps.iter().any(|p| p.title == "One") && ps.len() == 4);
        let _ = std::fs::remove_dir_all(root);
    }

    /// Fails the `fail_on`-th change of the vault (counting from 1), like a full disk would.
    struct Failing<'a> {
        inner: &'a Test,
        calls: std::sync::atomic::AtomicUsize,
        fail_on: usize,
    }

    impl VaultAccess for Failing<'_> {
        fn with<T>(&self, f: impl FnOnce(&Session) -> Result<T>) -> Result<T> {
            self.inner.with(f)
        }
        fn with_mut<T>(&self, f: impl FnOnce(&mut Session) -> Result<T>) -> Result<T> {
            let n = self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            if n == self.fail_on {
                return Err(Error::Other("disk full".into()));
            }
            self.inner.with_mut(f)
        }
    }

    #[test]
    fn a_failure_mid_way_leaves_no_partial_notebook_and_no_pictures() {
        let root = temp();
        let src = root.join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("a.md"), "# A\n\n![x](a.png)\n\n![y](b.png)").unwrap();
        std::fs::write(src.join("a.png"), PNG).unwrap();
        std::fs::write(src.join("b.png"), [PNG, b"more"].concat()).unwrap();
        let v = vault(&root.join("data"));
        let before = count(&v);
        // Calls: 1 and 2 store the pictures, 3 saves the notebook.
        for fail_on in [2usize, 3] {
            let f = Failing { inner: &v, calls: 0.into(), fail_on };
            let err = import_from(&f, &[src.join("a.md")], false, &root.join("data"), 1, &Control::standard()).unwrap_err();
            assert_eq!(err.to_string(), "disk full");
            assert_eq!(count(&v), before, "no notebook, section or page when it fails (fail_on {fail_on})");
            let orphans = v.with_mut(|s| s.collect_garbage()).unwrap();
            assert_eq!(orphans, 0, "the pictures it stored were taken back (fail_on {fail_on})");
        }
        // And the same files import fine afterwards.
        let ok = import_from(&v, &[src.join("a.md")], false, &root.join("data"), 1, &Control::standard()).unwrap();
        assert_eq!((ok.pages, ok.images), (1, 2));
        assert_eq!(v.with_mut(|s| s.collect_garbage()).unwrap(), 0, "both pictures are used");
        let _ = std::fs::remove_dir_all(root);
    }

    /// A ZIP of two notebooks in Notaya's own layout, with a picture.
    fn two_notebook_zip(dir: &Path) -> PathBuf {
        let marker = crate::export::marker_text(crate::export::Layout::Notebooks);
        let z = zip::build(
            &[
                ("notaya-export.txt", marker.as_bytes()),
                ("Alpha/Journal/One.md", b"# One\n\n![p](One.assets/image-001.png)\n\n\xE2\x86\x92 [Two](../../Beta/Work/Two.md)"),
                ("Alpha/Journal/One.assets/image-001.png", PNG),
                ("Beta/Work/Two.md", b"# Two\n\nsecond"),
            ],
            true,
        );
        let path = dir.join("2 notebooks.zip");
        std::fs::write(&path, z).unwrap();
        path
    }

    #[test]
    fn a_zip_of_two_notebooks_is_saved_as_two_notebooks_all_or_nothing() {
        let root = temp();
        let file = two_notebook_zip(&root);
        let v = vault(&root.join("data"));
        let before = count(&v);
        // Calls: 1 stores the picture, 2 saves both notebooks in one step.
        for fail_on in [1usize, 2] {
            let f = Failing { inner: &v, calls: 0.into(), fail_on };
            assert!(import_from(&f, &[file.clone()], false, &root.join("data"), 1, &Control::standard()).is_err());
            assert_eq!(count(&v), before, "no notebook at all when it fails (fail_on {fail_on})");
            assert_eq!(v.with_mut(|s| s.collect_garbage()).unwrap(), 0, "the picture was taken back (fail_on {fail_on})");
        }
        // Cancelled: nothing either.
        let flag = Arc::new(AtomicBool::new(true));
        assert_eq!(import_from(&v, &[file.clone()], false, &root.join("data"), 1, &flag_control(&flag)).unwrap_err().to_string(), plan::CANCELLED);
        assert_eq!(count(&v), before);
        // With a cleanup before every change, the picture is still there at the end.
        let hooked = Hooked {
            inner: &v,
            calls: 0.into(),
            hook: Box::new(|_, store| {
                let _ = store.with_mut(|s| s.collect_garbage());
            }),
        };
        let summary = import_from(&hooked, &[file], false, &root.join("data"), 1, &Control::standard()).unwrap();
        let names: Vec<&str> = summary.notebooks.iter().map(|n| n.name.as_str()).collect();
        assert_eq!(names, ["Alpha", "Beta"]);
        assert_eq!((summary.notebook_name.as_str(), summary.sections, summary.pages, summary.images), ("Alpha", 2, 2, 1));
        let ps = pages(&v);
        let one = ps.iter().find(|p| p.title == "One").unwrap();
        let two = ps.iter().find(|p| p.title == "Two").unwrap();
        assert!(one.content.contains(&format!("\"pageId\":\"{}\"", two.id)), "a link across notebooks: {}", one.content);
        v.with(|s| {
            let nbs: Vec<&str> = s.items().filter_map(|i| if let Item::Notebook(n) = i { Some(n.name.as_str()) } else { None }).collect();
            assert_eq!(nbs.len(), 2);
            assert!(nbs.contains(&"Alpha") && nbs.contains(&"Beta"));
            Ok(())
        })
        .unwrap();
        assert_eq!(v.with_mut(|s| s.collect_garbage()).unwrap(), 0);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn the_vault_saves_all_items_or_none() {
        let root = temp();
        let v = vault(&root.join("data"));
        let big = "x".repeat(33 * 1024 * 1024);
        let mk = |title: &str, content: String| {
            Item::Page(Page {
                source: None,
                id: new_id(),
                section_id: new_id(),
                title: title.into(),
                content,
                text: String::new(),
                order: 0,
                favorite: false,
                icon: String::new(),
                cover: String::new(),
                parent: String::new(),
                trashed_at: None,
                created: 1,
                updated: 1,
            })
        };
        let before = count(&v);
        let result = v.with_mut(|s| s.put_many(vec![mk("fine", "{}".into()), mk("too big", big)]));
        assert!(result.is_err());
        assert_eq!(count(&v), before, "the page that fit was not saved either");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn notaya_s_own_files_and_folders_are_refused() {
        let root = temp();
        let data = root.join("data");
        std::fs::create_dir_all(data.join("backups")).unwrap();
        std::fs::write(data.join("vault.nota"), b"vault").unwrap();
        std::fs::write(data.join("notes.md"), b"# inside the data folder").unwrap();
        std::fs::write(root.join("old.clf"), b"x").unwrap();
        std::fs::write(root.join("fine.md"), b"# ok").unwrap();
        assert!(check_sources(&[data.join("vault.nota")], &data, false).is_err());
        assert!(check_sources(&[root.join("old.CLF")], &data, false).is_err());
        assert!(check_sources(&[data.join("notes.md")], &data, false).is_err(), "a file inside the data folder");
        assert!(check_sources(&[data.clone()], &data, true).is_err(), "the data folder itself");
        assert!(check_sources(&[data.join("backups")], &data, true).is_err(), "a folder inside it");
        assert!(check_sources(&[root.clone()], &data, true).is_err(), "a folder that holds it");
        assert!(check_sources(&[data.join("../data/./notes.md")], &data, false).is_err(), "a path that climbs back in");
        assert!(check_sources(&[root.join("fine.md")], &data, false).is_ok());
        assert!(check_sources(&[], &data, false).is_err());
        // And end to end: nothing is written.
        let v = vault(&root.join("data2"));
        let before = count(&v);
        assert!(import_from(&v, &[data.join("notes.md")], false, &data, 1, &Control::standard()).is_err());
        assert_eq!(count(&v), before);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn hostile_files_are_refused_or_defused() {
        let root = temp();
        let data = root.join("data");
        let src = root.join("src");
        std::fs::create_dir_all(src.join("sub")).unwrap();
        std::fs::write(root.join("secret.png"), PNG).unwrap();
        let md = "# Evil\n\n[click](javascript:alert(1)) <script>alert(2)</script>\n\n![a](../../secret.png)\n\n![b](/etc/passwd)\n\n![c](C:/Windows/win.ini)\n\n![d](https://tracker.example/p.png)\n";
        std::fs::write(src.join("sub/page.md"), md).unwrap();
        let v = vault(&data);
        let summary = import_from(&v, &[src.clone()], true, &data, 1, &Control::standard()).unwrap();
        assert_eq!(summary.images, 0, "no picture outside the chosen folder was read");
        let page = &pages(&v)[0];
        for bad in ["javascript", "<script", "passwd", "Windows"] {
            assert!(!page.content.contains(bad), "{bad} leaked: {}", page.content);
        }
        assert!(page.content.contains("https://tracker.example/p.png"), "a remote picture stays a link and is not fetched");
        assert!(summary.skipped.len() >= 3, "{:?}", summary.skipped);
        assert_eq!(v.with_mut(|s| s.collect_garbage()).unwrap(), 0);
        let _ = std::fs::remove_dir_all(root);
    }

    /// Runs `hook(n, store)` before the n-th change of the vault (counting from 1), to play the
    /// part of something else happening during an import: a cleanup, a cancel, a lock.
    struct Hooked<'a> {
        inner: &'a Test,
        calls: std::sync::atomic::AtomicUsize,
        hook: Box<dyn Fn(usize, &Test) + Send + Sync + 'a>,
    }

    impl VaultAccess for Hooked<'_> {
        fn with<T>(&self, f: impl FnOnce(&Session) -> Result<T>) -> Result<T> {
            self.inner.with(f)
        }
        fn with_mut<T>(&self, f: impl FnOnce(&mut Session) -> Result<T>) -> Result<T> {
            let n = self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            (self.hook)(n, self.inner);
            self.inner.with_mut(f)
        }
    }

    fn two_picture_source(root: &Path) -> PathBuf {
        let src = root.join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("a.md"), "# A\n\n![x](a.png)\n\n![y](b.png)").unwrap();
        std::fs::write(src.join("a.png"), PNG).unwrap();
        std::fs::write(src.join("b.png"), [PNG, b"more"].concat()).unwrap();
        src.join("a.md")
    }

    fn flag_control(flag: &Arc<AtomicBool>) -> Control {
        Control::new(std::time::Duration::from_secs(60), flag.clone())
    }

    #[test]
    fn cancel_before_and_during_an_import_adds_nothing() {
        let root = temp();
        let file = two_picture_source(&root);
        let v = vault(&root.join("data"));
        let before = count(&v);
        // Already cancelled.
        let flag = Arc::new(AtomicBool::new(true));
        let err = import_from(&v, &[file.clone()], false, &root.join("data"), 1, &flag_control(&flag)).unwrap_err();
        assert_eq!(err.to_string(), plan::CANCELLED);
        assert_eq!(count(&v), before);
        // Cancelled after the first picture was stored: it is taken back.
        let flag = Arc::new(AtomicBool::new(false));
        let f2 = flag.clone();
        let hooked = Hooked {
            inner: &v,
            calls: 0.into(),
            hook: Box::new(move |n, _| {
                if n == 2 {
                    f2.store(true, Ordering::SeqCst);
                }
            }),
        };
        let err = import_from(&hooked, &[file.clone()], false, &root.join("data"), 1, &flag_control(&flag)).unwrap_err();
        assert_eq!(err.to_string(), plan::CANCELLED);
        assert_eq!(count(&v), before, "no notebook");
        assert_eq!(v.with_mut(|s| s.collect_garbage()).unwrap(), 0, "the stored picture was taken back");
        // The deadline ends the same way, with its own message.
        let err = import_from(&v, &[file.clone()], false, &root.join("data"), 1, &Control::expired()).unwrap_err();
        assert!(err.to_string().contains("took too long"), "{err}");
        // And a normal import afterwards works.
        let ok = import_from(&v, &[file], false, &root.join("data"), 1, &Control::standard()).unwrap();
        assert_eq!(ok.images, 2);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn the_cancel_command_reaches_the_running_import_and_only_one_runs() {
        assert!(!cancel_running(), "nothing is running");
        let running = Running::start().unwrap();
        assert!(Running::start().is_err(), "a second import is refused");
        assert!(!running.0.load(Ordering::SeqCst));
        assert!(import_cancel());
        assert!(running.0.load(Ordering::SeqCst));
        drop(running);
        assert!(!cancel_running());
        assert!(Running::start().is_ok());
    }

    #[test]
    fn a_cleanup_during_the_import_cannot_take_its_pictures() {
        let root = temp();
        let file = two_picture_source(&root);
        let v = vault(&root.join("data"));
        // Something runs the vault's cleanup before every change the import makes.
        let hooked = Hooked {
            inner: &v,
            calls: 0.into(),
            hook: Box::new(|_, store| {
                let _ = store.with_mut(|s| s.collect_garbage());
            }),
        };
        let summary = import_from(&hooked, &[file], false, &root.join("data"), 1, &Control::standard()).unwrap();
        assert_eq!(summary.images, 2);
        let page = &pages(&v)[0];
        for part in page.content.split("http://clatt.localhost/").skip(1) {
            let id = part.split('"').next().unwrap();
            assert!(v.with(|s| s.get_attachment(id)).is_ok(), "picture {id} survived");
        }
        // Once the pages are saved the pictures are referenced and nothing is held any more.
        assert_eq!(v.with_mut(|s| s.collect_garbage()).unwrap(), 0);
        // A picture stored outside any import is still collected.
        v.with_mut(|s| s.put_attachment("image/png", PNG)).unwrap();
        assert_eq!(v.with_mut(|s| s.collect_garbage()).unwrap(), 1);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn a_lock_in_the_middle_ends_the_import_cleanly() {
        let root = temp();
        let file = two_picture_source(&root);
        let data = root.join("data");
        let v = vault(&data);
        let before = count(&v);
        // Before the second change the session is replaced by a fresh one: a lock followed by an unlock.
        let path = data.join("vault.nota");
        let hooked = Hooked {
            inner: &v,
            calls: 0.into(),
            hook: Box::new(move |n, store| {
                if n == 2 {
                    *store.session.lock().unwrap() = Session::unlock_open(&path).unwrap();
                }
            }),
        };
        let err = import_from(&hooked, &[file], false, &data, 1, &Control::standard()).unwrap_err();
        assert!(err.to_string().contains("locked"), "{err}");
        assert_eq!(count(&v), before, "no notebook, section or page");
        assert_eq!(v.with_mut(|s| s.collect_garbage()).unwrap(), 0, "the first picture was taken back");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn a_document_cannot_pull_pictures_out_of_the_data_folder() {
        let root = temp();
        let data = root.join("data");
        let v = vault(&data);
        std::fs::write(data.join("pic.png"), PNG).unwrap();
        std::fs::write(root.join("a.md"), "# A\n\n![in data](data/pic.png)\n\n![beside](pic.png)").unwrap();
        std::fs::write(root.join("pic.png"), PNG).unwrap();
        let summary = import_from(&v, &[root.join("a.md")], false, &data, 1, &Control::standard()).unwrap();
        assert_eq!(summary.images, 1, "only the picture next to the document");
        assert!(summary.skipped.iter().any(|s| s.item.contains("in data")), "{:?}", summary.skipped);
        let _ = std::fs::remove_dir_all(root);
    }
}
