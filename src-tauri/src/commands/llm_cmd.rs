//! LLM command handlers

use crate::db::{DbState, SettingsRepo};
use crate::errors::{AppError, Result};
use crate::models::LlmConfig;
use crate::services::llm::*;
use crate::services::LlmState;
use futures::StreamExt;
use tauri::{AppHandle, Emitter, State};

/// Get all available provider presets
#[tauri::command]
pub fn get_llm_provider_presets() -> Vec<ProviderPreset> {
    get_all_presets()
}

/// List currently configured providers (which adapters are active)
#[tauri::command]
pub async fn list_configured_providers(llm: State<'_, LlmState>) -> Result<Vec<ConfiguredProvider>> {
    let router = &llm.0;
    let mut providers = Vec::new();

    // Always report Ollama
    let ollama_available = router.is_ollama_available().await;
    providers.push(ConfiguredProvider {
        id: "ollama".to_string(),
        name: "Ollama (本地)".to_string(),
        is_available: ollama_available,
        is_local: true,
    });

    // Check Anthropic
    let anthropic = router.get_anthropic_adapter().await;
    if let Some(adapter) = anthropic.as_ref() {
        providers.push(ConfiguredProvider {
            id: "anthropic".to_string(),
            name: "Anthropic Claude".to_string(),
            is_available: adapter.health_check().await,
            is_local: false,
        });
    }
    drop(anthropic);

    // Check OpenAI-compat
    let openai = router.get_openai_compat_adapter().await;
    if let Some(adapter) = openai.as_ref() {
        let preset = find_preset(adapter.provider_name());
        providers.push(ConfiguredProvider {
            id: adapter.provider_name().to_string(),
            name: preset.map(|p| p.name.clone()).unwrap_or_else(|| adapter.provider_name().to_string()),
            is_available: adapter.health_check().await,
            is_local: false,
        });
    }

    Ok(providers)
}

/// Switch the active provider (rebuilds the cloud adapter)
#[tauri::command]
pub async fn switch_active_provider(
    provider: String,
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
) -> Result<()> {
    // Save provider to DB
    {
        let db_conn = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        SettingsRepo::set(db_conn.conn(), "llm_provider", &provider)?;
    }

    // Rebuild cloud adapter
    rebuild_cloud_adapter(&provider, &llm).await?;

    Ok(())
}

/// Detect local Ollama availability and list models
#[tauri::command]
pub async fn detect_ollama(llm: State<'_, LlmState>) -> Result<OllamaDetectionResult> {
    let router = &llm.0;
    let available = router.is_ollama_available().await;

    let models = if available {
        router.list_ollama_models().await.unwrap_or_default()
    } else {
        vec![]
    };

    Ok(OllamaDetectionResult { available, models })
}

/// Get the current LLM status
#[tauri::command]
pub async fn get_llm_status(llm: State<'_, LlmState>) -> Result<LlmStatusResult> {
    let router = &llm.0;
    let status = router.get_status().await;

    let status_str = match status {
        LlmStatus::LocalAvailable => "local_available",
        LlmStatus::CloudAvailable => "cloud_available",
        LlmStatus::Unavailable => "unavailable",
    };

    let has_cloud = router.has_cloud_adapter().await;
    let active_cloud = router.active_cloud_provider().await;

    Ok(LlmStatusResult {
        status: status_str.to_string(),
        has_cloud,
        active_cloud_provider: active_cloud,
    })
}

/// List models from the current provider
#[tauri::command]
pub async fn list_llm_models(llm: State<'_, LlmState>, db: State<'_, DbState>) -> Result<Vec<ModelInfo>> {
    let router = &llm.0;

    // Determine which provider the user configured
    let provider = {
        let db_conn = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        SettingsRepo::get_value(db_conn.conn(), "llm_provider").unwrap_or(Some("ollama".to_string()))
    };

    match provider.as_deref() {
        Some("ollama") => router.list_ollama_models().await,
        _ => router.list_models().await,
    }
}

/// Non-streaming LLM completion
#[tauri::command]
pub async fn llm_complete(
    messages: Vec<MessageInput>,
    options: Option<CompletionOptions>,
    llm: State<'_, LlmState>,
) -> Result<String> {
    let router = &llm.0;
    let msgs: Vec<Message> = messages.into_iter().map(|m| m.into()).collect();
    let opts = options.unwrap_or_default();
    router.complete(msgs, opts).await
}

