//! Multi-format document import command handlers

use crate::db::{DbState, PaperRepo};
use crate::errors::{AppError, Result};
use crate::models::*;
use crate::services::{document_parser, AppState};
use std::path::PathBuf;
use tauri_plugin_dialog::DialogExt;
use tauri::State;

/// Import a document file (any supported format) via file dialog (non-blocking)
#[tauri::command]
pub async fn import_document(
    app: tauri::AppHandle,
    db: State<'_, DbState>,
    app_state: State<'_, AppState>,
    folder_id: Option<String>,
) -> Result<Paper> {
    let file_path = tauri::async_runtime::spawn_blocking(move || {
        let dialog = app.dialog();
        let file_paths = dialog
            .file()
            .set_title("选择文档文件")
            .add_filter("文档文件", &["pdf", "txt", "docx", "doc", "xlsx", "xls", "csv", "html", "htm"])
            .add_filter("PDF 文件", &["pdf"])
            .add_filter("文本文件", &["txt"])
            .add_filter("Word 文档", &["docx", "doc"])
            .add_filter("Excel 表格", &["xlsx", "xls"])
            .add_filter("CSV 文件", &["csv"])
            .add_filter("HTML 文件", &["html", "htm"])
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

    do_import_document(file_path, &db, &app_state, folder_id)
}

/// Import a document from a specific file path (drag-and-drop / crawl)
#[tauri::command]
pub async fn import_document_from_path(
    source_path: PathBuf,
    folder_id: Option<String>,
    db: State<'_, DbState>,
    app_state: State<'_, AppState>,
) -> Result<Paper> {
    do_import_document(source_path, &db, &app_state, folder_id)
}

/// Batch import multiple document files via file dialog (non-blocking)
#[tauri::command]
pub async fn batch_import_documents(
    app: tauri::AppHandle,
    folder_id: Option<String>,
    db: State<'_, DbState>,
    app_state: State<'_, AppState>,
) -> Result<Vec<Paper>> {
    let paths = tauri::async_runtime::spawn_blocking(move || {
        let dialog = app.dialog();
        let file_paths = dialog
            .file()
            .set_title("选择文档文件（可多选）")
            .add_filter("文档文件", &["pdf", "txt", "docx", "doc", "xlsx", "xls", "csv", "html", "htm"])
            .add_filter("所有文件", &["*"])
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
        match do_import_document(source_path, &db, &app_state, folder_id.clone()) {
            Ok(paper) => results.push(paper),
            Err(e) => {
                tracing::warn!(error = %e, "Failed to import document");
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

/// Core document import logic — shared by all entry points
pub fn do_import_document(
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

    let file_type = FileType::from_extension(&extension)
        .ok_or_else(|| AppError::InvalidInput(format!(
            "不支持的文件格式: .{}，支持的格式: PDF, TXT, Word(docx), Excel(xlsx), CSV, HTML",
            extension
        )))?;

    // Generate a unique filename for the stored file
    let file_stem = source_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled")
        .to_string();

    let stored_ext = file_type.as_extension();
    let stored_name = format!("{}_{}.{}",
        file_stem.chars().take(50).collect::<String>(),
        chrono::Utc::now().format("%Y%m%d%H%M%S"),
        stored_ext
    );
    let dest_path = app_state.papers_dir.join(&stored_name);

    // Copy file to papers directory
    std::fs::copy(&source_path, &dest_path).map_err(|e| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("复制文件失败: {}", e)
        ))
    })?;

    tracing::info!(src = %source_path.display(), dest = %dest_path.display(), ?file_type, "Document copied to papers dir");

    // Parse the document and extract metadata + text content
    let doc_content = document_parser::parse_document(&dest_path, &file_type).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to parse document, using filename as title");
        document_parser::DocumentContent {
            title: Some(file_stem.clone()),
            authors: None,
            abstract_text: None,
            journal: None,
            publication_date: None,
            pages: None,
            doi: None,
            text_content: None,
            page_count: 0,
        }
    });

    // Use filename as fallback title
    let title = doc_content.title
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| file_stem.clone());

    // Create paper record in database
    let input = CreatePaperInput {
        title,
        authors: doc_content.authors,
        abstract_text: doc_content.abstract_text,
        publication_date: doc_content.publication_date,
        journal: doc_content.journal,
        volume: None,
        issue: None,
        pages: doc_content.pages,
        doi: doc_content.doi,
        arxiv_id: None,
        pdf_url: None,
        pdf_path: Some(stored_name),
        file_type: Some(file_type.as_extension().to_string()),
        folder_id,
    };

    let db = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let paper = PaperRepo::insert(db.conn(), &input)?;

    tracing::info!(
        paper_id = %paper.id,
        title = %paper.title,
        file_type = %file_type,
        "Document imported successfully"
    );

    Ok(paper)
}
