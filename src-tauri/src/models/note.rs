//! Note model

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub paper_id: Option<String>,
    pub title: String,
    pub content: String,          // Markdown content
    pub note_type: String,        // "general" | "annotation" | "excerpt" | "summary"
    pub page_number: Option<i32>, // PDF page number for annotations
    pub position: Option<String>, // JSON position data for annotations
    pub date_created: String,
    pub date_modified: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNoteInput {
    pub paper_id: Option<String>,
    pub title: String,
    pub content: String,
    pub note_type: Option<String>,
    pub page_number: Option<i32>,
    pub position: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateNoteInput {
    pub id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub note_type: Option<String>,
    pub page_number: Option<i32>,
    pub position: Option<String>,
}

// ── Bidirectional link types ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteLink {
    pub source_id: String,
    pub target_id: String,
    pub label: Option<String>,
    pub date_created: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNoteLinkInput {
    pub source_id: String,
    pub target_id: String,
    pub label: Option<String>,
}

// ── Note search result ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteSearchResultItem {
    pub note: Note,
    pub score: f64,
    /// HTML snippet with <mark> tags wrapping matched terms
    pub highlight: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteSearchResult {
    pub items: Vec<NoteSearchResultItem>,
    pub total: u32,
    pub query: String,
}

// ── Note with outgoing links (for detail views) ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteWithLinks {
    pub note: Note,
    /// Notes this note links TO
    pub outgoing_links: Vec<NoteLinkTarget>,
    /// Notes that link TO this note
    pub backlinks: Vec<NoteLinkTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteLinkTarget {
    pub note_id: String,
    pub title: String,
    pub label: Option<String>,
}
