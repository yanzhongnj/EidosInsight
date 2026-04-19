//! Collection model (user-curated paper groups)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
    /// Paper count (populated via JOIN, not stored in DB)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paper_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCollectionInput {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateCollectionInput {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub sort_order: Option<i32>,
}
