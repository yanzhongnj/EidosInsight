//! Paper command handlers

use crate::db::{DbState, PaperRepo};
use crate::errors::{AppError, Result};
use crate::models::*;
use crate::services::{pdf_service, AppState};
use std::path::PathBuf;
use tauri_plugin_dialog::DialogExt;
use tauri::State;

/// Import a paper manually (user provides metadata)
#[tauri::command]
pub fn import_paper(input: CreatePaperInput, db: State<'_, DbState>) -> Result<Paper> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    let paper = PaperRepo::insert(db.conn(), &input)?;
    tracing::info!(paper_id = %paper.id, title = %paper.title, "Paper imported");
    Ok(paper)
}

/// Import a PDF file - opens file dialog, copies PDF to papers dir,
/// extracts metadata, and creates a paper record.
/// NOTE: Kept for backward compatibility. Prefer import_document for multi-format support.
#[tauri::command]
pub async fn import_pdf(
    app: tauri::AppHandle,
    folder_id: Option<String>,
    db: State<'_, DbState>,
    app_state: State<'_, AppState>,
) -> Result<Paper> {
    // Open file dialog for PDF selection using the Dialog plugin (non-blocking)
    let file_path = tauri::async_runtime::spawn_blocking(move || {
        let dialog = app.dialog();
        let file_paths = dialog
            .file()
            .set_title("选择 PDF 文件")
            .add_filter("PDF 文件", &["pdf"])
            .add_filter("所有文件", &["*"])
            .blocking_pick_files();

        file_paths
            .and_then(|paths| {
                if paths.is_empty() { None } else { Some(paths[0].clone()) }
            })
            .and_then(|p| p.as_path().map(|pp| pp.to_path_buf()))
    })
    .await
    .map_err(|e| AppError::Internal(format!("Dialog error: {}", e)))?
    .ok_or_else(|| AppError::InvalidInput("未选择文件".to_string()))?;

    do_import_pdf(file_path, &db, &app_state, folder_id)
}

/// Import a PDF from a specific file path (used by drag-and-drop)
#[tauri::command]
pub async fn import_pdf_from_path(
    source_path: PathBuf,
    folder_id: Option<String>,
    db: State<'_, DbState>,
    app_state: State<'_, AppState>,
) -> Result<Paper> {
    do_import_pdf(source_path, &db, &app_state, folder_id)
}

/// Core PDF import logic shared by both entry points
fn do_import_pdf(
    source_path: PathBuf,
    db: &State<'_, DbState>,
    app_state: &State<'_, AppState>,
    folder_id: Option<String>,
) -> Result<Paper> {
    // Validate the file exists
    if !source_path.exists() {
        return Err(AppError::InvalidInput(format!(
            "文件不存在: {}", source_path.display()
        )));
    }

    let extension = source_path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if extension != "pdf" {
        return Err(AppError::InvalidInput(format!(
            "不支持的文件格式: .{}，仅支持 PDF 文件", extension
        )));
    }

    // Generate a unique filename for the stored PDF
    let file_stem = source_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled")
        .to_string();

    let stored_name = format!("{}_{}.pdf",
        file_stem.chars().take(50).collect::<String>(),
        chrono::Utc::now().format("%Y%m%d%H%M%S")
    );
    let dest_path = app_state.papers_dir.join(&stored_name);

    // Copy PDF to papers directory
    std::fs::copy(&source_path, &dest_path).map_err(|e| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("复制 PDF 文件失败: {}", e)
        ))
    })?;

    tracing::info!(src = %source_path.display(), dest = %dest_path.display(), "PDF copied to papers dir");

    // Extract metadata from PDF
    let metadata = pdf_service::extract_metadata(&dest_path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to extract PDF metadata, using defaults");
        pdf_service::PdfMetadata {
            title: Some(file_stem.clone()),
            authors: None,
            abstract_text: None,
            journal: None,
            volume: None,
            issue: None,
            pages: None,
            doi: None,
            publication_date: None,
            page_count: 0,
        }
    });

    // Use filename as fallback title
    let title = metadata.title
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| file_stem.clone());

    // Create paper record in database
    let input = CreatePaperInput {
        title,
        authors: metadata.authors,
        abstract_text: metadata.abstract_text,
        publication_date: metadata.publication_date,
        journal: metadata.journal,
        volume: metadata.volume,
        issue: metadata.issue,
        pages: metadata.pages,
        doi: metadata.doi,
        arxiv_id: None,
        pdf_url: None,
        pdf_path: Some(stored_name),
        file_type: Some("pdf".to_string()),
        folder_id,
    };

    let db = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let paper = PaperRepo::insert(db.conn(), &input)?;

    tracing::info!(
        paper_id = %paper.id,
        title = %paper.title,
        pages = metadata.page_count,
        "PDF imported successfully"
    );

    Ok(paper)
}

