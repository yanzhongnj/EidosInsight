//! Incremental crawl service — monitors watch directories for new/modified documents
//!
//! Periodically scans enabled watch directories, compares file fingerprints
//! (mtime + size + optional SHA-256 hash), and automatically imports new or
//! changed documents into the knowledge base.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use sha2::{Sha256, Digest};

use crate::db::{CrawlRepo, DatabaseManager, PaperRepo};
use crate::errors::{AppError, Result};
use crate::models::*;
use crate::services::document_parser;

/// Compute SHA-256 hash of a file's contents
fn compute_file_hash(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hash = hasher.finalize();
    Some(format!("{:x}", hash))
}

/// Get file metadata (size + modified time) for fingerprinting
fn get_file_meta(path: &Path) -> Option<(i64, i64)> {
    let metadata = std::fs::metadata(path).ok()?;
    let size = metadata.len() as i64;
    let modified = metadata.modified().ok()?
        .duration_since(std::time::UNIX_EPOCH).ok()?
        .as_secs() as i64;
    Some((size, modified))
}

/// Check if a file extension matches the watch dir's filter
fn matches_filter(extension: &str, file_filter: &Option<String>) -> bool {
    match file_filter {
        None => {
            // No filter — accept all supported formats
            FileType::from_extension(extension).is_some()
        }
        Some(filter) => {
            let extensions: Vec<&str> = filter.split(',')
                .map(|s| s.trim().trim_start_matches('.'))
                .collect();
            extensions.contains(&extension.to_lowercase().as_str())
        }
    }
}

/// Recursively collect all files in a directory that match the filter
fn collect_files(dir: &Path, file_filter: &Option<String>, recursive: bool) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() && recursive {
                files.extend(collect_files(&path, file_filter, recursive));
            } else if path.is_file() {
                let ext = path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_string();
                if matches_filter(&ext, file_filter) {
                    files.push(path);
                }
            }
        }
    }

    files
}

/// Import a single file discovered during crawl (internal, used by scan_watch_dir)
fn import_crawled_file(
    conn: &Connection,
    source_path: &Path,
    watch_dir: &WatchDir,
    papers_dir: &Path,
) -> Result<Paper> {
    let extension = source_path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let file_type = FileType::from_extension(&extension)
        .ok_or_else(|| AppError::DocumentParse(format!(
            "Unsupported file type: .{}", extension
        )))?;

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
    let dest_path = papers_dir.join(&stored_name);

    // Copy file to papers directory
    std::fs::copy(source_path, &dest_path).map_err(|e| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to copy file: {}", e)
        ))
    })?;

    // Parse document
    let doc_content = document_parser::parse_document(&dest_path, &file_type).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to parse crawled document, using filename");
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

    let title = doc_content.title
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| file_stem.clone());

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
        folder_id: watch_dir.folder_id.clone(),
    };

    PaperRepo::insert(conn, &input)
}

