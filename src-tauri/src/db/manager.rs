//! Database connection manager

use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::errors::Result;

pub struct DatabaseManager {
    conn: Connection,
}

/// Thread-safe wrapper for Tauri state.
/// Uses `Arc<std::sync::Mutex>` so the scheduler can share the same DB handle.
pub struct DbState(pub Arc<Mutex<DatabaseManager>>);

impl DatabaseManager {
    /// Create a new database manager, opening/creating the SQLite file at the given path.
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;")?;
        Ok(Self { conn })
    }

    /// Run all pending migrations.
    pub fn run_migrations(&self) -> Result<()> {
        super::migrations::run_migrations(&self.conn)
    }

    /// Get a reference to the underlying connection.
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Execute a query that doesn't return rows.
    pub fn execute(&self, sql: &str, params: &[&dyn rusqlite::types::ToSql]) -> Result<usize> {
        Ok(self.conn.execute(sql, params)?)
    }

    /// Begin a transaction.
    pub fn execute_batch(&self, sql: &str) -> Result<()> {
        Ok(self.conn.execute_batch(sql)?)
    }
}