/// Streaming LLM completion — pushes chunks to the frontend via Tauri Event
#[tauri::command]
pub async fn llm_chat_stream(
    messages: Vec<MessageInput>,
    options: Option<CompletionOptions>,
    app: AppHandle,
    llm: State<'_, LlmState>,
) -> Result<()> {
    let router = &llm.0;
    let msgs: Vec<Message> = messages.into_iter().map(|m| m.into()).collect();
    let opts = options.unwrap_or_default();

    let mut stream = router.complete_stream(msgs, opts).await?;

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                if chunk.done {
                    app.emit("llm-done", ()).ok();
                    break;
                }
                app.emit("llm-stream", &chunk).ok();
            }
            Err(e) => {
                app.emit("llm-error", StreamErrorPayload { message: e.to_string() }).ok();
                break;
            }
        }
    }

    Ok(())
}

/// Save API key to OS keyring and update the cloud adapter
#[tauri::command]
pub async fn save_api_key(
    provider: String,
    api_key: String,
    llm: State<'_, LlmState>,
) -> Result<()> {
    tracing::info!(
        provider = %provider,
        key_len = api_key.len(),
        key_prefix = if api_key.len() >= 8 { &api_key[..8] } else { &api_key },
        "save_api_key called"
    );

    if api_key.is_empty() {
        return Err(AppError::Keyring("API key is empty — frontend may have passed the wrong parameter name (Tauri v2 uses camelCase by default, e.g. apiKey not api_key)".into()));
    }

    // Save to OS keyring
    let service = format!("com.eidosinsight.{}", provider);
    let entry = keyring::Entry::new(&service, "api_key")
        .map_err(|e| AppError::Keyring(format!("Failed to create keyring entry: {}", e)))?;

    entry
        .set_password(&api_key)
        .map_err(|e| AppError::Keyring(format!("Failed to save API key: {}", e)))?;

    // Update the cloud adapter in the router
    rebuild_cloud_adapter(&provider, &llm).await?;

    tracing::info!(provider = %provider, "API key saved successfully to keyring");
    Ok(())
}

/// Remove API key from OS keyring
#[tauri::command]
pub async fn remove_api_key(provider: String) -> Result<()> {
    let service = format!("com.eidosinsight.{}", provider);
    let entry = keyring::Entry::new(&service, "api_key")
        .map_err(|e| AppError::Keyring(format!("Failed to create keyring entry: {}", e)))?;

    entry
        .delete_credential()
        .map_err(|e| AppError::Keyring(format!("Failed to remove API key: {}", e)))?;

    Ok(())
}

/// Configure LLM provider and reload adapters
#[tauri::command]
pub async fn configure_llm_provider(
    config: LlmConfig,
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
) -> Result<()> {
    // Save config to settings DB
    {
        let db_conn = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        SettingsRepo::set(db_conn.conn(), "llm_provider", &config.provider)?;
        SettingsRepo::set(db_conn.conn(), "llm_model_name", &config.model_name)?;
        if let Some(ref url) = config.api_base_url {
            SettingsRepo::set(db_conn.conn(), "llm_api_base_url", url)?;
        }
        SettingsRepo::set(db_conn.conn(), "llm_max_tokens", &config.max_tokens.to_string())?;
        SettingsRepo::set(db_conn.conn(), "llm_temperature", &config.temperature.to_string())?;
    }

    // Update cloud adapter if needed
    if config.provider != "ollama" {
        rebuild_cloud_adapter(&config.provider, &llm).await?;
    } else {
        // Clear cloud adapters when switching to Ollama
        llm.0.set_openai_compat_adapter(None).await;
        llm.0.set_anthropic_adapter(None).await;
    }

    Ok(())
}

// --- Internal helpers ---

