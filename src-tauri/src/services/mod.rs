//! Services module

use std::path::PathBuf;
use std::sync::Arc;

use crate::services::llm::LlmRouter;

/// Application state shared across Tauri commands
pub struct AppState {
    pub app_data_dir: PathBuf,
    pub papers_dir: PathBuf,
    pub exports_dir: PathBuf,
    pub backups_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub skills_dir: PathBuf,
}

/// LLM state — wrapped in Arc for async-safe shared access
pub struct LlmState(pub Arc<LlmRouter>);

pub mod pdf_service;
pub mod document_parser;
pub mod crawl_service;
pub mod llm;
pub mod scheduler;
