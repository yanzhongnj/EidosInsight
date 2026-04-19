//! Tag command handlers

use crate::db::{DbState, TagRepo};
use crate::errors::Result;
use crate::models::*;
use tauri::State;

#[tauri::command]
pub fn create_tag(name: String, color: Option<String>, db: State<'_, DbState>) -> Result<Tag> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    TagRepo::find_or_create(db.conn(), &name, color.as_deref())
}

#[tauri::command]
pub fn list_tags(db: State<'_, DbState>) -> Result<Vec<Tag>> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    TagRepo::list(db.conn())
}

#[tauri::command]
pub fn rename_tag(id: String, new_name: String, db: State<'_, DbState>) -> Result<()> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    TagRepo::rename(db.conn(), &id, &new_name)
}

#[tauri::command]
pub fn delete_tag(id: String, db: State<'_, DbState>) -> Result<()> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    TagRepo::delete(db.conn(), &id)
}

#[tauri::command]
pub fn update_tag_color(id: String, color: String, db: State<'_, DbState>) -> Result<()> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    TagRepo::update_color(db.conn(), &id, &color)
}

/// Suggest tags for a paper based on its content and existing tags.
/// Uses simple keyword extraction from title + abstract as a heuristic.
/// Full AI-powered suggestions require LLM integration.
#[tauri::command]
pub fn suggest_tags(paper_id: String, db: State<'_, DbState>) -> Result<Vec<String>> {
    let db_lock = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    TagRepo::suggest_tags(db_lock.conn(), &paper_id)
}
