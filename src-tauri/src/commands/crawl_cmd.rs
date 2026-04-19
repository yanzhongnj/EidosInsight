//! Crawl command handlers — watch directory management and manual scan triggers

use crate::db::{CrawlRepo, DbState, PaperRepo};
use crate::errors::{AppError, Result};
use crate::models::*;
use crate::services::{crawl_service, AppState};
use tauri::State;

// ── Watch directory management ──────────────────────────────────────────────

/// Add a new watch directory
#[tauri::command]
pub fn add_watch_dir(
    input: CreateWatchDirInput,
    db: State<'_, DbState>,
) -> Result<WatchDir> {
    // Validate the directory exists
    let path = std::path::Path::new(&input.path);
    if !path.exists() || !path.is_dir() {
        return Err(AppError::InvalidInput(format!(
            "目录不存在或不是有效目录: {}", input.path
        )));
    }

    let db = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    CrawlRepo::add_watch_dir(db.conn(), &input)
}

/// List all watch directories
#[tauri::command]
pub fn list_watch_dirs(db: State<'_, DbState>) -> Result<Vec<WatchDir>> {
    let db = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    CrawlRepo::list_watch_dirs(db.conn())
}

/// Update a watch directory
#[tauri::command]
pub fn update_watch_dir(
    input: UpdateWatchDirInput,
    db: State<'_, DbState>,
) -> Result<WatchDir> {
    let db = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    CrawlRepo::update_watch_dir(db.conn(), &input)
}

/// Remove a watch directory
#[tauri::command]
pub fn delete_watch_dir(id: String, db: State<'_, DbState>) -> Result<()> {
    let db = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    // Also clean up fingerprints for this watch dir
    CrawlRepo::delete_fingerprints_by_watch_dir(db.conn(), &id)?;
    CrawlRepo::delete_watch_dir(db.conn(), &id)
}

// ── Manual scan triggers ────────────────────────────────────────────────────

/// Manually trigger a scan of a specific watch directory
#[tauri::command]
pub fn scan_watch_dir(
    id: String,
    db: State<'_, DbState>,
    app_state: State<'_, AppState>,
) -> Result<CrawlScanResult> {
    let db = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let watch_dir = CrawlRepo::get_watch_dir(db.conn(), &id)?;
    crawl_service::scan_watch_dir(db.conn(), &watch_dir, &app_state.papers_dir)
}

/// Manually trigger a scan of all enabled watch directories
#[tauri::command]
pub fn scan_all_watch_dirs(
    db: State<'_, DbState>,
    app_state: State<'_, AppState>,
) -> Result<Vec<CrawlScanResult>> {
    let db = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    crawl_service::scan_all_watch_dirs(db.conn(), &app_state.papers_dir)
}

/// Get crawl status — list of watch dirs with their last scan info
#[tauri::command]
pub fn get_crawl_status(db: State<'_, DbState>) -> Result<Vec<WatchDir>> {
    let db = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    CrawlRepo::list_watch_dirs(db.conn())
}

/// Read a non-PDF document file and return its text content
#[tauri::command]
pub fn read_document_file(
    paper_id: String,
    db: State<'_, DbState>,
    app_state: State<'_, AppState>,
) -> Result<String> {
    let db = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let paper = PaperRepo::get_by_id(db.conn(), &paper_id)?;

    let file_name = paper.pdf_path.ok_or_else(|| {
        AppError::InvalidInput(format!("Paper {} has no associated file", paper_id))
    })?;

    let file_path = app_state.papers_dir.join(&file_name);

    if !file_path.exists() {
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Document file not found: {}", file_path.display())
        )));
    }

    let file_type_str = paper.file_type.as_deref().unwrap_or("pdf");

    if file_type_str == "pdf" {
        // For PDFs, return base64-encoded content (existing behavior)
        let bytes = std::fs::read(&file_path).map_err(|e| {
            AppError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to read PDF file: {}", e)
            ))
        })?;
        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        Ok(b64)
    } else {
        // For non-PDF formats, return text content directly
        let file_type = FileType::from_extension(file_type_str)
            .ok_or_else(|| AppError::DocumentParse(format!("Unknown file type: {}", file_type_str)))?;

        let doc_content = crate::services::document_parser::parse_document(&file_path, &file_type)?;

        // Return the text content as a JSON string with metadata
        let result = serde_json::json!({
            "file_type": file_type_str,
            "title": doc_content.title,
            "text_content": doc_content.text_content,
            "page_count": doc_content.page_count,
        });
        Ok(result.to_string())
    }
}
