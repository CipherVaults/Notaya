//! The one-time, pre-1.6 safety copy for the live vault (A44/G3).

use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use super::{Format, Header, Store, Table, aad, anchor};
use crate::crypto::{self, Key};
use crate::error::{Error, Result};
use crate::filenames::{EXT, OLD_EXT, strip_vault_ext};
use crate::model::{Item, SETTINGS_ID};

const PREFIX: &str = "vault-before-1.6";
const PREFLIGHT_PREFIX: &str = "preflight-";
const PREFLIGHT_DB: &str = "vault.tmp";
const JOURNAL_TABLES: [Table; 3] = [Table::History, Table::Activity, Table::Anchor];

/// SQLite can report a hot journal on the first header query, after connecting.
/// Let it recover only on a disposable pair until the caller authenticates.
pub(super) fn with_preflight<T>(
    dir: &Path,
    live: &Path,
    inspect: impl FnOnce(Store, Header, bool) -> Result<T>,
) -> Result<T> {
    let read_only = || {
        let store = Store::open_read_only(live)?;
        let header = store.read_header()?;
        Ok((store, header))
    };
    match read_only() {
        Ok((store, header)) => inspect(store, header, false),
        Err(Error::Db(error)) if super::store::is_hot_journal(&error) => {
            let recovered = RecoveryCopy::new(dir, live)?;
            let store = Store::open(&recovered.db)?;
            let header = store.read_header()?;
            inspect(store, header, true)
        }
        Err(error) => Err(error),
    }
}

fn journal_path(db: &Path) -> PathBuf {
    let mut path = db.as_os_str().to_owned();
    path.push("-journal");
    path.into()
}

fn is_preflight_dir(path: &Path) -> bool {
    let generated_name = path.file_name().and_then(|name| name.to_str())
        .and_then(|name| name.strip_prefix(PREFLIGHT_PREFIX))
        .is_some_and(|id| id.len() == 32 && id.bytes().all(|byte| byte.is_ascii_hexdigit()));
    generated_name && fs::symlink_metadata(path).is_ok_and(|meta| {
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if meta.file_attributes() & 0x400 != 0 {
                return false; // Includes junctions, which can point outside the data folder.
            }
        }
        meta.is_dir()
    })
}

pub(crate) fn unprotected_preflight_dirs(data: &Path) -> impl Iterator<Item = PathBuf> + '_ {
    // Snapshot the candidates before probing, which can create a sibling recovery copy.
    let candidates: Vec<_> = fs::read_dir(data).into_iter().flatten()
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| is_preflight_dir(path))
        .collect();
    candidates.into_iter().filter(move |dir| {
        // Roll back a hot journal on a scratch pair before checking the open flag
        // and authenticating the header with the built-in open-vault credentials.
        with_preflight(data, &dir.join(PREFLIGHT_DB), |_, header, _| {
            header.unlock_open().map(|_| ())
        })
        .is_ok()
    })
}

pub(crate) fn remove_preflight_dir(dir: &Path) -> io::Result<()> {
    if !is_preflight_dir(dir) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "not a preflight directory"));
    }
    let db = dir.join(PREFLIGHT_DB);
    // Only the two files owned by preflight are disposable. Never delete a
    // directory tree, even if another file was put in it.
    for path in [&db, &journal_path(&db)] {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    fs::remove_dir(dir)
}

pub(crate) fn clean_stale_preflight(dir: &Path) {
    for stale in unprotected_preflight_dirs(dir) {
        if let Err(error) = remove_preflight_dir(&stale) {
            eprintln!("could not clean up a preflight copy: {:?}", error.kind());
        }
    }
}

/// Run before the window starts requesting vault operations, after the instance guard.
pub(crate) fn clean_stale_upgrade_parts(dir: &Path) {
    let prefix = format!("{PREFIX}-");
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let name = entry.file_name();
        let Some(id) = name.to_str()
            .and_then(|name| name.strip_prefix(&prefix))
            .and_then(|name| name.strip_suffix(".part").or_else(|| name.strip_suffix(".part-journal")))
        else {
            continue;
        };
        if id.len() == 32 && id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            if let Err(error) = remove_regular_upgrade_part(&entry.path()) {
                eprintln!("could not clean up a pre-upgrade copy: {:?}", error.kind());
            }
        }
    }
}

#[cfg(windows)]
fn remove_regular_upgrade_part(path: &Path) -> io::Result<()> {
    use std::os::windows::{fs::{MetadataExt, OpenOptionsExt}, io::AsRawHandle};
    use windows::Win32::{
        Foundation::HANDLE,
        Storage::FileSystem::{
            DELETE, FILE_ATTRIBUTE_REPARSE_POINT, FILE_DISPOSITION_INFO, FILE_FLAG_BACKUP_SEMANTICS,
            FILE_FLAG_OPEN_REPARSE_POINT, FILE_READ_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE,
            FileDispositionInfo, SetFileInformationByHandle,
        },
    };

    // Check and delete the same entry. Do not follow a link or allow the entry
    // to be renamed between the type check and the delete.
    let file = OpenOptions::new()
        .access_mode(DELETE.0 | FILE_READ_ATTRIBUTES.0)
        .share_mode((FILE_SHARE_READ | FILE_SHARE_WRITE).0)
        .custom_flags((FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS).0)
        .open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT.0 != 0 {
        return Ok(());
    }

    let disposition = FILE_DISPOSITION_INFO { DeleteFile: true };
    // The file handle and the correctly sized disposition remain live for the call.
    unsafe {
        SetFileInformationByHandle(
            HANDLE(file.as_raw_handle()),
            FileDispositionInfo,
            std::ptr::from_ref(&disposition).cast(),
            std::mem::size_of_val(&disposition) as u32,
        )
    }.map_err(io::Error::other)
}

