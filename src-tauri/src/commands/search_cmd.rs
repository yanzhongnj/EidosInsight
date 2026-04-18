//! Search command handlers — FTS5 full-text search with highlight, suggestions, and advanced filters

use crate::db::DbState;
use crate::errors::{AppError, Result};
use crate::models::*;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

// ─── Input/Output Types ────────────────────────────────────────────────────

/// Advanced search filter parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    /// FTS5 query string (supports AND, OR, NOT, phrase "...", prefix *)
    pub query: String,
    /// Filter by reading status
    pub reading_status: Option<String>,
    /// Filter by folder ID
    pub folder_id: Option<String>,
    /// Filter by tag name
    pub tag: Option<String>,
    /// Filter to favorites only
    pub is_favorite: Option<bool>,
    /// Published year range start (inclusive), e.g. "2020"
    pub year_from: Option<String>,
    /// Published year range end (inclusive), e.g. "2024"
    pub year_to: Option<String>,
    /// Which fields to search: "all" | "title" | "abstract" | "authors"
    pub search_field: Option<String>,
    /// Max results to return (default 20)
    pub limit: Option<u32>,
    /// Offset for pagination
    pub offset: Option<u32>,
}

/// Single suggestion item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionItem {
    pub id: String,
    pub title: String,
    pub authors: Option<String>,
    pub publication_date: Option<String>,
}

/// Search suggestion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestResult {
    pub items: Vec<SuggestionItem>,
    pub query: String,
}

// ─── Helper: FTS5 query sanitization ──────────────────────────────────────

/// Escape special FTS5 characters to prevent injection and syntax errors.
/// Allows: AND, OR, NOT, phrase quotes, prefix *, field: prefix.
fn sanitize_fts_query(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // If user typed a simple term (no FTS operators), auto-add prefix wildcard
    // so "transf" matches "transformer", "transformers", etc.
    let has_operators = trimmed.contains(" AND ")
        || trimmed.contains(" OR ")
        || trimmed.contains(" NOT ")
        || trimmed.contains('"')
        || trimmed.contains(':');

    if has_operators {
        trimmed.to_string()
    } else {
        // Multi-word: each word becomes a required prefix term
        let parts: Vec<String> = trimmed
            .split_whitespace()
            .map(|w| {
                // Escape special chars inside term
                let escaped = w.replace('"', "");
                format!("\"{}\"*", escaped)
            })
            .collect();
        parts.join(" ")
    }
}

// ─── Main full-text search ─────────────────────────────────────────────────

/// Full-text search using FTS5 with BM25 ranking and snippet highlights.
/// The snippet function extracts a ~30-token context window with <mark> tags.
#[tauri::command]
pub fn search_papers(query: String, limit: Option<u32>, db: State<'_, DbState>) -> Result<SearchResult> {
    let filters = SearchFilters {
        query,
        limit,
        reading_status: None,
        folder_id: None,
        tag: None,
        is_favorite: None,
        year_from: None,
        year_to: None,
        search_field: None,
        offset: None,
    };
    search_advanced(filters, db)
}

/// Alias kept for compatibility
#[tauri::command]
pub fn search_fulltext(query: String, limit: Option<u32>, db: State<'_, DbState>) -> Result<SearchResult> {
    search_papers(query, limit, db)
}

// ─── Advanced search ────────────────────────────────────────────────────────

