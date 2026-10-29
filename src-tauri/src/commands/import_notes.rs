//! "Import pages…": Markdown, HTML, Word and PDF files, a folder of them, and ZIP files (for
//! example a Notion export) become ONE new notebook (see `crate::importers`).
//!
//! The UI sends only which kind of source to import ("files" or "folder"). The dialog is opened
//! here, the files are read and converted here, and the UI gets back a summary. Nothing is ever
//! fetched from the network, links on disk are not followed, and the vault is changed in two
//! steps: the pictures are stored first, then the notebook, its sections and pages are saved in
//! one transaction (all or nothing). If anything fails, the pictures stored for this import are
//! taken back and nothing else has changed.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::error::{Error, Result};
use crate::importers::plan::{self, ImportSummary};
use crate::importers::source::Vfs;
use crate::model::{now_ms, Item};
use crate::state::AppState;
use crate::vault::Session;

/// Most files one pick may hold.
const MAX_PICKED: usize = 5_000;
/// Conversion nests function calls (documents inside documents), so it runs on a thread with a
/// big stack instead of the 2 MB default.
const STACK: usize = 128 * 1024 * 1024;

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

/// The whole import of the chosen paths into the vault. Nothing is written when it fails.
pub fn import_from<V: VaultAccess>(vault: &V, paths: &[PathBuf], folder: bool, data_dir: &Path, now: i64) -> Result<ImportSummary> {
    check_sources(paths, data_dir, folder)?;
    let hint = name_hint(paths, folder);
    on_big_stack(|| -> Result<ImportSummary> {
        let vfs = if folder { Vfs::from_folder(&paths[0]) } else { Vfs::from_files(paths) }.map_err(Error::invalid)?;
        let mut plan = plan::build(&vfs, hint.as_deref(), now).map_err(Error::invalid)?;
        // Pictures first: each is stored on its own, so the vault is not held while files are read.
        let mut stored: Vec<String> = Vec::new();
        let result = (|| -> Result<ImportSummary> {
            let urls = plan
                .store_images(&vfs, &mut |mime, bytes| {
                    let id = vault.with_mut(|s| s.put_attachment(mime, bytes)).map_err(|e| e.to_string())?;
                    stored.push(id.clone());
                    Ok(id)
                })
                .map_err(Error::Other)?;
            let order = vault.with(|s| Ok(next_notebook_order(s)))?;
            let (items, summary) = plan.items(&urls, now, order).map_err(Error::invalid)?;
            // The notebook, sections and pages in one transaction.
            vault.with_mut(|s| s.put_many(items))?;
            Ok(summary)
        })();
        if result.is_err() && !stored.is_empty() {
            // Take back exactly the pictures this import stored.
            let _ = vault.with_mut(|s| s.delete_attachments(&stored));
        }
        result
    })?
}

/// Asks what to import, then imports it as a new notebook. `None` if the dialog was cancelled.
/// Runs on a worker thread: the dialog waits for the person, and a big import takes a while.
#[tauri::command(async)]
pub fn import_notes(app: AppHandle, state: State<'_, AppState>, request: ImportRequest) -> Result<Option<ImportSummary>> {
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
    if paths.is_empty() {
        return Ok(None);
    }
    import_from(&*state, &paths, folder, &state.paths.data, now_ms()).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::importers::zip;
    use crate::model::{new_id, Page};
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
    fn a_folder_of_markdown_becomes_a_notebook_with_pictures() {
        let root = temp();
        let src = root.join("My notes");
        std::fs::create_dir_all(src.join("Work/img")).unwrap();
        std::fs::write(src.join("Work/plan.md"), "# Plan\n\nHello **world**\n\n![shot](img/a.png)\n").unwrap();
        std::fs::write(src.join("Work/img/a.png"), PNG).unwrap();
        std::fs::write(src.join("readme.md"), "Just text").unwrap();
        let v = vault(&root.join("data"));
        let summary = import_from(&v, &[src.clone()], true, &root.join("data"), 1_700_000_000_000).unwrap();
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
        import_from(&v, &[src], true, &root.join("data"), 1_700_000_000_001).unwrap();
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
        let summary = import_from(&v, &paths, false, &root.join("data"), 1_700_000_000_000).unwrap();
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
            let err = import_from(&f, &[src.join("a.md")], false, &root.join("data"), 1).unwrap_err();
            assert_eq!(err.to_string(), "disk full");
            assert_eq!(count(&v), before, "no notebook, section or page when it fails (fail_on {fail_on})");
            let orphans = v.with_mut(|s| s.collect_garbage()).unwrap();
            assert_eq!(orphans, 0, "the pictures it stored were taken back (fail_on {fail_on})");
        }
        // And the same files import fine afterwards.
        let ok = import_from(&v, &[src.join("a.md")], false, &root.join("data"), 1).unwrap();
        assert_eq!((ok.pages, ok.images), (1, 2));
        assert_eq!(v.with_mut(|s| s.collect_garbage()).unwrap(), 0, "both pictures are used");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn the_vault_saves_all_items_or_none() {
        let root = temp();
        let v = vault(&root.join("data"));
        let big = "x".repeat(33 * 1024 * 1024);
        let mk = |title: &str, content: String| {
            Item::Page(Page {
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
        assert!(import_from(&v, &[data.join("notes.md")], false, &data, 1).is_err());
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
        let summary = import_from(&v, &[src.clone()], true, &data, 1).unwrap();
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
}
