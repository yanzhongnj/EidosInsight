//! Note command handlers — CRUD + bidirectional links + full-text search

use crate::db::{DbState, NoteRepo};
use crate::errors::Result;
use crate::models::*;
use tauri::State;

// ── Core CRUD ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn create_note(input: CreateNoteInput, db: State<'_, DbState>) -> Result<Note> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    NoteRepo::insert(db.conn(), &input)
}

#[tauri::command]
pub fn get_note(id: String, db: State<'_, DbState>) -> Result<Note> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    NoteRepo::get_by_id(db.conn(), &id)
}

#[tauri::command]
pub fn list_notes(paper_id: Option<String>, limit: Option<u32>, offset: Option<u32>, db: State<'_, DbState>) -> Result<Vec<Note>> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    match paper_id {
        Some(pid) => NoteRepo::list_by_paper(db.conn(), &pid),
        None => NoteRepo::list(db.conn(), limit.unwrap_or(50), offset.unwrap_or(0)),
    }
}

#[tauri::command]
pub fn update_note(input: UpdateNoteInput, db: State<'_, DbState>) -> Result<Note> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    NoteRepo::update(db.conn(), &input)
}

#[tauri::command]
pub fn delete_note(id: String, db: State<'_, DbState>) -> Result<()> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    NoteRepo::delete(db.conn(), &id)
}

// ── Bidirectional links ─────────────────────────────────────────────────────

#[tauri::command]
pub fn create_note_link(input: CreateNoteLinkInput, db: State<'_, DbState>) -> Result<NoteLink> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    NoteRepo::create_link(db.conn(), &input)
}

#[tauri::command]
pub fn delete_note_link(source_id: String, target_id: String, db: State<'_, DbState>) -> Result<()> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    NoteRepo::delete_link(db.conn(), &source_id, &target_id)
}

#[tauri::command]
pub fn get_note_with_links(id: String, db: State<'_, DbState>) -> Result<NoteWithLinks> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    NoteRepo::get_with_links(db.conn(), &id)
}

#[tauri::command]
pub fn list_outgoing_links(note_id: String, db: State<'_, DbState>) -> Result<Vec<NoteLinkTarget>> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    NoteRepo::list_outgoing_links(db.conn(), &note_id)
}

#[tauri::command]
pub fn list_backlinks(note_id: String, db: State<'_, DbState>) -> Result<Vec<NoteLinkTarget>> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    NoteRepo::list_backlinks(db.conn(), &note_id)
}

// ── Full-text search ────────────────────────────────────────────────────────

#[tauri::command]
pub fn search_notes(
    query: String,
    paper_id: Option<String>,
    note_type: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
    db: State<'_, DbState>,
) -> Result<NoteSearchResult> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    NoteRepo::search(
        db.conn(),
        &query,
        paper_id.as_deref(),
        note_type.as_deref(),
        limit.unwrap_or(20),
        offset.unwrap_or(0),
    )
}

#[tauri::command]
pub fn note_link_suggest(prefix: String, limit: Option<u32>, db: State<'_, DbState>) -> Result<Vec<NoteLinkTarget>> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    NoteRepo::search_suggest(db.conn(), &prefix, limit.unwrap_or(10))
}