/// Advanced search with filters, field restriction, and snippet highlights.
#[tauri::command]
pub fn search_advanced(filters: SearchFilters, db: State<'_, DbState>) -> Result<SearchResult> {
    let db = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let limit = filters.limit.unwrap_or(20).min(200) as i64;
    let offset = filters.offset.unwrap_or(0) as i64;
    let raw_query = filters.query.trim().to_string();

    if raw_query.is_empty() {
        return Ok(SearchResult {
            papers: vec![],
            total: 0,
            query: raw_query,
            search_type: "fulltext".to_string(),
        });
    }

    // Build the FTS5 query, optionally scoped to a single field
    let fts_query = match filters.search_field.as_deref() {
        Some("title") => format!("title: {}", sanitize_fts_query(&raw_query)),
        Some("abstract") => format!("abstract_text: {}", sanitize_fts_query(&raw_query)),
        Some("authors") => format!("authors: {}", sanitize_fts_query(&raw_query)),
        _ => sanitize_fts_query(&raw_query),
    };

    // Build WHERE clauses for additional filters
    let mut extra_where: Vec<String> = Vec::new();
    if let Some(ref status) = filters.reading_status {
        extra_where.push(format!("p.reading_status = '{}'", status.replace('\'', "''")));
    }
    if let Some(ref folder) = filters.folder_id {
        extra_where.push(format!("p.folder_id = '{}'", folder.replace('\'', "''")));
    }
    if let Some(fav) = filters.is_favorite {
        extra_where.push(format!("p.is_favorite = {}", if fav { 1 } else { 0 }));
    }
    if let Some(ref yr) = filters.year_from {
        extra_where.push(format!("p.publication_date >= '{}'", yr.replace('\'', "''")));
    }
    if let Some(ref yr) = filters.year_to {
        extra_where.push(format!("p.publication_date <= '{}Z'", yr.replace('\'', "''")));
    }

    // Tag join condition
    let tag_join = if let Some(ref tag) = filters.tag {
        format!(
            "JOIN paper_tags pt ON pt.paper_id = p.id
             JOIN tags tg ON tg.id = pt.tag_id AND tg.name = '{}'",
            tag.replace('\'', "''")
        )
    } else {
        String::new()
    };

    let where_extra = if extra_where.is_empty() {
        String::new()
    } else {
        format!("AND {}", extra_where.join(" AND "))
    };

    // snippet() params: table-name, highlight-column-index (0-based), start-tag, end-tag, ellipsis, max-tokens
    // We use abstract_text (column index 2) as the primary snippet source
    // bm25() weights: title(×3), authors(×2), abstract_text(×1), journal(×1), tldr(×1)
    let sql = format!(
        "SELECT
            p.id, p.title, p.authors, p.abstract_text, p.publication_date, p.journal,
            p.volume, p.issue, p.pages, p.doi, p.arxiv_id, p.pdf_url, p.pdf_path,
            p.folder_id, p.tldr, p.reading_status, p.is_favorite, p.date_added, p.date_modified, p.cover_image,
            bm25(papers_fts, 3.0, 2.0, 1.0, 1.0, 1.0) AS score,
            snippet(papers_fts, 2, '<mark>', '</mark>', '…', 30) AS abstract_snip,
            snippet(papers_fts, 0, '<mark>', '</mark>', '…', 20) AS title_snip
         FROM papers_fts f
         JOIN papers p ON p.rowid = f.rowid
         {tag_join}
         WHERE papers_fts MATCH ?1
         {where_extra}
         ORDER BY score
         LIMIT ?2 OFFSET ?3",
        tag_join = tag_join,
        where_extra = where_extra
    );

    let mut stmt = db.conn().prepare(&sql).map_err(|e| {
        // If FTS query syntax error, return friendly error
        AppError::Internal(format!("搜索语法错误: {}", e))
    })?;

    let papers: Vec<SearchResultItem> = stmt
        .query_map(params![fts_query, limit, offset], |row| {
            let abstract_snip: Option<String> = row.get(21)?;
            let title_snip: Option<String> = row.get(22)?;

            // Build highlight: prefer abstract snippet, fall back to title snippet
            let highlight = abstract_snip.or(title_snip);

            Ok(SearchResultItem {
                paper: Paper {
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
                },
                score: row.get::<_, f64>(20)?,
                highlight,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Count total matches (for pagination)
    let count_sql = format!(
        "SELECT COUNT(*) FROM papers_fts f
         JOIN papers p ON p.rowid = f.rowid
         {tag_join}
         WHERE papers_fts MATCH ?1
         {where_extra}",
        tag_join = tag_join,
        where_extra = where_extra
    );
    let total: u32 = db
        .conn()
        .query_row(&count_sql, params![fts_query], |r| r.get(0))
        .unwrap_or(papers.len() as u32);

    Ok(SearchResult {
        total,
        papers,
        query: raw_query,
        search_type: "fulltext".to_string(),
    })
}

// ─── Search suggestions ─────────────────────────────────────────────────────

/// Real-time search suggestions: prefix-match on title using FTS5 MATCH 'term*'.
/// Returns up to 8 candidate titles with their IDs for instant navigation.
#[tauri::command]
pub fn search_suggest(query: String, db: State<'_, DbState>) -> Result<SuggestResult> {
    let db = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let raw = query.trim();

    if raw.is_empty() {
        return Ok(SuggestResult { items: vec![], query: raw.to_string() });
    }

    // Prefix search on title column only
    let fts_query = format!("title: \"{}\"*", raw.replace('"', ""));

    let mut stmt = db.conn().prepare(
        "SELECT p.id, p.title, p.authors, p.publication_date
         FROM papers_fts f
         JOIN papers p ON p.rowid = f.rowid
         WHERE papers_fts MATCH ?1
         ORDER BY bm25(papers_fts, 5.0, 0.0, 0.0, 0.0, 0.0)
         LIMIT 8"
    )?;

    let items: Vec<SuggestionItem> = stmt
        .query_map(params![fts_query], |row| {
            Ok(SuggestionItem {
                id: row.get(0)?,
                title: row.get(1)?,
                authors: row.get(2)?,
                publication_date: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(SuggestResult { items, query: raw.to_string() })
}
