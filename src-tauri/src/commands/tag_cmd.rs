//! Tag command handlers

use crate::db::{DbState, TagRepo};
use crate::errors::Result;
use crate::models::*;
use tauri::State;

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
