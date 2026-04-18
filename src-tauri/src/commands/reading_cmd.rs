//! Reading state command handlers

use crate::db::{DbState, ReadingRepo};
use crate::errors::Result;
use crate::models::*;
use tauri::State;

#[tauri::command]
pub fn get_reading_state(paper_id: String, db: State<'_, DbState>) -> Result<ReadingState> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    ReadingRepo::get_by_paper(db.conn(), &paper_id)
}

#[tauri::command]
pub fn update_reading_state(input: UpdateReadingStateInput, db: State<'_, DbState>) -> Result<ReadingState> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    ReadingRepo::upsert(db.conn(), &input)
}

#[tauri::command]
pub fn list_reading_progress(limit: Option<u32>, db: State<'_, DbState>) -> Result<Vec<ReadingState>> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    ReadingRepo::list_reading_progress(db.conn(), limit.unwrap_or(20))
}
