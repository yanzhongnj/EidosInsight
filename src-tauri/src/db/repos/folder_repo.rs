//! Folder repository

use rusqlite::{params, Connection};
use crate::errors::{AppError, Result};
use crate::models::*;

pub struct FolderRepo;

impl FolderRepo {
    pub fn insert(conn: &Connection, input: &CreateFolderInput) -> Result<Folder> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();

        // Get next sort order
        let max_order: i32 = conn.query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM folders WHERE parent_id IS ?1",
            params![input.parent_id],
            |row| row.get(0),
        ).unwrap_or(-1);

        conn.execute(
            "INSERT INTO folders (id, name, parent_id, color, icon, sort_order, date_created, date_modified)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
            params![id, input.name, input.parent_id, input.color, input.icon, max_order + 1, now],
        )?;

        Self::get_by_id(conn, &id)
    }

    pub fn get_by_id(conn: &Connection, id: &str) -> Result<Folder> {
        conn.query_row(
            "SELECT id, name, parent_id, color, icon, sort_order, date_created, date_modified
             FROM folders WHERE id = ?1",
            params![id],
            |row| Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                color: row.get(3)?,
                icon: row.get(4)?,
                sort_order: row.get(5)?,
                date_created: row.get(6)?,
                date_modified: row.get(7)?,
            }),
        ).map_err(|_| AppError::FolderNotFound { id: id.to_string() })
    }

    pub fn list(conn: &Connection) -> Result<Vec<Folder>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, parent_id, color, icon, sort_order, date_created, date_modified
             FROM folders ORDER BY sort_order ASC"
        )?;
        let folders = stmt.query_map([], |row| {
            Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                color: row.get(3)?,
                icon: row.get(4)?,
                sort_order: row.get(5)?,
                date_created: row.get(6)?,
                date_modified: row.get(7)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(folders)
    }

    pub fn update(conn: &Connection, input: &UpdateFolderInput) -> Result<Folder> {
        let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
        let mut updates = vec!["date_modified = ?1".to_string()];
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(now)];
        let mut idx = 2;

        if let Some(ref v) = input.name { updates.push(format!("name = ?{}", idx)); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(ref v) = input.parent_id { updates.push(format!("parent_id = ?{}", idx)); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(ref v) = input.color { updates.push(format!("color = ?{}", idx)); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(ref v) = input.icon { updates.push(format!("icon = ?{}", idx)); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(v) = input.sort_order { updates.push(format!("sort_order = ?{}", idx)); param_values.push(Box::new(v)); idx += 1; }

        param_values.push(Box::new(input.id.clone()));
        let sql = format!("UPDATE folders SET {} WHERE id = ?{}", updates.join(", "), idx);
        let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        let rows = conn.execute(&sql, params_refs.as_slice())?;
        if rows == 0 { return Err(AppError::FolderNotFound { id: input.id.clone() }); }
        Self::get_by_id(conn, &input.id)
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<()> {
        let rows = conn.execute("DELETE FROM folders WHERE id = ?1", params![id])?;
        if rows == 0 { return Err(AppError::FolderNotFound { id: id.to_string() }); }
        Ok(())
    }
}
