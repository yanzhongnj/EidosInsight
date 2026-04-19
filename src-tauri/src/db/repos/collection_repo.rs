//! Collection repository

use rusqlite::{params, Connection};
use crate::errors::{AppError, Result};
use crate::models::*;

pub struct CollectionRepo;

impl CollectionRepo {
    pub fn insert(conn: &Connection, input: &CreateCollectionInput) -> Result<Collection> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();

        let max_order: i32 = conn.query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM collections",
            [],
            |row| row.get(0),
        ).unwrap_or(-1);

        conn.execute(
            "INSERT INTO collections (id, name, description, color, icon, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
            params![id, input.name, input.description, input.color, input.icon, max_order + 1, now],
        )?;

        Self::get_by_id(conn, &id)
    }

    pub fn get_by_id(conn: &Connection, id: &str) -> Result<Collection> {
        conn.query_row(
            "SELECT c.id, c.name, c.description, c.color, c.icon, c.sort_order, c.created_at, c.updated_at,
                    (SELECT COUNT(*) FROM collection_papers cp WHERE cp.collection_id = c.id) as paper_count
             FROM collections c WHERE c.id = ?1",
            params![id],
            |row| Ok(Collection {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                color: row.get(3)?,
                icon: row.get(4)?,
                sort_order: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                paper_count: Some(row.get(8)?),
            }),
        ).map_err(|_| AppError::InvalidInput(format!("Collection not found: {id}")))
    }

    pub fn list(conn: &Connection) -> Result<Vec<Collection>> {
        let mut stmt = conn.prepare(
            "SELECT c.id, c.name, c.description, c.color, c.icon, c.sort_order, c.created_at, c.updated_at,
                    (SELECT COUNT(*) FROM collection_papers cp WHERE cp.collection_id = c.id) as paper_count
             FROM collections c ORDER BY c.sort_order ASC"
        )?;
        let collections = stmt.query_map([], |row| {
            Ok(Collection {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                color: row.get(3)?,
                icon: row.get(4)?,
                sort_order: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                paper_count: Some(row.get(8)?),
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(collections)
    }

    pub fn update(conn: &Connection, input: &UpdateCollectionInput) -> Result<Collection> {
        let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
        let mut updates = vec!["updated_at = ?1".to_string()];
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(now)];
        let mut idx = 2;

        if let Some(ref v) = input.name { updates.push(format!("name = ?{idx}")); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(ref v) = input.description { updates.push(format!("description = ?{idx}")); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(ref v) = input.color { updates.push(format!("color = ?{idx}")); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(ref v) = input.icon { updates.push(format!("icon = ?{idx}")); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(v) = input.sort_order { updates.push(format!("sort_order = ?{idx}")); param_values.push(Box::new(v)); idx += 1; }

        param_values.push(Box::new(input.id.clone()));
        let sql = format!("UPDATE collections SET {} WHERE id = ?{idx}", updates.join(", "));
        let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        let rows = conn.execute(&sql, params_refs.as_slice())?;
        if rows == 0 { return Err(AppError::InvalidInput(format!("Collection not found: {}", input.id))); }
        Self::get_by_id(conn, &input.id)
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<()> {
        let rows = conn.execute("DELETE FROM collections WHERE id = ?1", params![id])?;
        if rows == 0 { return Err(AppError::InvalidInput(format!("Collection not found: {id}"))); }
        Ok(())
    }

    // ── Collection ↔ Paper ──────────────────────────────────────────────────

    pub fn add_paper(conn: &Connection, collection_id: &str, paper_id: &str) -> Result<()> {
        let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT OR IGNORE INTO collection_papers (collection_id, paper_id, date_added) VALUES (?1, ?2, ?3)",
            params![collection_id, paper_id, now],
        )?;
        Ok(())
    }

    pub fn remove_paper(conn: &Connection, collection_id: &str, paper_id: &str) -> Result<()> {
        conn.execute(
            "DELETE FROM collection_papers WHERE collection_id = ?1 AND paper_id = ?2",
            params![collection_id, paper_id],
        )?;
        Ok(())
    }

    pub fn list_papers(conn: &Connection, collection_id: &str) -> Result<Vec<Paper>> {
        let mut stmt = conn.prepare(
            "SELECT p.id, p.title, p.authors, p.abstract_text, p.publication_date, p.journal,
                    p.volume, p.issue, p.pages, p.doi, p.arxiv_id, p.pdf_url, p.pdf_path,
                    p.file_type, p.folder_id, p.tldr, p.reading_status, p.is_favorite,
                    p.date_added, p.date_modified, p.cover_image
             FROM papers p
             INNER JOIN collection_papers cp ON p.id = cp.paper_id
             WHERE cp.collection_id = ?1
             ORDER BY cp.date_added DESC"
        )?;
        let papers = stmt.query_map(params![collection_id], |row| {
            Ok(Paper {
                id: row.get(0)?,
                title: row.get(1)?,
                authors: row.get(2)?,
                abstract_text: row.get(3)?,
                publication_date: row.get(4)?,
                journal: row.get(5)?,
                volume: row.get(6)?,
                issue: row.get(7)?,
                pages: row.get(8)?,
                doi: row.get(9)?,
                arxiv_id: row.get(10)?,
                pdf_url: row.get(11)?,
                pdf_path: row.get(12)?,
                file_type: row.get(13)?,
                folder_id: row.get(14)?,
                tldr: row.get(15)?,
                reading_status: row.get(16)?,
                is_favorite: row.get(17)?,
                date_added: row.get(18)?,
                date_modified: row.get(19)?,
                cover_image: row.get(20)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(papers)
    }

    pub fn add_papers_batch(conn: &Connection, collection_id: &str, paper_ids: &[String]) -> Result<u32> {
        let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
        let mut count = 0u32;
        for pid in paper_ids {
            let rows = conn.execute(
                "INSERT OR IGNORE INTO collection_papers (collection_id, paper_id, date_added) VALUES (?1, ?2, ?3)",
                params![collection_id, pid, now],
            )?;
            count += rows as u32;
        }
        Ok(count)
    }
}
