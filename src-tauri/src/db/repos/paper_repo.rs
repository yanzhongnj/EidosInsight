//! Paper repository - CRUD operations for papers

use rusqlite::{params, Connection};
use crate::errors::{AppError, Result};
use crate::models::*;

pub struct PaperRepo;

impl PaperRepo {
    pub fn insert(conn: &Connection, input: &CreatePaperInput) -> Result<Paper> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();

        conn.execute(
            "INSERT INTO papers (id, title, authors, abstract_text, publication_date, journal,
                volume, issue, pages, doi, arxiv_id, pdf_url, pdf_path, folder_id, date_added, date_modified)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?15)",
            params![
                id, input.title, input.authors, input.abstract_text,
                input.publication_date, input.journal, input.volume, input.issue,
                input.pages, input.doi, input.arxiv_id, input.pdf_url, input.pdf_path,
                input.folder_id, now,
            ],
        )?;

        Self::get_by_id(conn, &id)
    }

    pub fn get_by_id(conn: &Connection, id: &str) -> Result<Paper> {
        let mut stmt = conn.prepare(
            "SELECT id, title, authors, abstract_text, publication_date, journal,
                    volume, issue, pages, doi, arxiv_id, pdf_url, pdf_path,
                    folder_id, tldr, reading_status, is_favorite, date_added, date_modified, cover_image
             FROM papers WHERE id = ?1"
        )?;
        let paper = stmt.query_row(params![id], |row| {
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
                folder_id: row.get(13)?,
                tldr: row.get(14)?,
                reading_status: row.get(15)?,
                is_favorite: row.get::<_, i32>(16)? != 0,
                date_added: row.get(17)?,
                date_modified: row.get(18)?,
                cover_image: row.get(19)?,
            })
        }).map_err(|_| AppError::PaperNotFound { id: id.to_string() })?;
        Ok(paper)
    }

    pub fn list(conn: &Connection, filter: &PaperFilter) -> Result<PaperListResult> {
        let mut sql = String::from(
            "SELECT id, title, authors, abstract_text, publication_date, journal,
                    volume, issue, pages, doi, arxiv_id, pdf_url, pdf_path,
                    folder_id, tldr, reading_status, is_favorite, date_added, date_modified, cover_image
             FROM papers WHERE 1=1"
        );
        let mut count_sql = String::from("SELECT COUNT(*) FROM papers WHERE 1=1");
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut param_idx = 1;

        if let Some(ref folder_id) = filter.folder_id {
            sql.push_str(&format!(" AND folder_id = ?{}", param_idx));
            count_sql.push_str(&format!(" AND folder_id = ?{}", param_idx));
            param_values.push(Box::new(folder_id.clone()));
            param_idx += 1;
        }

        if let Some(ref reading_status) = filter.reading_status {
            sql.push_str(&format!(" AND reading_status = ?{}", param_idx));
            count_sql.push_str(&format!(" AND reading_status = ?{}", param_idx));
            param_values.push(Box::new(reading_status.clone()));
            param_idx += 1;
        }

        if let Some(is_favorite) = filter.is_favorite {
            sql.push_str(&format!(" AND is_favorite = ?{}", param_idx));
            count_sql.push_str(&format!(" AND is_favorite = ?{}", param_idx));
            param_values.push(Box::new(is_favorite as i32));
            param_idx += 1;
        }

        if let Some(ref query) = filter.query {
            if !query.is_empty() {
                sql.push_str(&format!(" AND (title LIKE ?{} OR authors LIKE ?{} OR abstract_text LIKE ?{})", param_idx, param_idx, param_idx));
                count_sql.push_str(&format!(" AND (title LIKE ?{} OR authors LIKE ?{} OR abstract_text LIKE ?{})", param_idx, param_idx, param_idx));
                let pattern = format!("%{}%", query);
                param_values.push(Box::new(pattern));
                param_idx += 1;
            }
        }

        // Count total
        let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        let total: u32 = conn.query_row(&count_sql, params_refs.as_slice(), |row| row.get(0))?;

        // Sort
        let sort_by = filter.sort_by.as_deref().unwrap_or("date_added");
        let sort_order = filter.sort_order.as_deref().unwrap_or("desc");
        let sort_column = match sort_by {
            "title" => "title",
            "date_modified" => "date_modified",
            _ => "date_added",
        };
        sql.push_str(&format!(" ORDER BY {} {}", sort_column, sort_order));

        // Pagination
        let limit = filter.limit.unwrap_or(50);
        let offset = filter.offset.unwrap_or(0);
        sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        let mut stmt = conn.prepare(&sql)?;
        let papers = stmt.query_map(params_refs.as_slice(), |row| {
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
                folder_id: row.get(13)?,
                tldr: row.get(14)?,
                reading_status: row.get(15)?,
                is_favorite: row.get::<_, i32>(16)? != 0,
                date_added: row.get(17)?,
                date_modified: row.get(18)?,
                cover_image: row.get(19)?,
            })
        })?.filter_map(|r| r.ok()).collect();

        Ok(PaperListResult { papers, total })
    }

    pub fn update(conn: &Connection, input: &UpdatePaperInput) -> Result<Paper> {
        let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();

        let mut updates = vec!["date_modified = ?1".to_string()];
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(now)];
        let mut idx = 2;

        if let Some(ref v) = input.title { updates.push(format!("title = ?{}", idx)); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(ref v) = input.authors { updates.push(format!("authors = ?{}", idx)); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(ref v) = input.abstract_text { updates.push(format!("abstract_text = ?{}", idx)); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(ref v) = input.publication_date { updates.push(format!("publication_date = ?{}", idx)); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(ref v) = input.journal { updates.push(format!("journal = ?{}", idx)); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(ref v) = input.doi { updates.push(format!("doi = ?{}", idx)); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(ref v) = input.arxiv_id { updates.push(format!("arxiv_id = ?{}", idx)); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(ref v) = input.tldr { updates.push(format!("tldr = ?{}", idx)); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(ref v) = input.reading_status { updates.push(format!("reading_status = ?{}", idx)); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(v) = input.is_favorite { updates.push(format!("is_favorite = ?{}", idx)); param_values.push(Box::new(v as i32)); idx += 1; }
        if let Some(ref v) = input.folder_id { updates.push(format!("folder_id = ?{}", idx)); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(ref v) = input.cover_image { updates.push(format!("cover_image = ?{}", idx)); param_values.push(Box::new(v.clone())); idx += 1; }

        param_values.push(Box::new(input.id.clone()));

        let sql = format!("UPDATE papers SET {} WHERE id = ?{}", updates.join(", "), idx);
        let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        let rows = conn.execute(&sql, params_refs.as_slice())?;

        if rows == 0 {
            return Err(AppError::PaperNotFound { id: input.id.clone() });
        }

        Self::get_by_id(conn, &input.id)
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<()> {
        let rows = conn.execute("DELETE FROM papers WHERE id = ?1", params![id])?;
        if rows == 0 {
            return Err(AppError::PaperNotFound { id: id.to_string() });
        }
        Ok(())
    }
}
