//! Folder model

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub date_created: String,
    pub date_modified: String,
    /// Number of papers in this folder (computed via LEFT JOIN)
    #[serde(default)]
    pub paper_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFolderInput {
    pub name: String,
    pub parent_id: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateFolderInput {
    pub id: String,
    pub name: Option<String>,
    pub parent_id: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub sort_order: Option<i32>,
}
