//! Reading state repository

use rusqlite::{params, Connection};
use crate::errors::Result;
use crate::models::*;

pub struct ReadingRepo;

impl ReadingRepo {
    pub fn upsert(conn: &Connection, input: &UpdateReadingStateInput) -> Result<ReadingState> {
        let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();

        // Try to get existing state
        let existing = Self::get_by_paper(conn, &input.paper_id).ok();

        let (current_page, total_pages, scroll_position, zoom_level) = match &existing {
            Some(s) => (
                input.current_page.unwrap_or(s.current_page),
                input.total_pages.unwrap_or(s.total_pages),
                input.scroll_position.unwrap_or(s.scroll_position),
                input.zoom_level.unwrap_or(s.zoom_level),
            ),
            None => (
                input.current_page.unwrap_or(0),
                input.total_pages.unwrap_or(0),
                input.scroll_position.unwrap_or(0.0),
                input.zoom_level.unwrap_or(1.0),
            ),
        };

        let progress = if total_pages > 0 {
            (current_page as f64) / (total_pages as f64)
        } else {
            0.0
        };

        if existing.is_some() {
            conn.execute(
                "UPDATE reading_states SET current_page = ?1, total_pages = ?2,
                    scroll_position = ?3, zoom_level = ?4, last_read_at = ?5, reading_progress = ?6
                 WHERE paper_id = ?7",
                params![current_page, total_pages, scroll_position, zoom_level, now, progress, input.paper_id],
            )?;
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO reading_states (id, paper_id, current_page, total_pages,
                    scroll_position, zoom_level, last_read_at, reading_progress)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![id, input.paper_id, current_page, total_pages, scroll_position, zoom_level, now, progress],
            )?;
        }

        Self::get_by_paper(conn, &input.paper_id)
    }

    pub fn get_by_paper(conn: &Connection, paper_id: &str) -> Result<ReadingState> {
        conn.query_row(
            "SELECT id, paper_id, current_page, total_pages, scroll_position, zoom_level, last_read_at, reading_progress
             FROM reading_states WHERE paper_id = ?1",
            params![paper_id],
            |row| Ok(ReadingState {
                id: row.get(0)?,
                paper_id: row.get(1)?,
                current_page: row.get(2)?,
                total_pages: row.get(3)?,
                scroll_position: row.get(4)?,
                zoom_level: row.get(5)?,
                last_read_at: row.get(6)?,
                reading_progress: row.get(7)?,
            }),
        ).map_err(|e| crate::errors::AppError::Database(e))
    }

    pub fn list_reading_progress(conn: &Connection, limit: u32) -> Result<Vec<ReadingState>> {
        let mut stmt = conn.prepare(
            "SELECT id, paper_id, current_page, total_pages, scroll_position, zoom_level, last_read_at, reading_progress
             FROM reading_states ORDER BY last_read_at DESC LIMIT ?1"
        )?;
        let states = stmt.query_map(params![limit], |row| {
            Ok(ReadingState {
                id: row.get(0)?,
                paper_id: row.get(1)?,
                current_page: row.get(2)?,
                total_pages: row.get(3)?,
                scroll_position: row.get(4)?,
                zoom_level: row.get(5)?,
                last_read_at: row.get(6)?,
                reading_progress: row.get(7)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(states)
    }
}
