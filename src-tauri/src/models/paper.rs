//! Paper model

use serde::{Deserialize, Serialize};

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
    pub pdf_path: Option<String>,
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
    pub pdf_path: Option<String>,
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
