//! AI result model (review, comparison, baseline outputs)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResult {
    pub id: String,
    /// "review" | "comparison" | "baseline"
    pub result_type: String,
    pub collection_id: Option<String>,
    pub title: String,
    /// Markdown output
    pub content: String,
    /// JSON for frontend table rendering (baseline)
    pub structured_data: Option<String>,
    /// e.g. "llama3", "gpt-4o"
    pub model_used: Option<String>,
    /// JSON array of paper IDs
    pub paper_ids: String,
    /// JSON array of comparison dimensions
    pub dimensions: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateReviewInput {
    pub collection_id: String,
    /// Optional focus area for the review
    pub focus: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateComparisonInput {
    pub collection_id: String,
    /// Comparison dimensions: "method" | "dataset" | "experiment" | "metric" | "limitation" | "innovation" | "reproducibility"
    /// If None, uses all preset dimensions
    pub dimensions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateBaselineInput {
    pub collection_id: String,
    /// Specific metric or aspect to compare
    pub metric: Option<String>,
}