#[cfg(not(windows))]
fn remove_regular_upgrade_part(path: &Path) -> io::Result<()> {
    if fs::symlink_metadata(path)?.is_file() {
        fs::remove_file(path)?;
    }
    Ok(())
}

struct RecoveryCopy {
    dir: PathBuf,
    db: PathBuf,
    journal: PathBuf,
}

impl RecoveryCopy {
    fn new(dir: &Path, live: &Path) -> Result<Self> {
        let dir = dir.join(format!("{PREFLIGHT_PREFIX}{}", crate::model::new_id()));
        fs::create_dir(&dir)?;
        let db = dir.join(PREFLIGHT_DB);
        let journal = journal_path(&db);
        let copy = Self { dir, db, journal };
        let mut db = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&copy.db)?;
        io::copy(&mut File::open(live)?, &mut db)?;
        let mut journal = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&copy.journal)?;
        // The probe found a hot journal. If it disappeared, fail without using
        // a database image that might have been copied before rollback.
        io::copy(&mut File::open(journal_path(live))?, &mut journal)?;
        drop(journal);
        super::store::check_rollback_journal(&copy.journal)?;
        Ok(copy)
    }
}

impl Drop for RecoveryCopy {
    fn drop(&mut self) {
        let _ = remove_preflight_dir(&self.dir);
    }
}

pub(crate) fn is_before_1_6_name(name: &str) -> bool {
    let Some(stem) = strip_vault_ext(name) else {
        return false;
    };
    stem == PREFIX
        || stem
            .strip_prefix(PREFIX)
            .and_then(|s| s.strip_prefix('-'))
            .is_some_and(|suffix| !suffix.is_empty() && suffix.bytes().all(|b| b.is_ascii_digit()))
}

/// The settings item is authenticated before its latch can identify a pre-journal
/// vault. A missing anchor on a latched vault is damage, even if every table is gone.
pub(super) fn needs_upgrade(source: &Store, header: &Header, dek: &Key) -> Result<bool> {
    let id = super::parse_id(SETTINGS_ID)?;
    let sealed = source.get(Table::Records, &id)?.ok_or(Error::Corrupt)?;
    let plain = crypto::open(
        dek,
        &aad(header.format, &header.vault_id, Table::Records, &id),
        &sealed,
    )?;
    let Item::Settings(settings) =
        serde_json::from_slice(crypto::unpad(&plain)?).map_err(|_| Error::Corrupt)?
    else {
        return Err(Error::Corrupt);
    };
    let has_anchor_table = source.exists(Table::Anchor)?;
    if has_anchor_table && source.get(Table::Anchor, &anchor::ID)?.is_some() {
        for table in [Table::History, Table::Activity] {
            if !source.exists(table)? {
                return Err(Error::Corrupt);
            }
        }
        return Ok(header.format == Format::Legacy);
    }
    if settings.journals_anchored || (has_anchor_table && !source.ids(Table::Anchor)?.is_empty()) {
        return Err(Error::invalid(
            "The journal anchor is missing. The log or history may have been changed.",
        ));
    }
    Ok(true)
}

pub(super) fn before_upgrade(
    dir: &Path,
    source: &Store,
    header: &Header,
    dek: &Key,
) -> Result<String> {
    save_copy(dir, source, header, dek).map_err(|_| Error::invalid(
        "Notaya could not save or verify the pre-upgrade safety copy. Check that the data folder and any existing vault-before-1.6 copy are accessible and that the drive has free space, then try opening the vault again.",
    ))
}

fn save_copy(dir: &Path, source: &Store, header: &Header, dek: &Key) -> Result<String> {
    // A previous attempt may have published the copy before its upgrade failed.
    // Match all saved rows, not just the header: 1.5.1 can edit a vault in place.
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str().filter(|name| is_before_1_6_name(name)) else {
            continue;
        };
        if !entry.file_type()?.is_file() {
            return Err(Error::Corrupt);
        }
        let copy = Store::open_read_only(&entry.path())?;
        if matches_source(source, &copy, header, dek, true)? {
            return Ok(name.to_owned());
        }
    }

    let ext = if header.format == Format::Legacy {
        OLD_EXT
    } else {
        EXT
    };
    let mut name = format!("{PREFIX}.{ext}");
    let mut dest = dir.join(&name);
    let mut suffix = 1;
    while dest.try_exists()? {
        suffix += 1;
        name = format!("{PREFIX}-{suffix}.{ext}");
        dest = dir.join(&name);
    }
    let part = dir.join(format!("{PREFIX}-{}.part", crate::model::new_id()));
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&part)?;
    let result = (|| -> Result<()> {
        source.copy_into(&part)?;
        let copy = Store::open_read_only(&part)?;
        if !matches_source(source, &copy, header, dek, true)? {
            return Err(Error::Corrupt);
        }
        drop(copy);
        crate::backup::sync_to_disk(&part)?;
        publish(&part, &dest)?;
        Ok(())
    })();
    let _ = fs::remove_file(part);
    result.map(|()| name)
}

