//! Paper model

use serde::{Deserialize, Serialize};

/// Supported document file types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    Pdf,
    Txt,
    Docx,
    Xlsx,
    Csv,
    Html,
}

impl FileType {
    /// Determine file type from extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "pdf" => Some(Self::Pdf),
            "txt" => Some(Self::Txt),
            "docx" | "doc" => Some(Self::Docx),
            "xlsx" | "xls" => Some(Self::Xlsx),
            "csv" => Some(Self::Csv),
            "html" | "htm" => Some(Self::Html),
            _ => None,
        }
    }

    /// Get the file extension for this type
    pub fn as_extension(&self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Txt => "txt",
            Self::Docx => "docx",
            Self::Xlsx => "xlsx",
            Self::Csv => "csv",
            Self::Html => "html",
        }
    }

    /// Get all supported extensions for file dialogs
    pub fn all_extensions() -> Vec<&'static str> {
        vec!["pdf", "txt", "docx", "doc", "xlsx", "xls", "csv", "html", "htm"]
    }
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pdf => write!(f, "PDF"),
            Self::Txt => write!(f, "TXT"),
            Self::Docx => write!(f, "Word"),
            Self::Xlsx => write!(f, "Excel"),
            Self::Csv => write!(f, "CSV"),
            Self::Html => write!(f, "HTML"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paper {
    pub id: String,
    pub title: String,
    pub authors: Option<String>,
    pub abstract_text: Option<String>,
    pub publication_date: Option<String>,
    pub journal: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub doi: Option<String>,
    pub arxiv_id: Option<String>,
    pub pdf_url: Option<String>,
    /// Path to the stored document file (relative to papers_dir).
    /// Legacy field name retained for DB compatibility; renamed conceptually to file_path.
    pub pdf_path: Option<String>,
    /// Document file type (pdf, txt, docx, xlsx, csv, html).
    /// NULL for legacy records defaults to "pdf".
    pub file_type: Option<String>,
    pub folder_id: Option<String>,
    pub tldr: Option<String>,
    pub reading_status: String, // "unread" | "reading" | "read"
    pub is_favorite: bool,
    pub date_added: String,
    pub date_modified: String,
    pub cover_image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaperInput {
    pub title: String,
    pub authors: Option<String>,
    pub abstract_text: Option<String>,
    pub publication_date: Option<String>,
    pub journal: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub doi: Option<String>,
    pub arxiv_id: Option<String>,
    pub pdf_url: Option<String>,
    /// Legacy field — stores the relative file path in papers_dir
    pub pdf_path: Option<String>,
    /// Document file type string (e.g. "pdf", "txt", "docx")
    pub file_type: Option<String>,
    pub folder_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdatePaperInput {
    pub id: String,
    pub title: Option<String>,
    pub authors: Option<String>,
    pub abstract_text: Option<String>,
    pub publication_date: Option<String>,
    pub journal: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub doi: Option<String>,
    pub arxiv_id: Option<String>,
    pub pdf_url: Option<String>,
    pub folder_id: Option<String>,
    pub tldr: Option<String>,
    pub reading_status: Option<String>,
    pub is_favorite: Option<bool>,
    pub cover_image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperFilter {
    pub folder_id: Option<String>,
    pub tag: Option<String>,
    pub reading_status: Option<String>,
    pub is_favorite: Option<bool>,
    pub query: Option<String>,
    pub sort_by: Option<String>,   // "date_added" | "title" | "date_modified"
    pub sort_order: Option<String>, // "asc" | "desc"
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Default for PaperFilter {
    fn default() -> Self {
        Self {
            folder_id: None,
            tag: None,
            reading_status: None,
            is_favorite: None,
            query: None,
            sort_by: Some("date_added".to_string()),
            sort_order: Some("desc".to_string()),
            limit: Some(50),
            offset: Some(0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperListResult {
    pub papers: Vec<Paper>,
    pub total: u32,
}
