//! Collection command handlers

use crate::db::{CollectionRepo, DbState};
use crate::errors::Result;
use crate::models::*;
use tauri::State;

#[tauri::command]
pub fn create_collection(input: CreateCollectionInput, db: State<'_, DbState>) -> Result<Collection> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    CollectionRepo::insert(db.conn(), &input)
}

#[tauri::command]
pub fn get_collection(id: String, db: State<'_, DbState>) -> Result<Collection> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    CollectionRepo::get_by_id(db.conn(), &id)
}

#[tauri::command]
pub fn list_collections(db: State<'_, DbState>) -> Result<Vec<Collection>> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    CollectionRepo::list(db.conn())
}

#[tauri::command]
pub fn update_collection(input: UpdateCollectionInput, db: State<'_, DbState>) -> Result<Collection> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    CollectionRepo::update(db.conn(), &input)
}

#[tauri::command]
pub fn delete_collection(id: String, db: State<'_, DbState>) -> Result<()> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    CollectionRepo::delete(db.conn(), &id)
}

#[tauri::command]
pub fn add_paper_to_collection(collection_id: String, paper_id: String, db: State<'_, DbState>) -> Result<()> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    CollectionRepo::add_paper(db.conn(), &collection_id, &paper_id)
}

#[tauri::command]
pub fn remove_paper_from_collection(collection_id: String, paper_id: String, db: State<'_, DbState>) -> Result<()> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    CollectionRepo::remove_paper(db.conn(), &collection_id, &paper_id)
}

#[tauri::command]
pub fn list_collection_papers(collection_id: String, db: State<'_, DbState>) -> Result<Vec<Paper>> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    CollectionRepo::list_papers(db.conn(), &collection_id)
}

#[tauri::command]
pub fn add_papers_to_collection(collection_id: String, paper_ids: Vec<String>, db: State<'_, DbState>) -> Result<u32> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    CollectionRepo::add_papers_batch(db.conn(), &collection_id, &paper_ids)
}
