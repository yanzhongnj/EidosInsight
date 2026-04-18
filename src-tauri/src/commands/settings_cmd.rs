//! Settings command handlers

use crate::db::{DbState, SettingsRepo};
use crate::errors::Result;
use crate::models::*;
use tauri::State;

#[tauri::command]
pub fn get_settings(db: State<'_, DbState>) -> Result<AppSettings> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    SettingsRepo::get(db.conn())
}

#[tauri::command]
pub fn update_settings(settings: AppSettings, db: State<'_, DbState>) -> Result<()> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    SettingsRepo::update_settings(db.conn(), &settings)
}

#[tauri::command]
pub fn get_llm_config(db: State<'_, DbState>) -> Result<LlmConfig> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    let mut config = LlmConfig::default();

    if let Some(provider) = SettingsRepo::get_value(db.conn(), "llm_provider")? {
        config.provider = provider;
    }
    if let Some(model) = SettingsRepo::get_value(db.conn(), "llm_model_name")? {
        config.model_name = model;
    }
    if let Some(url) = SettingsRepo::get_value(db.conn(), "llm_api_base_url")? {
        config.api_base_url = Some(url);
    }
    if let Some(tokens) = SettingsRepo::get_value(db.conn(), "llm_max_tokens")? {
        config.max_tokens = tokens.parse().unwrap_or(4096);
    }
    if let Some(temp) = SettingsRepo::get_value(db.conn(), "llm_temperature")? {
        config.temperature = temp.parse().unwrap_or(0.7);
    }

    config.is_local = config.provider == "ollama";
    config.api_key_set = check_api_key_set(&config.provider);

    Ok(config)
}

#[tauri::command]
pub fn update_llm_config(config: LlmConfig, db: State<'_, DbState>) -> Result<()> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    SettingsRepo::set(db.conn(), "llm_provider", &config.provider)?;
    SettingsRepo::set(db.conn(), "llm_model_name", &config.model_name)?;
    if let Some(ref url) = config.api_base_url {
        SettingsRepo::set(db.conn(), "llm_api_base_url", url)?;
    }
    SettingsRepo::set(db.conn(), "llm_max_tokens", &config.max_tokens.to_string())?;
    SettingsRepo::set(db.conn(), "llm_temperature", &config.temperature.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn test_llm_connection(config: LlmConfig) -> Result<bool> {
    // Simple connection test: try to reach the LLM endpoint
    let (url, needs_auth) = match config.provider.as_str() {
        "ollama" => {
            let base = config.api_base_url.as_deref().unwrap_or("http://localhost:11434");
            (format!("{}/api/tags", base), false)
        }
        "openai" => ("https://api.openai.com/v1/models".to_string(), true),
        "claude" => ("https://api.anthropic.com/v1/models".to_string(), true),
        "deepseek" => ("https://api.deepseek.com/v1/models".to_string(), true),
        _ => return Err(crate::errors::AppError::Llm(format!("Unknown provider: {}", config.provider))),
    };

    let client = reqwest::Client::new();
    let mut request = client.get(&url).timeout(std::time::Duration::from_secs(10));

    // For cloud providers, attach API key from keyring
    if needs_auth {
        let service = format!("com.eidosinsight.{}", config.provider);
        match keyring::Entry::new(&service, "api_key") {
            Ok(entry) => match entry.get_password() {
                Ok(api_key) => {
                    if api_key.is_empty() {
                        tracing::warn!(provider = %config.provider, "API key in keyring is empty — was it saved with a wrong parameter name?");
                    } else {
                        tracing::info!(provider = %config.provider, key_len = api_key.len(), "API key found in keyring, attaching to request");
                        request = request.header("Authorization", format!("Bearer {}", api_key));
                    }
                }
                Err(e) => {
                    tracing::warn!(provider = %config.provider, error = %e, "API key not found in keyring — connection test will likely fail");
                }
            },
            Err(e) => {
                tracing::warn!(provider = %config.provider, error = %e, "Failed to access keyring — connection test will likely fail");
            }
        }
    }

    let response = request.send().await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                tracing::info!(provider = %config.provider, status = %status, "LLM connection test succeeded");
                Ok(true)
            } else {
                let body = resp.text().await.unwrap_or_default();
                tracing::warn!(provider = %config.provider, status = %status, body = %body, "LLM connection test failed — non-success status");
                Ok(false)
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "LLM connection test failed — network error");
            Ok(false)
        }
    }
}

fn check_api_key_set(provider: &str) -> bool {
    if provider == "ollama" {
        return true; // Ollama doesn't need an API key
    }
    // Try to check keyring for API key
    let service = format!("com.eidosinsight.{}", provider);
    match keyring::Entry::new(&service, "api_key") {
        Ok(entry) => entry.get_password().is_ok(),
        Err(_) => false,
    }
}