/// Rebuild the cloud adapter from DB settings and keyring
async fn rebuild_cloud_adapter(provider: &str, llm: &LlmState) -> Result<()> {
    if provider == "ollama" {
        llm.0.set_openai_compat_adapter(None).await;
        llm.0.set_anthropic_adapter(None).await;
        return Ok(());
    }

    // Try to get API key from keyring
    let service = format!("com.eidosinsight.{}", provider);
    let entry = keyring::Entry::new(&service, "api_key")
        .map_err(|e| AppError::Keyring(format!("Failed to access keyring: {}", e)))?;

    let api_key = match entry.get_password() {
        Ok(key) => key,
        Err(_) => {
            // No API key stored yet — just clear the cloud adapters
            llm.0.set_openai_compat_adapter(None).await;
            llm.0.set_anthropic_adapter(None).await;
            return Ok(());
        }
    };

    // Use the router's set_cloud_adapter which routes to correct slot
    llm.0.set_cloud_adapter(provider, &api_key, None).await?;

    Ok(())
}

// ── Wizard: lightweight AI demo ────────────────────────────────────────────────

/// Demo result returned by the AI configuration wizard
#[derive(Debug, Clone, serde::Serialize)]
pub struct WizardDemoResult {
    pub success: bool,
    /// Short demo text generated by the LLM
    pub demo_text: String,
    /// Provider that answered
    pub provider: String,
    /// Model that answered
    pub model: String,
    /// Time in milliseconds the request took
    pub latency_ms: u64,
    /// Error message if success == false
    pub error: Option<String>,
}

/// Run a lightweight AI demo — one-shot, non-streaming, short prompt.
/// Used by the settings wizard to prove the LLM is working.
#[tauri::command]
pub async fn ai_wizard_demo(
    llm: State<'_, LlmState>,
    db: State<'_, DbState>,
) -> Result<WizardDemoResult> {
    let router = &llm.0;

    // Determine current provider & model
    let (provider, model) = {
        let db_conn = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let p = SettingsRepo::get_value(db_conn.conn(), "llm_provider")
            .unwrap_or(Some("ollama".to_string()))
            .unwrap_or_else(|| "ollama".to_string());
        let m = SettingsRepo::get_value(db_conn.conn(), "llm_model_name")
            .unwrap_or(Some("llama3".to_string()))
            .unwrap_or_else(|| "llama3".to_string());
        (p, m)
    };

    let start = std::time::Instant::now();

    let messages = vec![
        Message::system("You are a helpful research assistant. Respond concisely."),
        Message::user("用一句话介绍量子计算的优势。"),
    ];

    let options = CompletionOptions {
        model: Some(model.clone()),
        temperature: Some(0.5),
        max_tokens: Some(150),
        ..Default::default()
    };

    match router.complete(messages, options).await {
        Ok(text) => Ok(WizardDemoResult {
            success: true,
            demo_text: text.trim().to_string(),
            provider,
            model,
            latency_ms: start.elapsed().as_millis() as u64,
            error: None,
        }),
        Err(e) => Ok(WizardDemoResult {
            success: false,
            demo_text: String::new(),
            provider,
            model,
            latency_ms: start.elapsed().as_millis() as u64,
            error: Some(e.to_string()),
        }),
    }
}

// --- DTOs ---

/// Frontend message input
#[derive(Debug, Clone, serde::Deserialize)]
pub struct MessageInput {
    pub role: String,
    pub content: String,
}

impl From<MessageInput> for Message {
    fn from(input: MessageInput) -> Self {
        let role = match input.role.as_str() {
            "system" => Role::System,
            "assistant" => Role::Assistant,
            "tool" => Role::Tool,
            _ => Role::User,
        };
        Message { role, content: input.content }
    }
}

/// Ollama detection result
#[derive(Debug, Clone, serde::Serialize)]
pub struct OllamaDetectionResult {
    pub available: bool,
    pub models: Vec<ModelInfo>,
}

/// LLM status result for frontend
#[derive(Debug, Clone, serde::Serialize)]
pub struct LlmStatusResult {
    pub status: String,
    pub has_cloud: bool,
    pub active_cloud_provider: Option<String>,
}

/// Configured provider info
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConfiguredProvider {
    pub id: String,
    pub name: String,
    pub is_available: bool,
    pub is_local: bool,
}

/// Stream error payload
#[derive(Debug, Clone, serde::Serialize)]
struct StreamErrorPayload {
    message: String,
}
