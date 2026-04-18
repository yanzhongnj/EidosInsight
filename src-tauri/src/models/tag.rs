//! Tag model

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub usage_count: i32,
    pub date_created: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperTag {
    pub paper_id: String,
    pub tag_id: String,
    pub date_added: String,
}
