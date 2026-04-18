//! Note repository

use rusqlite::{params, Connection};
use crate::errors::{AppError, Result};
use crate::models::*;

pub struct NoteRepo;

// ── Core CRUD ────────────────────────────────────────────────────────────────

impl NoteRepo {
    pub fn insert(conn: &Connection, input: &CreateNoteInput) -> Result<Note> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
        let note_type = input.note_type.as_deref().unwrap_or("general");

        conn.execute(
            "INSERT INTO notes (id, paper_id, title, content, note_type, page_number, position, date_created, date_modified)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
            params![id, input.paper_id, input.title, input.content, note_type, input.page_number, input.position, now],
        )?;

        Self::get_by_id(conn, &id)
    }

    pub fn get_by_id(conn: &Connection, id: &str) -> Result<Note> {
        conn.query_row(
            "SELECT id, paper_id, title, content, note_type, page_number, position, date_created, date_modified
             FROM notes WHERE id = ?1",
            params![id],
            |row| Ok(Note {
                id: row.get(0)?,
                paper_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                note_type: row.get(4)?,
                page_number: row.get(5)?,
                position: row.get(6)?,
                date_created: row.get(7)?,
                date_modified: row.get(8)?,
            }),
        ).map_err(|_| AppError::NoteNotFound { id: id.to_string() })
    }

    pub fn list_by_paper(conn: &Connection, paper_id: &str) -> Result<Vec<Note>> {
        let mut stmt = conn.prepare(
            "SELECT id, paper_id, title, content, note_type, page_number, position, date_created, date_modified
             FROM notes WHERE paper_id = ?1 ORDER BY date_created ASC"
        )?;
        let notes = stmt.query_map(params![paper_id], |row| {
            Ok(Note {
                id: row.get(0)?,
                paper_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                note_type: row.get(4)?,
                page_number: row.get(5)?,
                position: row.get(6)?,
                date_created: row.get(7)?,
                date_modified: row.get(8)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(notes)
    }

    pub fn list(conn: &Connection, limit: u32, offset: u32) -> Result<Vec<Note>> {
        let mut stmt = conn.prepare(
            "SELECT id, paper_id, title, content, note_type, page_number, position, date_created, date_modified
             FROM notes ORDER BY date_modified DESC LIMIT ?1 OFFSET ?2"
        )?;
        let notes = stmt.query_map(params![limit, offset], |row| {
            Ok(Note {
                id: row.get(0)?,
                paper_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                note_type: row.get(4)?,
                page_number: row.get(5)?,
                position: row.get(6)?,
                date_created: row.get(7)?,
                date_modified: row.get(8)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(notes)
    }

    pub fn update(conn: &Connection, input: &UpdateNoteInput) -> Result<Note> {
        let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
        let mut updates = vec!["date_modified = ?1".to_string()];
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(now)];
        let mut idx = 2;

        if let Some(ref v) = input.title { updates.push(format!("title = ?{}", idx)); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(ref v) = input.content { updates.push(format!("content = ?{}", idx)); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(ref v) = input.note_type { updates.push(format!("note_type = ?{}", idx)); param_values.push(Box::new(v.clone())); idx += 1; }
        if let Some(v) = input.page_number { updates.push(format!("page_number = ?{}", idx)); param_values.push(Box::new(v)); idx += 1; }
        if let Some(ref v) = input.position { updates.push(format!("position = ?{}", idx)); param_values.push(Box::new(v.clone())); idx += 1; }

        param_values.push(Box::new(input.id.clone()));
        let sql = format!("UPDATE notes SET {} WHERE id = ?{}", updates.join(", "), idx);
        let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        let rows = conn.execute(&sql, params_refs.as_slice())?;
        if rows == 0 { return Err(AppError::NoteNotFound { id: input.id.clone() }); }
        Self::get_by_id(conn, &input.id)
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<()> {
        // note_links cascade on DELETE, so links are auto-cleaned
        let rows = conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
        if rows == 0 { return Err(AppError::NoteNotFound { id: id.to_string() }); }
        Ok(())
    }
}

// ── Bidirectional links ─────────────────────────────────────────────────────

impl NoteRepo {
    /// Create a directed link from source to target.
    /// The reverse direction is discovered via `list_backlinks(target)`.
    pub fn create_link(conn: &Connection, input: &CreateNoteLinkInput) -> Result<NoteLink> {
        let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();

        // Verify both notes exist
        Self::get_by_id(conn, &input.source_id)?;
        Self::get_by_id(conn, &input.target_id)?;

        conn.execute(
            "INSERT OR IGNORE INTO note_links (source_id, target_id, label, date_created)
             VALUES (?1, ?2, ?3, ?4)",
            params![input.source_id, input.target_id, input.label, now],
        )?;

        Ok(NoteLink {
            source_id: input.source_id.clone(),
            target_id: input.target_id.clone(),
            label: input.label.clone(),
            date_created: now,
        })
    }

    pub fn delete_link(conn: &Connection, source_id: &str, target_id: &str) -> Result<()> {
        let rows = conn.execute(
            "DELETE FROM note_links WHERE source_id = ?1 AND target_id = ?2",
            params![source_id, target_id],
        )?;
        if rows == 0 {
            return Err(AppError::Internal("Link not found".to_string()));
        }
        Ok(())
    }

    /// Notes that THIS note links TO (outgoing)
    pub fn list_outgoing_links(conn: &Connection, note_id: &str) -> Result<Vec<NoteLinkTarget>> {
        let mut stmt = conn.prepare(
            "SELECT n.id, n.title, nl.label
             FROM note_links nl
             JOIN notes n ON n.id = nl.target_id
             WHERE nl.source_id = ?1
             ORDER BY n.title ASC"
        )?;
        let links: Vec<NoteLinkTarget> = stmt
            .query_map(params![note_id], |row| {
                Ok(NoteLinkTarget {
                    note_id: row.get(0)?,
                    title: row.get(1)?,
                    label: row.get(2)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(links)
    }

    /// Notes that link TO this note (backlinks / incoming)
    pub fn list_backlinks(conn: &Connection, note_id: &str) -> Result<Vec<NoteLinkTarget>> {
        let mut stmt = conn.prepare(
            "SELECT n.id, n.title, nl.label
             FROM note_links nl
             JOIN notes n ON n.id = nl.source_id
             WHERE nl.target_id = ?1
             ORDER BY n.title ASC"
        )?;
        let links: Vec<NoteLinkTarget> = stmt
            .query_map(params![note_id], |row| {
                Ok(NoteLinkTarget {
                    note_id: row.get(0)?,
                    title: row.get(1)?,
                    label: row.get(2)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(links)
    }

    /// Get a note with its outgoing + backlinks
    pub fn get_with_links(conn: &Connection, id: &str) -> Result<NoteWithLinks> {
        let note = Self::get_by_id(conn, id)?;
        let outgoing_links = Self::list_outgoing_links(conn, id)?;
        let backlinks = Self::list_backlinks(conn, id)?;
        Ok(NoteWithLinks {
            note,
            outgoing_links,
            backlinks,
        })
    }
}

// ── Full-text search ────────────────────────────────────────────────────────

impl NoteRepo {
    /// Search notes using FTS5 with BM25 ranking and snippet highlights.
    /// Optionally filter by paper_id or note_type.
    pub fn search(
        conn: &Connection,
        query: &str,
        paper_id: Option<&str>,
        note_type: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<NoteSearchResult> {
        let raw_query = query.trim().to_string();
        if raw_query.is_empty() {
            return Ok(NoteSearchResult {
                items: vec![],
                total: 0,
                query: raw_query,
            });
        }

        // Build FTS5 query with prefix matching
        let fts_query = Self::sanitize_note_fts_query(&raw_query);

        // Extra WHERE clauses
        let mut extra_where: Vec<String> = Vec::new();
        if let Some(pid) = paper_id {
            extra_where.push(format!("n.paper_id = '{}'", pid.replace('\'', "''")));
        }
        if let Some(nt) = note_type {
            extra_where.push(format!("n.note_type = '{}'", nt.replace('\'', "''")));
        }
        let where_extra = if extra_where.is_empty() {
            String::new()
        } else {
            format!("AND {}", extra_where.join(" AND "))
        };

        let sql = format!(
            "SELECT
                n.id, n.paper_id, n.title, n.content, n.note_type,
                n.page_number, n.position, n.date_created, n.date_modified,
                bm25(notes_fts, 3.0, 1.0) AS score,
                snippet(notes_fts, 1, '<mark>', '</mark>', '…', 20) AS content_snip,
                snippet(notes_fts, 0, '<mark>', '</mark>', '…', 10) AS title_snip
             FROM notes_fts f
             JOIN notes n ON n.rowid = f.rowid
             WHERE notes_fts MATCH ?1
             {where_extra}
             ORDER BY score
             LIMIT ?2 OFFSET ?3",
            where_extra = where_extra
        );

        let limit_i64 = limit.min(200) as i64;
        let offset_i64 = offset as i64;

        let mut stmt = conn.prepare(&sql).map_err(|e| {
            AppError::Internal(format!("笔记搜索语法错误: {}", e))
        })?;

        let items: Vec<NoteSearchResultItem> = stmt
            .query_map(params![fts_query, limit_i64, offset_i64], |row| {
                let content_snip: Option<String> = row.get(10)?;
                let title_snip: Option<String> = row.get(11)?;
                let highlight = content_snip.or(title_snip);

                Ok(NoteSearchResultItem {
                    note: Note {
                        id: row.get(0)?,
                        paper_id: row.get(1)?,
                        title: row.get(2)?,
                        content: row.get(3)?,
                        note_type: row.get(4)?,
                        page_number: row.get(5)?,
                        position: row.get(6)?,
                        date_created: row.get(7)?,
                        date_modified: row.get(8)?,
                    },
                    score: row.get::<_, f64>(9)?,
                    highlight,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        // Count total
        let count_sql = format!(
            "SELECT COUNT(*) FROM notes_fts f
             JOIN notes n ON n.rowid = f.rowid
             WHERE notes_fts MATCH ?1
             {where_extra}",
            where_extra = where_extra
        );
        let total: u32 = conn
            .query_row(&count_sql, params![fts_query], |r| r.get(0))
            .unwrap_or(items.len() as u32);

        Ok(NoteSearchResult {
            items,
            total,
            query: raw_query,
        })
    }

    /// Quick prefix-match suggestions for [[link autocomplete]]
    pub fn search_suggest(conn: &Connection, prefix: &str, limit: u32) -> Result<Vec<NoteLinkTarget>> {
        let raw = prefix.trim();
        if raw.is_empty() {
            return Ok(vec![]);
        }

        let fts_query = format!("title: \"{}\"*", raw.replace('"', ""));

        let mut stmt = conn.prepare(
            "SELECT n.id, n.title, NULL as label
             FROM notes_fts f
             JOIN notes n ON n.rowid = f.rowid
             WHERE notes_fts MATCH ?1
             ORDER BY bm25(notes_fts, 5.0, 0.0)
             LIMIT ?2"
        )?;

        let items: Vec<NoteLinkTarget> = stmt
            .query_map(params![fts_query, limit.min(20)], |row| {
                Ok(NoteLinkTarget {
                    note_id: row.get(0)?,
                    title: row.get(1)?,
                    label: row.get(2)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(items)
    }

    /// Sanitize user input into a safe FTS5 query with prefix matching
    fn sanitize_note_fts_query(raw: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        let has_operators = trimmed.contains(" AND ")
            || trimmed.contains(" OR ")
            || trimmed.contains(" NOT ")
            || trimmed.contains('"')
            || trimmed.contains(':');

        if has_operators {
            trimmed.to_string()
        } else {
            let parts: Vec<String> = trimmed
                .split_whitespace()
                .map(|w| {
                    let escaped = w.replace('"', "");
                    format!("\"{}\"*", escaped)
                })
                .collect();
            parts.join(" ")
        }
    }
}
