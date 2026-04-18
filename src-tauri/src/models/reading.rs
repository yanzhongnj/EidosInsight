//! Reading state model

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingState {
    pub id: String,
    pub paper_id: String,
    pub current_page: i32,
    pub total_pages: i32,
    pub scroll_position: f64,
    pub zoom_level: f64,
    pub last_read_at: String,
    pub reading_progress: f64, // 0.0 ~ 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReadingStateInput {
    pub paper_id: String,
    pub current_page: Option<i32>,
    pub total_pages: Option<i32>,
    pub scroll_position: Option<f64>,
    pub zoom_level: Option<f64>,
}
