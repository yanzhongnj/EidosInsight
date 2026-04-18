//! Folder command handlers

use crate::db::{DbState, FolderRepo};
use crate::errors::Result;
use crate::models::*;
use tauri::State;

#[tauri::command]
pub fn create_folder(input: CreateFolderInput, db: State<'_, DbState>) -> Result<Folder> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    FolderRepo::insert(db.conn(), &input)
}

#[tauri::command]
pub fn list_folders(db: State<'_, DbState>) -> Result<Vec<Folder>> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    FolderRepo::list(db.conn())
}

#[tauri::command]
pub fn update_folder(input: UpdateFolderInput, db: State<'_, DbState>) -> Result<Folder> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    FolderRepo::update(db.conn(), &input)
}

#[tauri::command]
pub fn delete_folder(id: String, db: State<'_, DbState>) -> Result<()> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    FolderRepo::delete(db.conn(), &id)
}
