//! LLM Router — selects the best available adapter with fallback strategy

use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::adapter::*;
use super::ollama::OllamaAdapter;
use super::openai_compat::OpenAiCompatAdapter;
use crate::errors::{AppError, Result};

/// The LLM router holds available adapters and selects the best one
/// Strategy: prefer local (Ollama) → fallback to cloud (if configured) → return error
pub struct LlmRouter {
    ollama: Arc<OllamaAdapter>,
    cloud: RwLock<Option<Arc<OpenAiCompatAdapter>>>,
}

impl LlmRouter {
    /// Create a new router with the Ollama adapter (always present as default)
    pub fn new(ollama: OllamaAdapter) -> Self {
        Self {
            ollama: Arc::new(ollama),
            cloud: RwLock::new(None),
        }
    }

    /// Create with a cloud adapter pre-configured
    pub fn with_cloud(ollama: OllamaAdapter, cloud: OpenAiCompatAdapter) -> Self {
        Self {
            ollama: Arc::new(ollama),
            cloud: RwLock::new(Some(Arc::new(cloud))),
        }
    }

    /// Update the cloud adapter (called when user configures API key)
    pub async fn set_cloud_adapter(&self, adapter: Option<OpenAiCompatAdapter>) {
        let mut cloud = self.cloud.write().await;
        *cloud = adapter.map(Arc::new);
    }

    /// Select the best available adapter
    /// Priority: Ollama (if healthy) → Cloud (if configured)
    async fn select_adapter(&self) -> Result<Arc<dyn LlmAdapter>> {
        // 1. Try local Ollama first
        if self.ollama.health_check().await {
            return Ok(self.ollama.clone());
        }

        // 2. Fall back to cloud adapter
        let cloud = self.cloud.read().await;
        if let Some(cloud_adapter) = cloud.as_ref() {
            if cloud_adapter.health_check().await {
                return Ok(cloud_adapter.clone());
            }
        }

        Err(AppError::Llm("No LLM adapter available. Please configure Ollama or a cloud API.".to_string()))
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

        // Check cloud
        let cloud = self.cloud.read().await;
        if let Some(cloud_adapter) = cloud.as_ref() {
            if cloud_adapter.health_check().await {
                return LlmStatus::CloudAvailable;
            }
        }

        LlmStatus::Unavailable
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
        self.cloud.read().await.is_some()
    }
}