/// Scan a single watch directory and import new/modified files
pub fn scan_watch_dir(
    conn: &Connection,
    watch_dir: &WatchDir,
    papers_dir: &Path,
) -> Result<CrawlScanResult> {
    let dir_path = Path::new(&watch_dir.path);
    if !dir_path.exists() || !dir_path.is_dir() {
        return Err(AppError::InvalidInput(format!(
            "Watch directory does not exist or is not a directory: {}", watch_dir.path
        )));
    }

    let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
    let files = collect_files(dir_path, &watch_dir.file_filter, watch_dir.recursive);

    let mut result = CrawlScanResult {
        new_files: 0,
        modified_files: 0,
        imported: 0,
        failed: 0,
        total_scanned: files.len() as u32,
        errors: Vec::new(),
    };

    for file_path in &files {
        let path_str = file_path.to_string_lossy().to_string();

        // Get current file metadata
        let (size, mtime) = match get_file_meta(file_path) {
            Some(meta) => meta,
            None => {
                tracing::warn!(path = %path_str, "Cannot read file metadata, skipping");
                continue;
            }
        };

        // Check existing fingerprint
        let existing = CrawlRepo::get_fingerprint(conn, &path_str).unwrap_or(None);

        match existing {
            None => {
                // New file — import it
                result.new_files += 1;
            }
            Some(fp) => {
                // Check if file has been modified
                if fp.file_size == size && fp.modified_time == mtime {
                    // Unchanged — skip
                    continue;
                }
                // Modified file
                if fp.is_imported {
                    result.modified_files += 1;
                } else {
                    result.new_files += 1;
                }
            }
        }

        // Import the file
        match import_crawled_file(conn, file_path, watch_dir, papers_dir) {
            Ok(paper) => {
                result.imported += 1;

                // Update fingerprint
                let content_hash = compute_file_hash(file_path);
                let fp = FileFingerprint {
                    id: uuid::Uuid::new_v4().to_string(),
                    file_path: path_str.clone(),
                    watch_dir_id: watch_dir.id.clone(),
                    file_size: size,
                    modified_time: mtime,
                    content_hash,
                    is_imported: true,
                    paper_id: Some(paper.id),
                    discovered_at: now.clone(),
                    last_checked: now.clone(),
                };
                if let Err(e) = CrawlRepo::upsert_fingerprint(conn, &fp) {
                    tracing::warn!(error = %e, path = %path_str, "Failed to update fingerprint");
                }
            }
            Err(e) => {
                result.failed += 1;
                result.errors.push(format!("{}: {}", path_str, e));
                tracing::warn!(error = %e, path = %path_str, "Failed to import crawled file");

                // Still record the fingerprint so we don't retry immediately
                let fp = FileFingerprint {
                    id: uuid::Uuid::new_v4().to_string(),
                    file_path: path_str.clone(),
                    watch_dir_id: watch_dir.id.clone(),
                    file_size: size,
                    modified_time: mtime,
                    content_hash: None,
                    is_imported: false,
                    paper_id: None,
                    discovered_at: now.clone(),
                    last_checked: now.clone(),
                };
                let _ = CrawlRepo::upsert_fingerprint(conn, &fp);
            }
        }
    }

    // Update last_scan timestamp
    let _ = CrawlRepo::update_last_scan(conn, &watch_dir.id);

    tracing::info!(
        watch_dir = %watch_dir.path,
        new = result.new_files,
        modified = result.modified_files,
        imported = result.imported,
        failed = result.failed,
        "Crawl scan completed"
    );

    Ok(result)
}

/// Scan all enabled watch directories
pub fn scan_all_watch_dirs(
    conn: &Connection,
    papers_dir: &Path,
) -> Result<Vec<CrawlScanResult>> {
    let watch_dirs = CrawlRepo::list_enabled_watch_dirs(conn)?;
    let mut results = Vec::new();

    for watch_dir in &watch_dirs {
        match scan_watch_dir(conn, watch_dir, papers_dir) {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!(
                    watch_dir = %watch_dir.path,
                    error = %e,
                    "Failed to scan watch directory"
                );
                results.push(CrawlScanResult {
                    new_files: 0,
                    modified_files: 0,
                    imported: 0,
                    failed: 0,
                    total_scanned: 0,
                    errors: vec![e.to_string()],
                });
            }
        }
    }

    Ok(results)
}

/// Perform a single crawl tick — called by the scheduler
/// This is the main entry point for periodic crawl execution.
pub fn crawl_tick(db_arc: Arc<Mutex<DatabaseManager>>, papers_dir: PathBuf) {
    let db_manager = match db_arc.lock() {
        Ok(m) => m,
        Err(e) => {
            tracing::error!(error = %e, "Failed to acquire DB lock for crawl tick");
            return;
        }
    };

    let conn = db_manager.conn();

    match scan_all_watch_dirs(conn, &papers_dir) {
        Ok(results) => {
            let total_imported: u32 = results.iter().map(|r| r.imported).sum();
            if total_imported > 0 {
                tracing::info!(total_imported, "Crawl tick: new documents imported");
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "Crawl tick failed");
        }
    }
}
