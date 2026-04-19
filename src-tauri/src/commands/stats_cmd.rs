//! Library statistics command handlers

use crate::db::DbState;
use crate::errors::Result;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryStats {
    pub total_papers: u32,
    pub unread_count: u32,
    pub reading_count: u32,
    pub read_count: u32,
    pub favorite_count: u32,
    pub folder_count: u32,
    pub tag_count: u32,
    pub note_count: u32,
    pub collection_count: u32,
    /// Number of papers added in the last 7 days
    pub recent_papers_count: u32,
    /// Number of papers added in the last 30 days
    pub monthly_papers_count: u32,
    /// File type distribution: file_type → count
    pub file_type_distribution: Vec<FileTypeCount>,
    /// Top 5 most used tags
    pub top_tags: Vec<TagCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeCount {
    pub file_type: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagCount {
    pub name: String,
    pub count: u32,
}

#[tauri::command]
pub fn get_library_stats(db: State<'_, DbState>) -> Result<LibraryStats> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    let conn = db.conn();

    let total_papers: u32 = conn.query_row(
        "SELECT COUNT(*) FROM papers", [], |row| row.get(0)
    ).unwrap_or(0);

    let unread_count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM papers WHERE reading_status = 'unread'", [], |row| row.get(0)
    ).unwrap_or(0);

    let reading_count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM papers WHERE reading_status = 'reading'", [], |row| row.get(0)
    ).unwrap_or(0);

    let read_count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM papers WHERE reading_status = 'read'", [], |row| row.get(0)
    ).unwrap_or(0);

    let favorite_count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM papers WHERE is_favorite = 1", [], |row| row.get(0)
    ).unwrap_or(0);

    let folder_count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM folders", [], |row| row.get(0)
    ).unwrap_or(0);

    let tag_count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM tags", [], |row| row.get(0)
    ).unwrap_or(0);

    let note_count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM notes", [], |row| row.get(0)
    ).unwrap_or(0);

    let collection_count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM collections", [], |row| row.get(0)
    ).unwrap_or(0);

    // Papers added in last 7 days
    let recent_papers_count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM papers WHERE date_added >= datetime('now', '-7 days')",
        [], |row| row.get(0)
    ).unwrap_or(0);

    // Papers added in last 30 days
    let monthly_papers_count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM papers WHERE date_added >= datetime('now', '-30 days')",
        [], |row| row.get(0)
    ).unwrap_or(0);

    // File type distribution
    let mut ft_stmt = conn.prepare(
        "SELECT COALESCE(file_type, 'pdf') as ft, COUNT(*) as cnt FROM papers GROUP BY ft ORDER BY cnt DESC"
    )?;
    let file_type_distribution = ft_stmt.query_map([], |row| {
        Ok(FileTypeCount {
            file_type: row.get(0)?,
            count: row.get(1)?,
        })
    })?.filter_map(|r| r.ok()).collect();

    // Top 5 tags
    let mut tag_stmt = conn.prepare(
        "SELECT name, usage_count FROM tags ORDER BY usage_count DESC LIMIT 5"
    )?;
    let top_tags = tag_stmt.query_map([], |row| {
        Ok(TagCount {
            name: row.get(0)?,
            count: row.get(1)?,
        })
    })?.filter_map(|r| r.ok()).collect();

    Ok(LibraryStats {
        total_papers,
        unread_count,
        reading_count,
        read_count,
        favorite_count,
        folder_count,
        tag_count,
        note_count,
        collection_count,
        recent_papers_count,
        monthly_papers_count,
        file_type_distribution,
        top_tags,
    })
}
