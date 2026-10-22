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

    /// Runs several writes as one all-or-nothing transaction.
    pub fn transaction<T>(&self, f: impl FnOnce(&Store) -> Result<T>) -> Result<T> {
        self.conn.execute_batch("BEGIN IMMEDIATE")?;
        match f(self) {
            Ok(v) => {
                self.conn.execute_batch("COMMIT")?;
                Ok(v)
            }
            Err(e) => {
                let _ = self.conn.execute_batch("ROLLBACK");
                Err(e)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Table {
    Records,
    Attachments,
}

impl Table {
    fn name(self) -> &'static str {
        match self {
            Table::Records => "records",
            Table::Attachments => "attachments",
        }
    }

    /// Domain-separation tag mixed into each row's AAD.
    pub fn tag(self) -> &'static [u8] {
        match self {
            Table::Records => b"rec",
            Table::Attachments => b"att",
        }
    }
}
