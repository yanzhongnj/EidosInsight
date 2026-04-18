//! LLM adapter trait and shared types

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use crate::errors::Result;

/// Role of a message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// A single message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: Role::System, content: content.into() }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self { role: Role::User, content: content.into() }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: Role::Assistant, content: content.into() }
    }
}

/// A chunk of streamed LLM response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub content: String,
    pub done: bool,
}

/// LLM completion options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionOptions {
    pub model: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<i32>,
    pub top_p: Option<f64>,
}

impl Default for CompletionOptions {
    fn default() -> Self {
        Self {
            model: None,
            temperature: Some(0.7),
            max_tokens: Some(4096),
            top_p: None,
        }
    }
}

/// LLM status reported to the frontend
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LlmStatus {
    /// Local Ollama is available
    LocalAvailable,
    /// Cloud API is configured and reachable
    CloudAvailable,
    /// No LLM is available
    Unavailable,
}

/// A model descriptor returned by the adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub size_bytes: Option<u64>,
    pub quantization: Option<String>,
}

/// Unified LLM adapter trait — all providers implement this
#[async_trait]
pub trait LlmAdapter: Send + Sync {
    /// Non-streaming completion
    async fn complete(&self, messages: Vec<Message>, options: CompletionOptions) -> Result<String>;

    /// Streaming completion — returns a Stream of chunks
    async fn complete_stream(
        &self,
        messages: Vec<Message>,
        options: CompletionOptions,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>>;

    /// Generate embedding vector for a text
    async fn embed(&self, text: &str, model: Option<&str>) -> Result<Vec<f32>>;

    /// Health check — returns true if the backend is reachable
    async fn health_check(&self) -> bool;

    /// List available models
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;

    /// Adapter identifier (e.g. "ollama", "openai", "deepseek")
    fn provider_name(&self) -> &str;

    /// Whether this is a local adapter
    fn is_local(&self) -> bool;
}
