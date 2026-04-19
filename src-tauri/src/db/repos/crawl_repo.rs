//! Crawl repository - CRUD for watch_dirs and file_fingerprints

use rusqlite::{params, Connection};
use crate::errors::{AppError, Result};
use crate::models::*;

pub struct CrawlRepo;

// ── Watch directories ───────────────────────────────────────────────────────

impl CrawlRepo {
    pub fn add_watch_dir(conn: &Connection, input: &CreateWatchDirInput) -> Result<WatchDir> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();

        let is_enabled = input.is_enabled.unwrap_or(true) as i32;
        let recursive = input.recursive.unwrap_or(true) as i32;
        let scan_interval_min = input.scan_interval_min.unwrap_or(60);

        conn.execute(
            "INSERT INTO watch_dirs (id, path, is_enabled, file_filter, folder_id, recursive, scan_interval_min, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
            params![
                id, input.path, is_enabled, input.file_filter,
                input.folder_id, recursive, scan_interval_min, now,
            ],
        )?;

        Self::get_watch_dir(conn, &id)
    }

    pub fn get_watch_dir(conn: &Connection, id: &str) -> Result<WatchDir> {
        conn.query_row(
            "SELECT id, path, is_enabled, file_filter, folder_id, recursive, scan_interval_min, last_scan, created_at, updated_at
             FROM watch_dirs WHERE id = ?1",
            params![id],
            |row| Ok(WatchDir {
                id: row.get(0)?,
                path: row.get(1)?,
                is_enabled: row.get::<_, i32>(2)? != 0,
                file_filter: row.get(3)?,
                folder_id: row.get(4)?,
                recursive: row.get::<_, i32>(5)? != 0,
                scan_interval_min: row.get(6)?,
                last_scan: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            }),
        ).map_err(|_| AppError::InvalidInput(format!("Watch dir not found: {}", id)))
    }

    pub fn list_watch_dirs(conn: &Connection) -> Result<Vec<WatchDir>> {
        let mut stmt = conn.prepare(
            "SELECT id, path, is_enabled, file_filter, folder_id, recursive, scan_interval_min, last_scan, created_at, updated_at
             FROM watch_dirs ORDER BY created_at"
        )?;
        let dirs = stmt.query_map([], |row| {
            Ok(WatchDir {
                id: row.get(0)?,
                path: row.get(1)?,
                is_enabled: row.get::<_, i32>(2)? != 0,
                file_filter: row.get(3)?,
                folder_id: row.get(4)?,
                recursive: row.get::<_, i32>(5)? != 0,
                scan_interval_min: row.get(6)?,
                last_scan: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(dirs)
    }

    pub fn list_enabled_watch_dirs(conn: &Connection) -> Result<Vec<WatchDir>> {
        let mut stmt = conn.prepare(
            "SELECT id, path, is_enabled, file_filter, folder_id, recursive, scan_interval_min, last_scan, created_at, updated_at
             FROM watch_dirs WHERE is_enabled = 1 ORDER BY created_at"
        )?;
        let dirs = stmt.query_map([], |row| {
            Ok(WatchDir {
                id: row.get(0)?,
                path: row.get(1)?,
                is_enabled: row.get::<_, i32>(2)? != 0,
                file_filter: row.get(3)?,
                folder_id: row.get(4)?,
                recursive: row.get::<_, i32>(5)? != 0,
                scan_interval_min: row.get(6)?,
                last_scan: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(dirs)
    }

    pub fn update_watch_dir(conn: &Connection, input: &UpdateWatchDirInput) -> Result<WatchDir> {
        let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();

        let mut updates = vec!["updated_at = ?1".to_string()];
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(now)];
        let mut idx = 2;

        if let Some(ref v) = input.path { updates.push(format!("path = ?{}", idx)); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(v) = input.is_enabled { updates.push(format!("is_enabled = ?{}", idx)); param_values.push(Box::new(v as i32)); idx += 1; }
        if let Some(ref v) = input.file_filter { updates.push(format!("file_filter = ?{}", idx)); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(ref v) = input.folder_id { updates.push(format!("folder_id = ?{}", idx)); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(v) = input.recursive { updates.push(format!("recursive = ?{}", idx)); param_values.push(Box::new(v as i32)); idx += 1; }
        if let Some(v) = input.scan_interval_min { updates.push(format!("scan_interval_min = ?{}", idx)); param_values.push(Box::new(v)); idx += 1; }

        param_values.push(Box::new(input.id.clone()));

        let sql = format!("UPDATE watch_dirs SET {} WHERE id = ?{}", updates.join(", "), idx);
        let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        conn.execute(&sql, params_refs.as_slice())?;

        Self::get_watch_dir(conn, &input.id)
    }

    pub fn delete_watch_dir(conn: &Connection, id: &str) -> Result<()> {
        conn.execute("DELETE FROM watch_dirs WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_last_scan(conn: &Connection, id: &str) -> Result<()> {
        let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "UPDATE watch_dirs SET last_scan = ?1, updated_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }
}

// ── File fingerprints ───────────────────────────────────────────────────────

impl CrawlRepo {
    pub fn get_fingerprint(conn: &Connection, file_path: &str) -> Result<Option<FileFingerprint>> {
        conn.query_row(
            "SELECT id, file_path, watch_dir_id, file_size, modified_time, content_hash, is_imported, paper_id, discovered_at, last_checked
             FROM file_fingerprints WHERE file_path = ?1",
            params![file_path],
            |row| Ok(FileFingerprint {
                id: row.get(0)?,
                file_path: row.get(1)?,
                watch_dir_id: row.get(2)?,
                file_size: row.get(3)?,
                modified_time: row.get(4)?,
                content_hash: row.get(5)?,
                is_imported: row.get::<_, i32>(6)? != 0,
                paper_id: row.get(7)?,
                discovered_at: row.get(8)?,
                last_checked: row.get(9)?,
            }),
        ).ok().map_or(Ok(None), |v| Ok(Some(v)))
    }

    pub fn upsert_fingerprint(conn: &Connection, fp: &FileFingerprint) -> Result<()> {
        let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO file_fingerprints (id, file_path, watch_dir_id, file_size, modified_time, content_hash, is_imported, paper_id, discovered_at, last_checked)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(file_path) DO UPDATE SET
                file_size = excluded.file_size,
                modified_time = excluded.modified_time,
                content_hash = excluded.content_hash,
                is_imported = excluded.is_imported,
                paper_id = excluded.paper_id,
                last_checked = excluded.last_checked",
            params![
                fp.id, fp.file_path, fp.watch_dir_id, fp.file_size,
                fp.modified_time, fp.content_hash, fp.is_imported as i32,
                fp.paper_id, fp.discovered_at, now,
            ],
        )?;
        Ok(())
    }

    pub fn mark_imported(conn: &Connection, file_path: &str, paper_id: &str) -> Result<()> {
        let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "UPDATE file_fingerprints SET is_imported = 1, paper_id = ?1, last_checked = ?2 WHERE file_path = ?3",
            params![paper_id, now, file_path],
        )?;
        Ok(())
    }

    /// Delete fingerprints belonging to a watch dir
    pub fn delete_fingerprints_by_watch_dir(conn: &Connection, watch_dir_id: &str) -> Result<()> {
        conn.execute(
            "DELETE FROM file_fingerprints WHERE watch_dir_id = ?1",
            params![watch_dir_id],
        )?;
        Ok(())
    }

    /// List all fingerprints for a watch dir
    pub fn list_fingerprints_by_watch_dir(conn: &Connection, watch_dir_id: &str) -> Result<Vec<FileFingerprint>> {
        let mut stmt = conn.prepare(
            "SELECT id, file_path, watch_dir_id, file_size, modified_time, content_hash, is_imported, paper_id, discovered_at, last_checked
             FROM file_fingerprints WHERE watch_dir_id = ?1"
        )?;
        let fps = stmt.query_map(params![watch_dir_id], |row| {
            Ok(FileFingerprint {
                id: row.get(0)?,
                file_path: row.get(1)?,
                watch_dir_id: row.get(2)?,
                file_size: row.get(3)?,
                modified_time: row.get(4)?,
                content_hash: row.get(5)?,
                is_imported: row.get::<_, i32>(6)? != 0,
                paper_id: row.get(7)?,
                discovered_at: row.get(8)?,
                last_checked: row.get(9)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(fps)
    }
}
