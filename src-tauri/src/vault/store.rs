//! The on-disk vault: one SQLite file holding only the header and opaque
//! encrypted blobs. Row ids are random, and titles, types and timestamps all
//! sit inside the encrypted payloads.

use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};

use super::header::Header;
use crate::error::{Error, Result};

pub type RowId = [u8; 16];

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn create(path: &Path, header: &Header) -> Result<Store> {
        if path.exists() {
            return Err(Error::VaultExists);
        }
        let store = Self::connect(path)?;
        store.conn.execute_batch(
            "CREATE TABLE header (id INTEGER PRIMARY KEY CHECK (id = 1), data BLOB NOT NULL);
             CREATE TABLE records (id BLOB PRIMARY KEY, data BLOB NOT NULL) WITHOUT ROWID;
             CREATE TABLE attachments (id BLOB PRIMARY KEY, data BLOB NOT NULL) WITHOUT ROWID;",
        )?;
        store.ensure_journal_tables()?;
        store.write_header(header)?;
        Ok(store)
    }

    pub fn open(path: &Path) -> Result<Store> {
        if !path.exists() {
            return Err(Error::NoVault);
        }
        Self::connect(path)
    }

    fn connect(path: &Path) -> Result<Store> {
        let conn = Connection::open(path)?;
        // DELETE journal + FULL sync: every write reaches the disk before it is
        // reported done, so pulling a USB stick can't leave a half-written vault.
        // secure_delete overwrites freed pages, so deleted ciphertext doesn't linger.
        conn.execute_batch(
            "PRAGMA journal_mode = DELETE;
             PRAGMA synchronous = FULL;
             PRAGMA secure_delete = ON;
             PRAGMA temp_store = MEMORY;",
        )?;
        Ok(Store { conn })
    }

    pub fn read_header(&self) -> Result<Header> {
        let data: Option<Vec<u8>> = self
            .conn
            .query_row("SELECT data FROM header WHERE id = 1", [], |r| r.get(0))
            .optional()
            .map_err(|_| Error::Corrupt)?;
        Header::from_bytes(&data.ok_or(Error::Corrupt)?)
    }

    /// Called only after authentication when opening an older vault.
    pub fn ensure_journal_tables(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS history (id BLOB PRIMARY KEY, data BLOB NOT NULL) WITHOUT ROWID;
             CREATE TABLE IF NOT EXISTS activity (id BLOB PRIMARY KEY, data BLOB NOT NULL) WITHOUT ROWID;",
        )?;
        Ok(())
    }

    pub fn exists(&self, table: Table) -> Result<bool> {
        Ok(self.conn.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
            [table.name()], |r| r.get(0))?)
    }

    /// A history metadata row starts with 0; bodies start with 1. SQLite filters
    /// before reading blobs, so unlock never reads a version body.
    pub fn history_meta(&self) -> Result<Vec<(RowId, Vec<u8>)>> {
        let mut stmt = self.conn.prepare("SELECT id, data FROM history WHERE substr(id,1,1)=x'00'")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, Vec<u8>>(0)?, r.get::<_, Vec<u8>>(1)?)))?;
        rows.map(|r| { let (id, data) = r?; Ok((id.try_into().map_err(|_| Error::Corrupt)?, data)) }).collect()
    }

    pub fn write_header(&self, header: &Header) -> Result<()> {
        self.conn.execute(
            "INSERT INTO header (id, data) VALUES (1, ?1)
             ON CONFLICT(id) DO UPDATE SET data = excluded.data",
            params![header.to_bytes()],
        )?;
        Ok(())
    }

    pub fn put(&self, table: Table, id: &RowId, data: &[u8]) -> Result<()> {
        self.conn.execute(
            &format!(
                "INSERT INTO {} (id, data) VALUES (?1, ?2)
                 ON CONFLICT(id) DO UPDATE SET data = excluded.data",
                table.name()
            ),
            params![&id[..], data],
        )?;
        Ok(())
    }

    pub fn get(&self, table: Table, id: &RowId) -> Result<Option<Vec<u8>>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT data FROM {} WHERE id = ?1", table.name()),
                params![&id[..]],
                |r| r.get(0),
            )
            .optional()?)
    }

    pub fn delete(&self, table: Table, id: &RowId) -> Result<()> {
        self.conn.execute(
            &format!("DELETE FROM {} WHERE id = ?1", table.name()),
            params![&id[..]],
        )?;
        Ok(())
    }

    pub fn all(&self, table: Table) -> Result<Vec<(RowId, Vec<u8>)>> {
        let mut stmt = self.conn.prepare(&format!("SELECT id, data FROM {}", table.name()))?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, Vec<u8>>(0)?, r.get::<_, Vec<u8>>(1)?)))?;
        let mut out = Vec::new();
        for row in rows {
            let (id, data) = row?;
            let id: RowId = id.try_into().map_err(|_| Error::Corrupt)?;
            out.push((id, data));
        }
        Ok(out)
    }

    pub fn ids(&self, table: Table) -> Result<Vec<RowId>> {
        let mut stmt = self.conn.prepare(&format!("SELECT id FROM {}", table.name()))?;
        let rows = stmt.query_map([], |r| r.get::<_, Vec<u8>>(0))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?.try_into().map_err(|_| Error::Corrupt)?);
        }
        Ok(out)
    }

    /// Writes a consistent, compacted copy of the (still encrypted) vault to `dest`.
    pub fn backup_to(&self, dest: &Path) -> Result<()> {
        if dest.exists() {
            std::fs::remove_file(dest)?;
        }
        let dest = dest.to_str().ok_or_else(|| Error::invalid("Unsupported backup path."))?;
        self.conn.execute("VACUUM INTO ?1", params![dest])?;
        Ok(())
    }

    /// Runs several writes as one all-or-nothing transaction. If the commit
    /// itself fails, the transaction is rolled back so the connection is never
    /// left inside an open transaction.
    pub fn transaction<T>(&self, f: impl FnOnce(&Store) -> Result<T>) -> Result<T> {
        let nested = !self.conn.is_autocommit();
        self.conn.execute_batch(if nested { "SAVEPOINT store_write" } else { "BEGIN IMMEDIATE" })?;
        let rollback = if nested { "ROLLBACK TO store_write; RELEASE store_write" } else { "ROLLBACK" };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(self)));
        match result {
            Err(panic) => {
                let _ = self.conn.execute_batch(rollback);
                std::panic::resume_unwind(panic)
            }
            Ok(result) => match result {
            Ok(v) => {
                if let Err(e) = self.conn.execute_batch(if nested { "RELEASE store_write" } else { "COMMIT" }) {
                    let _ = self.conn.execute_batch(rollback);
                    return Err(e.into());
                }
                Ok(v)
            }
            Err(e) => {
                let _ = self.conn.execute_batch(rollback);
                Err(e)
            }
            }
        }
    }

    pub(super) fn begin_action(&self) -> Result<()> {
        self.conn.execute_batch("BEGIN IMMEDIATE")?;
        Ok(())
    }
    pub(super) fn commit_action(&self) -> Result<()> {
        self.conn.execute_batch("COMMIT")?;
        Ok(())
    }
    pub(super) fn rollback_action(&self) {
        let _ = self.conn.execute_batch("ROLLBACK");
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Table {
    Records,
    Attachments,
    History,
    Activity,
}

impl Table {
    fn name(self) -> &'static str {
        match self {
            Table::Records => "records",
            Table::Attachments => "attachments",
            Table::History => "history",
            Table::Activity => "activity",
        }
    }

    /// Domain-separation tag mixed into each row's AAD.
    pub fn tag(self) -> &'static [u8] {
        match self {
            Table::Records => b"rec",
            Table::Attachments => b"att",
            Table::History => b"his",
            Table::Activity => b"act",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A bare store (no header) in a fresh temp folder, with the two data tables.
    fn temp_store() -> Store {
        let dir = std::env::temp_dir().join(format!("notaya-store-{}", crate::model::new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        let store = Store::connect(&dir.join("test.clf")).unwrap();
        store
            .conn
            .execute_batch(
                "CREATE TABLE records (id BLOB PRIMARY KEY, data BLOB NOT NULL) WITHOUT ROWID;
                 CREATE TABLE attachments (id BLOB PRIMARY KEY, data BLOB NOT NULL) WITHOUT ROWID;",
            )
            .unwrap();
        store
    }

    #[test]
    fn failing_closure_rolls_back_and_store_keeps_working() {
        let store = temp_store();
        let id = [1u8; 16];
        let result: Result<()> = store.transaction(|s| {
            s.put(Table::Records, &id, b"first")?;
            Err(Error::Other("boom".into()))
        });
        assert!(result.is_err());
        assert!(store.get(Table::Records, &id).unwrap().is_none(), "write must not survive");
        // Not stuck inside the old transaction: a new one begins and commits.
        store.transaction(|s| s.put(Table::Records, &[2u8; 16], b"second")).unwrap();
        assert_eq!(store.get(Table::Records, &[2u8; 16]).unwrap().unwrap(), b"second");
        assert!(store.get(Table::Records, &id).unwrap().is_none());
    }

    #[test]
    fn failing_commit_rolls_back_and_store_keeps_working() {
        let store = temp_store();
        // A deferred foreign key violation makes COMMIT (not the statements) fail.
        store
            .conn
            .execute_batch(
                "PRAGMA foreign_keys = ON;
                 CREATE TABLE parent (id INTEGER PRIMARY KEY);
                 CREATE TABLE child (pid INTEGER REFERENCES parent(id) DEFERRABLE INITIALLY DEFERRED);",
            )
            .unwrap();
        let result: Result<()> = store.transaction(|s| {
            s.put(Table::Records, &[1u8; 16], b"lost")?;
            s.conn.execute_batch("INSERT INTO child (pid) VALUES (99)")?;
            Ok(())
        });
        assert!(result.is_err(), "the commit should have failed");
        assert!(store.get(Table::Records, &[1u8; 16]).unwrap().is_none());
        // A second BEGIN would fail if the failed commit had left a transaction open.
        store.transaction(|s| s.put(Table::Records, &[2u8; 16], b"ok")).unwrap();
        assert_eq!(store.get(Table::Records, &[2u8; 16]).unwrap().unwrap(), b"ok");
    }
}
