//! Search result model

use serde::{Deserialize, Serialize};

use super::Paper;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub papers: Vec<SearchResultItem>,
    pub total: u32,
    pub query: String,
    pub search_type: String, // "keyword" | "fulltext"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub paper: Paper,
    pub score: f64,
    pub highlight: Option<String>, // Highlighted matching snippet
}