/// Verification is read-only and checks one row at a time. A pre-journal copy
/// must have exactly the same journal tables as its source, including absence.
pub(super) fn matches_source(
    source: &Store,
    copy: &Store,
    header: &Header,
    dek: &Key,
    allow_pre_journal: bool,
) -> Result<bool> {
    let expected = header.to_bytes();
    if source.read_header()?.to_bytes() != expected || copy.read_header()?.to_bytes() != expected {
        return Ok(false);
    }
    for table in [Table::Records, Table::Attachments]
        .into_iter()
        .chain(JOURNAL_TABLES)
    {
        let source_present = source.exists(table)?;
        let copy_present = copy.exists(table)?;
        if !source_present && !copy_present && allow_pre_journal && JOURNAL_TABLES.contains(&table)
        {
            continue;
        }
        if !source_present || !copy_present {
            return Ok(false);
        }
        if !source.matching_rows(copy, table, |id, sealed| {
            let plain = crypto::open(
                dek,
                &aad(header.format, &header.vault_id, table, id),
                sealed,
            )?;
            crypto::unpad(&plain)?;
            Ok(())
        })? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Publish in the same directory with write-through and without replacing a
/// previous safety copy, including on portable Windows filesystems without hard links.
#[cfg(windows)]
fn publish(part: &Path, dest: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Storage::FileSystem::{MOVEFILE_WRITE_THROUGH, MoveFileExW};
    use windows::core::PCWSTR;

    let wide = |path: &Path| {
        path.as_os_str()
            .encode_wide()
            .chain([0])
            .collect::<Vec<_>>()
    };
    let from = wide(part);
    let to = wide(dest);
    // Both buffers are terminated and live for the entire call.
    unsafe {
        MoveFileExW(
            PCWSTR(from.as_ptr()),
            PCWSTR(to.as_ptr()),
            MOVEFILE_WRITE_THROUGH,
        )
    }
    .map_err(std::io::Error::other)
}

#[cfg(not(windows))]
fn publish(part: &Path, dest: &Path) -> std::io::Result<()> {
    fs::hard_link(part, dest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::vault::{auto_open_core, unlock_vault_core};
    use crate::crypto::KdfParams;
    use crate::model::{Page, new_id};
    use crate::openvault;
    use crate::portable::Paths;
    use crate::state::AppState;
    use crate::vault::{Session, store::RowId};
    use rusqlite::Connection;

    const PASSWORD: &str = "Disposable vault test password";
    const FAST: KdfParams = KdfParams {
        m_kib: 64,
        t: 1,
        p: 1,
    };
    const NOTA_COPY: &str = "vault-before-1.6.nota";
    const CLF_COPY: &str = "vault-before-1.6.clf";
    const TEXT: &str = "A note from before 1.6";
    const ATTACHMENT: &[u8] = b"A test attachment from before 1.6";

    struct Before {
        header: Vec<u8>,
        records: Vec<(RowId, Vec<u8>)>,
        attachments: Vec<(RowId, Vec<u8>)>,
        note_id: String,
        attachment_id: String,
    }

    fn paths() -> Paths {
        let paths = Paths::at(&std::env::temp_dir().join(format!("notaya-g3-{}", new_id())));
        paths.ensure_dirs().unwrap();
        paths
    }

    // The old reader had only header, records and attachments, and its settings
    // did not have the authenticated latch. Keep the real format-specific AEAD.
    fn pre_journal(paths: &Paths, mut session: Session) -> Before {
        let page = Page {
            id: new_id(),
            section_id: new_id(),
            title: "Test note".into(),
            text: TEXT.into(),
            content: "{}".into(),
            order: 0,
            favorite: false,
            icon: String::new(),
            cover: String::new(),
            parent: String::new(),
            trashed_at: None,
            source: None,
            created: 1,
            updated: 2,
        };
        let note_id = page.id.clone();
        session.put(Item::Page(page)).unwrap();
        let attachment_id = session.put_attachment("image/png", ATTACHMENT).unwrap();
        let mut settings = session.settings();
        settings.journals_anchored = false;
        let (id, bytes) = session.seal_item(&Item::Settings(settings)).unwrap();
        session.store.put(Table::Records, &id, &bytes).unwrap();
        let before = Before {
            header: session.store.read_header().unwrap().to_bytes(),
            records: session.store.all(Table::Records).unwrap(),
            attachments: session.store.all(Table::Attachments).unwrap(),
            note_id,
            attachment_id,
        };
        drop(session);
        Connection::open(&paths.vault)
            .unwrap()
            .execute_batch("DROP TABLE history; DROP TABLE activity; DROP TABLE journal_anchor;")
            .unwrap();
        before
    }

    fn assert_upgraded(
        state: &AppState,
        before: &Before,
        name: &str,
        event: Option<String>,
    ) -> PathBuf {
        let path = state.paths.data.join(name);
        assert_eq!(openvault::earlier_copies(&state.paths), vec![path.clone()]);
        assert_eq!(fs::read_dir(&state.paths.backups).unwrap().count(), 0);
        let copy = Store::open_read_only(&path).unwrap();
        assert_eq!(copy.read_header().unwrap().to_bytes(), before.header);
        assert_eq!(copy.all(Table::Records).unwrap(), before.records);
        assert_eq!(copy.all(Table::Attachments).unwrap(), before.attachments);
        for table in JOURNAL_TABLES {
            assert!(!copy.exists(table).unwrap());
        }

        let header = Session::read_header(&state.paths.vault).unwrap();
        assert_eq!(&header.to_bytes()[..6], b"NOTA\x02\x00");
        if name == NOTA_COPY {
            assert_eq!(header.to_bytes(), before.header, "no re-key for NOTA v2");
        } else {
            assert_eq!(&before.header[..6], b"CLF1\x01\x00");
        }
        state.with(|session| {
            assert!(session.settings().journals_anchored);
            assert!(matches!(session.get(&before.note_id), Some(Item::Page(page)) if page.text == TEXT));
            assert_eq!(session.get_attachment(&before.attachment_id)?.1, ATTACHMENT);
            Ok(())
        }).unwrap();

        let event = event.expect("notice to emit when the session opens");
        assert!(event.contains(name));
        let pending = state.pending_notice().unwrap();
        assert_eq!(pending.text, event);
        state.acknowledge_notice(&pending.id);
        assert_eq!(state.pending_notice(), None);
        path
    }

    #[test]
    fn cleanup_removes_only_regular_upgrade_parts_and_journals() {
        let paths = paths();
        let data = &paths.data;
        let stale = data.join(format!("{PREFIX}-{}.part", "aF".repeat(16)));
        let orphan_journal = data.join(format!("{PREFIX}-{}.part-journal", "B".repeat(32)));
        let removable = [&stale, &journal_path(&stale), &orphan_journal];
        let preserved = [
            NOTA_COPY.to_string(),
            CLF_COPY.to_string(),
            format!("{PREFIX}-2.nota"),
            format!("{PREFIX}-2.clf"),
            "vault.nota".to_string(),
            "vault.clf".to_string(),
            format!("{PREFIX}-{}.part", "a".repeat(31)),
            format!("{PREFIX}-{}.part", "a".repeat(33)),
            format!("{PREFIX}-{}.part", "g".repeat(32)),
            format!("{PREFIX}-{}.part", "é".repeat(16)),
            format!("{PREFIX}-{}.part-wal", "b".repeat(32)),
            format!("{PREFIX}-{}.nota", "d".repeat(32)),
            format!("other-{PREFIX}-{}.part-journal", "b".repeat(32)),
        ];
        let directory = data.join(format!("{PREFIX}-{}.part", "c".repeat(32)));
        fs::create_dir(&directory).unwrap();
        fs::write(directory.join("marker"), b"directory").unwrap();
        for file in removable {
            fs::write(file, b"abandoned copy").unwrap();
        }
        for name in &preserved {
            fs::write(data.join(name), b"keep").unwrap();
        }

        clean_stale_upgrade_parts(data);

        for file in removable {
            assert!(!file.exists(), "stale part remains: {file:?}");
        }
        for name in &preserved {
            assert_eq!(fs::read(data.join(name)).unwrap(), b"keep");
        }
        assert_eq!(fs::read(directory.join("marker")).unwrap(), b"directory");
    }

    #[test]
    fn cleanup_skips_matching_symlinks() {
        #[cfg(unix)]
        use std::os::unix::fs::symlink as link;
        #[cfg(windows)]
        use std::os::windows::fs::symlink_file as link;

        let paths = paths();
        let target = paths.data.join("vault.nota");
        fs::write(&target, b"vault").unwrap();
        let candidate = paths.data.join(format!("{PREFIX}-{}.part", "d".repeat(32)));
        if let Err(error) = link(&target, &candidate) {
            #[cfg(windows)]
            if error.raw_os_error() == Some(1314) {
                eprintln!("file symlink check skipped: Windows symlink privilege unavailable");
                return;
            }
            panic!("could not create test symlink: {error}");
        }

        clean_stale_upgrade_parts(&paths.data);

        assert!(fs::symlink_metadata(&candidate).unwrap().is_symlink());
        assert_eq!(fs::read(&target).unwrap(), b"vault");
    }

    #[test]
    fn verification_rejects_different_ciphertext_ids_counts_and_table_presence() {
        let paths = paths();
        let before = pre_journal(&paths, Session::create_open(&paths.vault).unwrap());
        let source = Store::open_read_only(&paths.vault).unwrap();
        let header = source.read_header().unwrap();
        let dek = header.unlock_open().unwrap();
        let candidate = paths.data.join("candidate.tmp");
        source.copy_into(&candidate).unwrap();
        let copy = Store::open(&candidate).unwrap();
        let matches = || matches_source(&source, &copy, &header, &dek, true).unwrap();
        assert!(matches());

        let (id, sealed) = &before.attachments[0];
        let mut changed = sealed.clone();
        *changed.last_mut().unwrap() ^= 1;
        copy.put(Table::Attachments, id, &changed).unwrap();
        assert!(!matches(), "ciphertext or tag differs");

        copy.delete(Table::Attachments, id).unwrap();
        assert!(!matches(), "copy has fewer rows");
        let different_id = [0; 16];
        assert_ne!(*id, different_id);
        copy.put(Table::Attachments, &different_id, sealed).unwrap();
        assert!(!matches(), "same row count with a different id");
        copy.put(Table::Attachments, id, sealed).unwrap();
        assert!(!matches(), "copy has more rows");

        copy.delete(Table::Attachments, &different_id).unwrap();
        assert!(matches());
        copy.ensure_journal_tables().unwrap();
        assert!(!matches(), "journal table presence differs");
    }

    #[test]
    fn auto_open_copies_pre_journal_nota_and_notifies_once() {
        let paths = paths();
        let before = pre_journal(&paths, Session::create_open(&paths.vault).unwrap());
        let state = AppState::new(paths.clone());
        let (opened, notice) = auto_open_core(&state).unwrap();
        assert!(opened);
        let copy = assert_upgraded(&state, &before, NOTA_COPY, notice);
        assert_eq!(openvault::unprotected_copies(&paths), vec![copy.clone()]);
        drop(state);

        let reopened = AppState::new(paths.clone());
        assert_eq!(auto_open_core(&reopened).unwrap(), (true, None));
        assert_eq!(reopened.pending_notice(), None);
        assert_eq!(openvault::earlier_copies(&paths), vec![copy]);
    }

    #[test]
    fn simultaneous_auto_opens_publish_one_copy_and_notice() {
        let paths = paths();
        let before = pre_journal(&paths, Session::create_open(&paths.vault).unwrap());
        let state = AppState::new(paths);
        let start = std::sync::Barrier::new(2);
        let (first, second) = std::thread::scope(|scope| {
            let worker = scope.spawn(|| {
                start.wait();
                auto_open_core(&state).unwrap()
            });
            start.wait();
            (auto_open_core(&state).unwrap(), worker.join().unwrap())
        });
        let (opened, skipped) = if first.0 {
            (first, second)
        } else {
            (second, first)
        };
        assert!(opened.0);
        assert_eq!(skipped, (false, None));
        assert_upgraded(&state, &before, NOTA_COPY, opened.1);
    }

    fn protected_upgrade(format: Format, use_keyfile: bool) {
        let paths = paths();
        let good_key = paths.data.join("test.key");
        let wrong_key = paths.data.join("wrong.key");
        let digest = use_keyfile.then(|| {
            fs::write(&good_key, b"test keyfile").unwrap();
            fs::write(&wrong_key, b"wrong keyfile").unwrap();
            crypto::keyfile_digest(&good_key).unwrap()
        });
        let keyfile = use_keyfile.then(|| good_key.to_str().unwrap().to_owned());
        let (session, name) = match format {
            Format::Legacy => (
                Session::create_legacy(&paths.vault, PASSWORD, digest.as_deref(), FAST),
                CLF_COPY,
            ),
            Format::Current => (
                Session::create(&paths.vault, PASSWORD, digest.as_deref(), FAST),
                NOTA_COPY,
            ),
        };
        let before = pre_journal(&paths, session.unwrap());
        let original = fs::read(&paths.vault).unwrap();
        let state = AppState::new(paths.clone());
        assert_eq!(auto_open_core(&state).unwrap(), (false, None));
        let mut wrong_credentials = vec![("wrong password", keyfile.clone())];
        if use_keyfile {
            wrong_credentials.push((PASSWORD, Some(wrong_key.to_str().unwrap().to_owned())));
        }
        for (password, keyfile) in wrong_credentials {
            assert!(matches!(
                unlock_vault_core(&state, password.into(), keyfile),
                Err(Error::BadCredentials)
            ));
            assert!(!state.is_unlocked());
            assert_eq!(state.pending_notice(), None);
            assert!(openvault::earlier_copies(&paths).is_empty());
            assert_eq!(fs::read(&paths.vault).unwrap(), original);
        }

        let (opened, notice) = unlock_vault_core(&state, PASSWORD.into(), keyfile.clone()).unwrap();
        assert!(opened);
        let copy = assert_upgraded(&state, &before, name, notice);
        assert!(openvault::unprotected_copies(&paths).is_empty());
        drop(state);

        let reopened = AppState::new(paths.clone());
        assert_eq!(
            unlock_vault_core(&reopened, PASSWORD.into(), keyfile).unwrap(),
            (true, None)
        );
        assert_eq!(reopened.pending_notice(), None);
        assert_eq!(openvault::earlier_copies(&paths), vec![copy]);
    }

    #[test]
    fn unlock_copies_pre_journal_nota_with_password() {
        protected_upgrade(Format::Current, false);
    }

    #[test]
    fn unlock_copies_pre_journal_nota_with_password_and_keyfile() {
        protected_upgrade(Format::Current, true);
    }

    #[test]
    fn unlock_copies_clf1_with_password() {
        protected_upgrade(Format::Legacy, false);
    }

    #[test]
    fn unlock_copies_clf1_with_password_and_keyfile() {
        protected_upgrade(Format::Legacy, true);
    }

    fn interrupt_write(db_path: &Path) -> (Vec<u8>, Vec<u8>) {
        let original = fs::read(db_path).unwrap();
        let journal_path = journal_path(db_path);
        let writer = Connection::open(db_path).unwrap();
        // Spill uncommitted pages so SQLite syncs a real rollback journal and
        // changes the database on disk. Capture the pair before closing SQLite.
        writer.execute_batch(
            "PRAGMA journal_mode = DELETE;
             PRAGMA synchronous = FULL;
             PRAGMA cache_size = 1;
             PRAGMA cache_spill = ON;
             BEGIN IMMEDIATE;
             UPDATE header SET data = zeroblob(4096);
             UPDATE records SET data = zeroblob(262144);",
        ).unwrap();
        let db = fs::read(db_path).unwrap();
        let journal = fs::read(&journal_path).unwrap();
        assert_ne!(db, original, "SQLite must have spilled uncommitted pages");
        drop(writer);
        // Restore the interrupted disk image without the original writer's lock.
        fs::write(db_path, &db).unwrap();
        fs::write(&journal_path, &journal).unwrap();
        assert!(matches!(
            Session::read_header(db_path),
            Err(Error::Db(e)) if super::super::store::is_hot_journal(&e)
        ));
        (db, journal)
    }

    fn assert_preflight_cleaned(paths: &Paths) {
        assert!(
            !fs::read_dir(&paths.data).unwrap().any(|entry| {
                entry.unwrap().file_name().to_string_lossy().starts_with("preflight-")
            }),
            "the temporary rollback pair must be removed"
        );
    }

    #[test]
    fn protected_preflight_with_hot_journal_survives_cleanup() {
        let paths = paths();
        let stale = paths.data.join(format!("{PREFLIGHT_PREFIX}{}", new_id()));
        fs::create_dir(&stale).unwrap();
        let db_path = stale.join(PREFLIGHT_DB);
        drop(Session::create(&db_path, PASSWORD, None, FAST).unwrap());
        assert!(!Session::read_header(&db_path).unwrap().open);
        let (db, journal) = interrupt_write(&db_path);

        assert!(openvault::unprotected_copies(&paths).is_empty());
        assert_eq!(openvault::delete_unprotected(&paths), 0);
        assert_eq!(fs::read(&db_path).unwrap(), db);
        assert_eq!(fs::read(journal_path(&db_path)).unwrap(), journal);

        clean_stale_preflight(&paths.data);
        assert_eq!(fs::read(&db_path).unwrap(), db);
        assert_eq!(fs::read(journal_path(&db_path)).unwrap(), journal);
    }

    #[test]
    fn hot_journal_unlock_authenticates_before_touching_live_v2_vault() {
        let paths = paths();
        let good_keyfile = paths.data.join("test.key");
        let wrong_keyfile = paths.data.join("wrong.key");
        fs::write(&good_keyfile, b"test keyfile").unwrap();
        fs::write(&wrong_keyfile, b"wrong keyfile").unwrap();
        let digest = crypto::keyfile_digest(&good_keyfile).unwrap();
        let before = pre_journal(
            &paths,
            Session::create(&paths.vault, PASSWORD, Some(&digest), FAST).unwrap(),
        );
        let (live, journal) = interrupt_write(&paths.vault);
        let journal_path = journal_path(&paths.vault);
        let state = AppState::new(paths.clone());
        assert_eq!(auto_open_core(&state).unwrap(), (false, None));
        assert!(Session::read_live_header(&paths).unwrap().keyfile_required);
        for (password, keyfile) in [("wrong password", &good_keyfile), (PASSWORD, &wrong_keyfile)] {
            assert!(matches!(
                unlock_vault_core(&state, password.into(), Some(keyfile.to_str().unwrap().into())),
                Err(Error::BadCredentials)
            ));
            assert_eq!(fs::read(&paths.vault).unwrap(), live);
            assert_eq!(fs::read(&journal_path).unwrap(), journal);
            assert!(openvault::earlier_copies(&paths).is_empty());
            assert!(state.pending_notice().is_none());
            assert!(!state.is_unlocked());
            assert_preflight_cleaned(&paths);
        }

        let (opened, notice) = unlock_vault_core(
            &state, PASSWORD.into(), Some(good_keyfile.to_str().unwrap().into()),
        ).unwrap();
        assert!(opened);
        assert_upgraded(&state, &before, NOTA_COPY, notice);
        assert!(!journal_path.exists());
        assert_preflight_cleaned(&paths);
    }

    #[test]
    fn hot_journal_auto_open_reuses_a_verified_v2_copy() {
        let paths = paths();
        let before = pre_journal(&paths, Session::create_open(&paths.vault).unwrap());
        let store = Store::open_read_only(&paths.vault).unwrap();
        let header = store.read_header().unwrap();
        let dek = header.unlock_open().unwrap();
        let name = before_upgrade(&paths.data, &store, &header, &dek).unwrap();
        drop(store);
        let copy = paths.data.join(name);
        let modified = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000);
        OpenOptions::new().write(true).open(&copy).unwrap().set_modified(modified).unwrap();
        interrupt_write(&paths.vault);
        // Model a second hard stop while recovering the previous interrupted write.
        let stale = paths.data.join(format!("{PREFLIGHT_PREFIX}{}", new_id()));
        fs::create_dir(&stale).unwrap();
        let stale_db = stale.join(PREFLIGHT_DB);
        fs::copy(&paths.vault, &stale_db).unwrap();
        fs::copy(journal_path(&paths.vault), journal_path(&stale_db)).unwrap();

        let state = AppState::new(paths.clone());
        let (opened, notice) = auto_open_core(&state).unwrap();
        assert!(opened);
        assert_eq!(assert_upgraded(&state, &before, NOTA_COPY, notice), copy);
        assert_eq!(fs::metadata(copy).unwrap().modified().unwrap(), modified);
        assert!(!journal_path(&paths.vault).exists());
        assert_preflight_cleaned(&paths);
    }

    #[test]
    fn start_over_and_restore_recover_the_preserved_hot_journal() {
        use crate::commands::backup::{restore_vault_stage_core, start_over_core};

        let paths = paths();
        let before = pre_journal(
            &paths,
            Session::create(&paths.vault, PASSWORD, None, FAST).unwrap(),
        );
        let (interrupted, journal) = interrupt_write(&paths.vault);
        let state = AppState::new(paths.clone());
        assert_eq!(auto_open_core(&state).unwrap(), (false, None));

        let source = paths.data.join(start_over_core(&state).unwrap());
        let source_journal = journal_path(&source);
        let staged = paths.data.join("restore-check.tmp");
        let source_name = source.to_str().unwrap();
        state.set_picked_backup(source.canonicalize().unwrap());
        assert!(!paths.vault.exists());
        assert!(!journal_path(&paths.vault).exists());

        assert!(matches!(
            restore_vault_stage_core(&state, source_name, "wrong password", None, None),
            Err(Error::BadCredentials)
        ));
        assert!(!staged.exists());
        assert!(!journal_path(&staged).exists());
        assert!(!paths.vault.exists());

        restore_vault_stage_core(&state, source_name, PASSWORD, None, None).unwrap();
        assert!(!journal_path(&staged).exists());
        assert_eq!(crate::restore::install(&paths, 1).unwrap(), None);
        assert!(!staged.exists());
        assert!(!journal_path(&paths.vault).exists());
        let restored = Session::unlock(&paths.vault, PASSWORD, None).unwrap();
        assert_eq!(restored.store.read_header().unwrap().to_bytes(), before.header);
        // Opening a pre-journal vault initializes its anchor and updates settings.
        assert!(restored.settings().journals_anchored);
        let mut restored_records = restored.store.all(Table::Records).unwrap();
        let mut original_records = before.records;
        let is_content = |(id, _): &(RowId, Vec<u8>)| hex::encode(id) != SETTINGS_ID;
        restored_records.retain(is_content);
        original_records.retain(is_content);
        assert!(restored_records == original_records, "all saved content rows must match");
        assert_eq!(restored.store.all(Table::Attachments).unwrap(), before.attachments);
        assert_eq!(fs::read(&source).unwrap(), interrupted);
        assert_eq!(fs::read(&source_journal).unwrap(), journal);
    }

    #[test]
    fn hot_journal_with_tampered_settings_leaves_live_pair_untouched() {
        let paths = paths();
        pre_journal(
            &paths,
            Session::create(&paths.vault, PASSWORD, None, FAST).unwrap(),
        );
        let settings_id = super::super::parse_id(SETTINGS_ID).unwrap();
        Store::open(&paths.vault)
            .unwrap()
            .put(Table::Records, &settings_id, b"tampered settings")
            .unwrap();
        let (live, journal) = interrupt_write(&paths.vault);
        let state = AppState::new(paths.clone());
        assert!(unlock_vault_core(&state, PASSWORD.into(), None).is_err());
        assert_eq!(fs::read(&paths.vault).unwrap(), live);
        assert_eq!(fs::read(journal_path(&paths.vault)).unwrap(), journal);
        assert!(openvault::earlier_copies(&paths).is_empty());
        assert!(state.pending_notice().is_none());
        assert!(!state.is_unlocked());
        assert_preflight_cleaned(&paths);
    }

    #[test]
    fn blocked_copy_leaves_live_bytes_unchanged_and_can_be_retried() {
        let paths = paths();
        let before = pre_journal(&paths, Session::create_open(&paths.vault).unwrap());
        let original = fs::read(&paths.vault).unwrap();
        let blocked = paths.data.join(NOTA_COPY);
        fs::create_dir(&blocked).unwrap();
        let state = AppState::new(paths.clone());
        let error = auto_open_core(&state).unwrap_err().to_string();
        assert!(
            error.contains("pre-upgrade safety copy")
                && error.contains("try opening the vault again")
        );
        assert!(!state.is_unlocked());
        assert_eq!(state.pending_notice(), None);
        assert_eq!(fs::read(&paths.vault).unwrap(), original);
        assert!(openvault::earlier_copies(&paths).is_empty());

        fs::remove_dir(blocked).unwrap();
        let (opened, notice) = auto_open_core(&state).unwrap();
        assert!(opened);
        assert_upgraded(&state, &before, NOTA_COPY, notice);
    }

    #[test]
    fn a_copy_that_cannot_authenticate_all_rows_is_never_published() {
        let paths = paths();
        let mut before = pre_journal(
            &paths,
            Session::create(&paths.vault, PASSWORD, None, FAST).unwrap(),
        );
        let (id, good_attachment) = before.attachments.pop().unwrap();
        let store = Store::open(&paths.vault).unwrap();
        store
            .put(Table::Attachments, &id, b"damaged ciphertext")
            .unwrap();
        drop(store);
        let original = fs::read(&paths.vault).unwrap();
        let state = AppState::new(paths.clone());
        let error = unlock_vault_core(&state, PASSWORD.into(), None)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("pre-upgrade safety copy")
                && error.contains("try opening the vault again")
        );
        assert_eq!(fs::read(&paths.vault).unwrap(), original);
        assert!(!state.is_unlocked());
        assert_eq!(state.pending_notice(), None);
        assert!(openvault::earlier_copies(&paths).is_empty());
        assert!(
            !fs::read_dir(&paths.data)
                .unwrap()
                .flatten()
                .any(|e| e.path().extension().is_some_and(|s| s == "part"))
        );

        Store::open(&paths.vault)
            .unwrap()
            .put(Table::Attachments, &id, &good_attachment)
            .unwrap();
        before.attachments.push((id, good_attachment));
        let (opened, notice) = unlock_vault_core(&state, PASSWORD.into(), None).unwrap();
        assert!(opened);
        assert_upgraded(&state, &before, NOTA_COPY, notice);
    }

    #[test]
    fn retry_uses_the_verified_copy_published_before_an_interruption() {
        let paths = paths();
        let before = pre_journal(&paths, Session::create_open(&paths.vault).unwrap());
        let original = fs::read(&paths.vault).unwrap();
        // A reader permits the SQLite snapshot but prevents the upgrade from committing.
        let reader =
            Connection::open_with_flags(&paths.vault, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                .unwrap();
        reader.execute_batch("BEGIN").unwrap();
        reader
            .query_row("SELECT id FROM header", [], |row| row.get::<_, i64>(0))
            .unwrap();
        let state = AppState::new(paths.clone());
        assert!(matches!(auto_open_core(&state), Err(Error::Db(_))));
        let copy = paths.data.join(NOTA_COPY);
        assert_eq!(openvault::earlier_copies(&paths), vec![copy.clone()]);
        assert_eq!(fs::read(&paths.vault).unwrap(), original);
        assert_eq!(state.pending_notice(), None);
        assert!(!state.is_unlocked());
        drop(reader);

        let mtime = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000);
        OpenOptions::new()
            .write(true)
            .open(&copy)
            .unwrap()
            .set_modified(mtime)
            .unwrap();
        let (opened, notice) = auto_open_core(&state).unwrap();
        assert!(opened);
        assert_eq!(assert_upgraded(&state, &before, NOTA_COPY, notice), copy);
        assert_eq!(fs::metadata(copy).unwrap().modified().unwrap(), mtime);
    }

    #[test]
    fn corrupted_open_credentials_and_missing_current_journals_do_not_trigger_a_copy() {
        let paths = paths();
        drop(Session::create_open(&paths.vault).unwrap());
        let store = Store::open(&paths.vault).unwrap();
        let header = store.read_header().unwrap();
        let mut damaged_header = header.clone();
        damaged_header.salt[0] ^= 1;
        store.write_header(&damaged_header).unwrap();
        drop(store);
        let original = fs::read(&paths.vault).unwrap();
        let state = AppState::new(paths.clone());
        assert!(matches!(auto_open_core(&state), Err(Error::BadCredentials)));
        assert_eq!(fs::read(&paths.vault).unwrap(), original);

        Store::open(&paths.vault)
            .unwrap()
            .write_header(&header)
            .unwrap();
        Connection::open(&paths.vault)
            .unwrap()
            .execute_batch("DROP TABLE history; DROP TABLE activity; DROP TABLE journal_anchor;")
            .unwrap();
        let missing_journals = fs::read(&paths.vault).unwrap();
        assert!(
            auto_open_core(&state)
                .unwrap_err()
                .to_string()
                .contains("anchor is missing")
        );
        assert_eq!(fs::read(&paths.vault).unwrap(), missing_journals);
        assert!(openvault::earlier_copies(&paths).is_empty());
        assert_eq!(state.pending_notice(), None);
    }
}
