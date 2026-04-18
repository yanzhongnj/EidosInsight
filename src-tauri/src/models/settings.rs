//! Settings model

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,               // "light" | "dark" | "system"
    pub language: String,            // "zh-CN" | "en-US"
    pub auto_process_papers: bool,   // Auto-extract metadata on import
    pub default_llm_provider: String, // "ollama" | "openai" | "claude" | "deepseek"
    pub check_updates_on_start: bool,
    pub paper_storage_path: Option<String>,
    pub max_backup_count: i32,
    pub export_format: String,       // "bibtex" | "apa" | "csv" | "markdown"
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            language: "zh-CN".to_string(),
            auto_process_papers: true,
            default_llm_provider: "ollama".to_string(),
            check_updates_on_start: true,
            paper_storage_path: None,
            max_backup_count: 5,
            export_format: "bibtex".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,       // "ollama" | "openai" | "claude" | "deepseek"
    pub model_name: String,
    pub api_base_url: Option<String>,
    pub api_key_set: bool,      // Don't expose actual key
    pub max_tokens: i32,
    pub temperature: f64,
    pub is_local: bool,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: "ollama".to_string(),
            model_name: "llama3".to_string(),
            api_base_url: Some("http://localhost:11434".to_string()),
            api_key_set: false,
            max_tokens: 4096,
            temperature: 0.7,
            is_local: true,
        }
    }
}
