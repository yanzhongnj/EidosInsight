//! LLM Router — selects the best available adapter with fallback strategy
//!
//! Architecture: 3 adapter slots
//! 1. `ollama` — always present, local Ollama adapter
//! 2. `openai_compat` — shared OpenAI-compatible adapter (OpenAI, DeepSeek, Qwen, etc.)
//! 3. `anthropic` — independent Anthropic adapter (Claude)

use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::adapter::*;
use super::anthropic::AnthropicAdapter;
use super::ollama::OllamaAdapter;
use super::openai_compat::OpenAiCompatAdapter;
use crate::errors::{AppError, Result};

/// The LLM router holds available adapters and selects the best one
/// Strategy: prefer local (Ollama) → fallback to cloud (if configured) → return error
pub struct LlmRouter {
    ollama: Arc<OllamaAdapter>,
    openai_compat: RwLock<Option<Arc<OpenAiCompatAdapter>>>,
    anthropic: RwLock<Option<Arc<AnthropicAdapter>>>,
}

/// Which provider slot is currently active for cloud usage
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ActiveProvider {
    Ollama,
    OpenAiCompat,
    Anthropic,
}

impl LlmRouter {
    /// Create a new router with the Ollama adapter (always present as default)
    pub fn new(ollama: OllamaAdapter) -> Self {
        Self {
            ollama: Arc::new(ollama),
            openai_compat: RwLock::new(None),
            anthropic: RwLock::new(None),
        }
    }

    /// Create with a cloud adapter pre-configured (backward compat)
    pub fn with_cloud(ollama: OllamaAdapter, cloud: OpenAiCompatAdapter) -> Self {
        Self {
            ollama: Arc::new(ollama),
            openai_compat: RwLock::new(Some(Arc::new(cloud))),
            anthropic: RwLock::new(None),
        }
    }

    /// Update the OpenAI-compat cloud adapter
    pub async fn set_openai_compat_adapter(&self, adapter: Option<OpenAiCompatAdapter>) {
        let mut slot = self.openai_compat.write().await;
        *slot = adapter.map(Arc::new);
    }

    /// Update the Anthropic adapter
    pub async fn set_anthropic_adapter(&self, adapter: Option<AnthropicAdapter>) {
        let mut slot = self.anthropic.write().await;
        *slot = adapter.map(Arc::new);
    }

