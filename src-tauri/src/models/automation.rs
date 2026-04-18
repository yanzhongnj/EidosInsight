//! Automation and Skill data models

use serde::{Deserialize, Serialize};

// ── AutoTask ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoTask {
    pub id: String,
    pub name: String,
    /// "preset" | "custom"
    pub task_type: String,
    /// Preset template ID (preset tasks only)
    pub preset_id: Option<String>,
    /// JSON: task step definitions
    pub definition: String,
    /// JSON: human-readable schedule config
    pub schedule: String,
    /// Internal cron expression (user never sees this)
    pub schedule_cron: Option<String>,
    pub is_enabled: bool,
    /// Last execution timestamp
    pub last_run: Option<String>,
    /// Next scheduled execution timestamp
    pub next_run: Option<String>,
    /// JSON: user configuration (keywords, etc.)
    pub config: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAutoTask {
    pub name: String,
    pub task_type: String,
    pub preset_id: Option<String>,
    pub definition: String,
    pub schedule: String,
    pub schedule_cron: Option<String>,
    pub config: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateAutoTask {
    pub name: Option<String>,
    pub definition: Option<String>,
    pub schedule: Option<String>,
    pub schedule_cron: Option<String>,
    pub is_enabled: Option<bool>,
    pub config: Option<String>,
}

// ── TaskExecution ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecution {
    pub id: String,
    pub task_id: String,
    /// "running" | "success" | "failed"
    pub status: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    /// JSON: execution result
    pub result: Option<String>,
    pub error: Option<String>,
    pub duration_ms: Option<i64>,
}

// ── Skill ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    /// Markdown + YAML frontmatter definition
    pub definition: String,
    /// "builtin" | "user_created" | "ai_generated" | "community"
    pub source: String,
    pub version: String,
    /// Trigger keyword (e.g. /trending-weekly)
    pub trigger: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSkill {
    pub name: String,
    pub definition: String,
    pub source: String,
    pub trigger: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateSkill {
    pub name: Option<String>,
    pub definition: Option<String>,
    pub trigger: Option<String>,
    pub is_active: Option<bool>,
}

// ── Schedule Config (human-readable) ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    /// "daily" | "weekly" | "custom"
    pub frequency: String,
    /// "morning_8" | "noon_12" | "evening_8" | "custom"
    pub time: String,
    /// For weekly: selected weekdays (0=Sun..6=Sat)
    pub weekdays: Option<Vec<u8>>,
    /// For custom frequency: interval in hours
    pub custom_interval_hours: Option<u32>,
    /// For custom time: hour (0-23)
    pub custom_hour: Option<u8>,
    /// For custom time: minute (0-59)
    pub custom_minute: Option<u8>,
}

// ── Preset Template ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetTemplate {
    pub preset_id: String,
    pub name: String,
    pub description: String,
    pub definition: String,
    pub default_cron: String,
    /// Fields the user needs to configure
    pub config_fields: Vec<PresetConfigField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetConfigField {
    pub key: String,
    pub label: String,
    pub field_type: String, // "text" | "select" | "multiselect"
    pub required: bool,
    pub default_value: Option<String>,
    pub options: Option<Vec<String>>,
}

// ── Skill Execution Result ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillResult {
    pub success: bool,
    pub output: String,
    pub duration_ms: u64,
    pub error: Option<String>,
}