/// Batch import multiple PDF files (non-blocking)
#[tauri::command]
pub async fn batch_import_pdfs(
    app: tauri::AppHandle,
    folder_id: Option<String>,
    db: State<'_, DbState>,
    app_state: State<'_, AppState>,
) -> Result<Vec<Paper>> {
    let paths = tauri::async_runtime::spawn_blocking(move || {
        let dialog = app.dialog();
        let file_paths = dialog
            .file()
            .set_title("选择 PDF 文件（可多选）")
            .add_filter("PDF 文件", &["pdf"])
            .blocking_pick_files();

        file_paths
            .map(|paths| {
                paths.into_iter()
                    .filter_map(|p| p.as_path().map(|pp| pp.to_path_buf()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    })
    .await
    .map_err(|e| AppError::Internal(format!("Dialog error: {}", e)))?;

    let mut results = Vec::new();
    let mut errors = Vec::new();

    for source_path in paths {
        match do_import_pdf(source_path, &db, &app_state, folder_id.clone()) {
            Ok(paper) => results.push(paper),
            Err(e) => {
                tracing::warn!(error = %e, "Failed to import PDF");
                errors.push(e.to_string());
            }
        }
    }

    if results.is_empty() && !errors.is_empty() {
        return Err(AppError::InvalidInput(format!(
            "所有文件导入失败: {}", errors.join("; ")
        )));
    }

    if !errors.is_empty() {
        tracing::warn!(
            imported = results.len(),
            failed = errors.len(),
            "Batch import completed with some failures"
        );
    }

    Ok(results)
}

#[tauri::command]
pub fn get_paper(id: String, db: State<'_, DbState>) -> Result<Paper> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    PaperRepo::get_by_id(db.conn(), &id)
}

#[tauri::command]
pub fn list_papers(filter: Option<PaperFilter>, db: State<'_, DbState>) -> Result<PaperListResult> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    let filter = filter.unwrap_or_default();
    PaperRepo::list(db.conn(), &filter)
}

#[tauri::command]
pub fn update_paper(input: UpdatePaperInput, db: State<'_, DbState>) -> Result<Paper> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    PaperRepo::update(db.conn(), &input)
}

#[tauri::command]
pub fn delete_paper(id: String, db: State<'_, DbState>, app_state: State<'_, AppState>) -> Result<()> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;

    // Get paper to find its PDF file
    let paper = PaperRepo::get_by_id(db.conn(), &id)?;

    // Delete the PDF file if it exists
    if let Some(ref pdf_filename) = paper.pdf_path {
        let pdf_file = app_state.papers_dir.join(pdf_filename);
        if pdf_file.exists() {
            std::fs::remove_file(&pdf_file).ok();
            tracing::info!(path = %pdf_file.display(), "PDF file deleted");
        }
    }

    PaperRepo::delete(db.conn(), &id)?;
    tracing::info!(paper_id = %id, "Paper deleted");
    Ok(())
}

#[tauri::command]
pub fn move_paper_to_folder(paper_id: String, folder_id: Option<String>, db: State<'_, DbState>) -> Result<Paper> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    // Directly set folder_id (including NULL when folder_id is None) —
    // PaperRepo::update skips None fields, so we handle this specially.
    let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
    if let Some(ref fid) = folder_id {
        db.conn().execute(
            "UPDATE papers SET folder_id = ?1, date_modified = ?2 WHERE id = ?3",
            rusqlite::params![fid, now, paper_id],
        )?;
    } else {
        db.conn().execute(
            "UPDATE papers SET folder_id = NULL, date_modified = ?1 WHERE id = ?2",
            rusqlite::params![now, paper_id],
        )?;
    }
    PaperRepo::get_by_id(db.conn(), &paper_id)
}

/// Get tags for a specific paper
#[tauri::command]
pub fn get_tags_for_paper(paper_id: String, db: State<'_, DbState>) -> Result<Vec<Tag>> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    crate::db::TagRepo::get_tags_for_paper(db.conn(), &paper_id)
}

/// Batch get tags for multiple papers — returns a map of paper_id → Vec<Tag>
#[tauri::command]
pub fn batch_get_paper_tags(paper_ids: Vec<String>, db: State<'_, DbState>) -> Result<std::collections::HashMap<String, Vec<Tag>>> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    let mut result = std::collections::HashMap::new();
    for paper_id in paper_ids {
        let tags = crate::db::TagRepo::get_tags_for_paper(db.conn(), &paper_id).unwrap_or_default();
        result.insert(paper_id, tags);
    }
    Ok(result)
}

#[tauri::command]
pub fn add_paper_tag(paper_id: String, tag_name: String, db: State<'_, DbState>) -> Result<()> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    let tag = crate::db::TagRepo::find_or_create(db.conn(), &tag_name, None)?;
    crate::db::TagRepo::add_paper_tag(db.conn(), &paper_id, &tag.id)
}

#[tauri::command]
pub fn remove_paper_tag(paper_id: String, tag_id: String, db: State<'_, DbState>) -> Result<()> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    crate::db::TagRepo::remove_paper_tag(db.conn(), &paper_id, &tag_id)
}

/// Read a PDF file and return its content as Base64 for frontend rendering
#[tauri::command]
pub fn read_pdf_file(paper_id: String, db: State<'_, DbState>, app_state: State<'_, AppState>) -> Result<String> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;

    // Get paper record to find the PDF path
    let paper = PaperRepo::get_by_id(db.conn(), &paper_id)?;

    let pdf_filename = paper.pdf_path.ok_or_else(|| {
        AppError::InvalidInput(format!("Paper {} has no associated PDF file", paper_id))
    })?;

    let pdf_path = app_state.papers_dir.join(&pdf_filename);

    if !pdf_path.exists() {
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("PDF file not found: {}", pdf_path.display())
        )));
    }

    let bytes = std::fs::read(&pdf_path).map_err(|e| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to read PDF file: {}", e)
        ))
    })?;

    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(b64)
}