    /// Backward compat: set_cloud_adapter routes to the correct slot based on provider type
    pub async fn set_cloud_adapter(&self, provider: &str, api_key: &str, base_url: Option<&str>) -> Result<()> {
        match provider {
            "anthropic" | "claude" => {
                let config = super::anthropic::AnthropicConfig {
                    base_url: base_url
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "https://api.anthropic.com".to_string()),
                    api_key: api_key.to_string(),
                    default_model: "claude-sonnet-4-20250514".to_string(),
                    ..Default::default()
                };
                self.set_anthropic_adapter(Some(AnthropicAdapter::new(config))).await;
                // Clear the openai_compat slot when switching to anthropic
                self.set_openai_compat_adapter(None).await;
            }
            "ollama" => {
                self.set_openai_compat_adapter(None).await;
                self.set_anthropic_adapter(None).await;
            }
            _ => {
                // All other providers use OpenAI-compat
                let preset = super::preset::find_preset(provider);
                let (default_url, default_model) = match &preset {
                    Some(p) => (p.base_url.clone(), p.default_model.clone()),
                    None => (
                        base_url.map(|s| s.to_string()).unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
                        "gpt-4o-mini".to_string(),
                    ),
                };

                let config = super::openai_compat::OpenAiCompatConfig {
                    base_url: if let Some(url) = base_url { url.to_string() } else { default_url },
                    api_key: api_key.to_string(),
                    default_model,
                    provider_label: provider.to_string(),
                    timeout_secs: 15,
                };
                self.set_openai_compat_adapter(Some(OpenAiCompatAdapter::new(config))).await;
                // Clear the anthropic slot when switching to openai_compat
                self.set_anthropic_adapter(None).await;
            }
        }
        Ok(())
    }

    /// Select the best available adapter based on configured provider
    async fn select_adapter(&self) -> Result<Arc<dyn LlmAdapter>> {
        // 1. Try local Ollama first (always preferred for speed & privacy)
        if self.ollama.health_check().await {
            return Ok(self.ollama.clone());
        }

        // 2. Try Anthropic adapter
        let anthropic = self.anthropic.read().await;
        if let Some(adapter) = anthropic.as_ref() {
            if adapter.health_check().await {
                return Ok(adapter.clone());
            }
        }
        drop(anthropic);

        // 3. Try OpenAI-compat adapter
        let openai = self.openai_compat.read().await;
        if let Some(adapter) = openai.as_ref() {
            if adapter.health_check().await {
                return Ok(adapter.clone());
            }
        }

        Err(AppError::Llm("No LLM adapter available. Please configure Ollama or a cloud API.".to_string()))
    }

    /// Select a specific provider adapter by name
    pub async fn select_provider(&self, provider: &str) -> Result<Arc<dyn LlmAdapter>> {
        match provider {
            "ollama" => {
                if self.ollama.health_check().await {
                    Ok(self.ollama.clone())
                } else {
                    Err(AppError::Llm("Ollama is not available".to_string()))
                }
            }
            "anthropic" | "claude" => {
                let anthropic = self.anthropic.read().await;
                match anthropic.as_ref() {
                    Some(adapter) => Ok(adapter.clone()),
                    None => Err(AppError::Llm("Anthropic adapter is not configured".to_string())),
                }
            }
            _ => {
                let openai = self.openai_compat.read().await;
                match openai.as_ref() {
                    Some(adapter) => Ok(adapter.clone()),
                    None => Err(AppError::Llm(format!("{} adapter is not configured", provider))),
                }
            }
        }
    }

    /// Get the Ollama adapter directly (for model listing, detection, etc.)
    pub fn ollama_adapter(&self) -> &OllamaAdapter {
        &self.ollama
    }

    /// Get the current LLM status
    pub async fn get_status(&self) -> LlmStatus {
        // Check local first
        if self.ollama.health_check().await {
            return LlmStatus::LocalAvailable;
        }

        // Check Anthropic
        let anthropic = self.anthropic.read().await;
        if let Some(adapter) = anthropic.as_ref() {
            if adapter.health_check().await {
                return LlmStatus::CloudAvailable;
            }
        }
        drop(anthropic);

        // Check OpenAI-compat
        let openai = self.openai_compat.read().await;
        if let Some(adapter) = openai.as_ref() {
            if adapter.health_check().await {
                return LlmStatus::CloudAvailable;
            }
        }

        LlmStatus::Unavailable
    }

    /// Get the active cloud provider name
    pub async fn active_cloud_provider(&self) -> Option<String> {
        let anthropic = self.anthropic.read().await;
        if anthropic.is_some() {
            return Some("anthropic".to_string());
        }
        drop(anthropic);

        let openai = self.openai_compat.read().await;
        if let Some(adapter) = openai.as_ref() {
            return Some(adapter.provider_name().to_string());
        }

        None
    }

    // --- Delegate methods to the selected adapter ---

    /// Non-streaming completion via the best available adapter
    pub async fn complete(&self, messages: Vec<Message>, options: CompletionOptions) -> Result<String> {
        let adapter = self.select_adapter().await?;
        adapter.complete(messages, options).await
    }

    /// Streaming completion via the best available adapter
    pub async fn complete_stream(
        &self,
        messages: Vec<Message>,
        options: CompletionOptions,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<StreamChunk>> + Send>>> {
        let adapter = self.select_adapter().await?;
        adapter.complete_stream(messages, options).await
    }

    /// Generate embedding via the best available adapter
    pub async fn embed(&self, text: &str, model: Option<&str>) -> Result<Vec<f32>> {
        let adapter = self.select_adapter().await?;
        adapter.embed(text, model).await
    }

    /// List models from the best available adapter
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let adapter = self.select_adapter().await?;
        adapter.list_models().await
    }

    /// List Ollama models specifically
    pub async fn list_ollama_models(&self) -> Result<Vec<ModelInfo>> {
        self.ollama.list_models().await
    }

    /// Check if Ollama is available
    pub async fn is_ollama_available(&self) -> bool {
        self.ollama.health_check().await
    }

    /// Check if a cloud adapter is configured
    pub async fn has_cloud_adapter(&self) -> bool {
        let anthropic = self.anthropic.read().await;
        if anthropic.is_some() {
            return true;
        }
        drop(anthropic);
        let openai = self.openai_compat.read().await;
        openai.is_some()
    }

    /// Get a reference to the Anthropic adapter (for listing configured providers)
    pub async fn get_anthropic_adapter(&self) -> tokio::sync::RwLockReadGuard<'_, Option<Arc<AnthropicAdapter>>> {
        self.anthropic.read().await
    }

    /// Get a reference to the OpenAI-compat adapter (for listing configured providers)
    pub async fn get_openai_compat_adapter(&self) -> tokio::sync::RwLockReadGuard<'_, Option<Arc<OpenAiCompatAdapter>>> {
        self.openai_compat.read().await
    }
}
