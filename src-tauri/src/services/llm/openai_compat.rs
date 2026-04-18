//! OpenAI-compatible API adapter (works with OpenAI, DeepSeek, and any OpenAI-compatible endpoint)

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;

use super::adapter::*;
use crate::errors::{AppError, Result};

/// OpenAI chat completion request
#[derive(Debug, Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

/// OpenAI chat completion response
#[derive(Debug, Deserialize)]
struct OpenAiChatResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

/// OpenAI streaming response chunk
#[derive(Debug, Deserialize)]
struct OpenAiStreamDelta {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamChoice {
    delta: OpenAiStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamChunk {
    choices: Vec<OpenAiStreamChoice>,
}

/// OpenAI models list response
#[derive(Debug, Deserialize)]
struct OpenAiModelsResponse {
    data: Vec<OpenAiModelItem>,
}

#[derive(Debug, Deserialize)]
struct OpenAiModelItem {
    id: String,
}

/// OpenAI embedding request
#[derive(Debug, Serialize)]
struct OpenAiEmbedRequest {
    model: String,
    input: String,
}

/// OpenAI embedding response
#[derive(Debug, Deserialize)]
struct OpenAiEmbedResponse {
    data: Vec<OpenAiEmbedData>,
}

#[derive(Debug, Deserialize)]
struct OpenAiEmbedData {
    embedding: Vec<f32>,
}

/// OpenAI-compatible adapter configuration
#[derive(Debug, Clone)]
pub struct OpenAiCompatConfig {
    /// API base URL (e.g., "https://api.openai.com/v1" or "https://api.deepseek.com/v1")
    pub base_url: String,
    /// API key
    pub api_key: String,
    /// Default model name
    pub default_model: String,
    /// Provider name for display ("openai", "deepseek", "custom")
    pub provider_label: String,
    /// Request timeout in seconds
    pub timeout_secs: u64,
}

/// OpenAI-compatible API adapter
pub struct OpenAiCompatAdapter {
    client: Client,
    config: OpenAiCompatConfig,
}

impl OpenAiCompatAdapter {
    pub fn new(config: OpenAiCompatConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    fn chat_url(&self) -> String {
        format!("{}/chat/completions", self.config.base_url)
    }

    fn models_url(&self) -> String {
        format!("{}/models", self.config.base_url)
    }

    fn embed_url(&self) -> String {
        format!("{}/embeddings", self.config.base_url)
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.config.api_key)
    }

    fn convert_messages(&self, messages: Vec<Message>) -> Vec<OpenAiMessage> {
        messages
            .into_iter()
            .map(|m| OpenAiMessage {
                role: match m.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::Tool => "tool".to_string(),
                },
                content: m.content,
            })
            .collect()
    }
}

#[async_trait]
impl LlmAdapter for OpenAiCompatAdapter {
    async fn complete(&self, messages: Vec<Message>, options: CompletionOptions) -> Result<String> {
        let model = options.model.clone().unwrap_or_else(|| self.config.default_model.clone());
        let openai_messages = self.convert_messages(messages);

        let request = OpenAiChatRequest {
            model,
            messages: openai_messages,
            stream: Some(false),
            temperature: options.temperature,
            max_tokens: options.max_tokens,
            top_p: options.top_p,
        };

        let response = self
            .client
            .post(self.chat_url())
            .header("Authorization", self.auth_header())
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Llm(format!("{} request failed: {}", self.config.provider_label, e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Llm(format!("{} error ({}): {}", self.config.provider_label, status, body)));
        }

        let chat_response: OpenAiChatResponse = response
            .json()
            .await
            .map_err(|e| AppError::Llm(format!("Failed to parse {} response: {}", self.config.provider_label, e)))?;

        chat_response
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| AppError::Llm(format!("{} returned no choices", self.config.provider_label)))
    }

    async fn complete_stream(
        &self,
        messages: Vec<Message>,
        options: CompletionOptions,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>> {
        let model = options.model.clone().unwrap_or_else(|| self.config.default_model.clone());
        let openai_messages = self.convert_messages(messages);

        let request = OpenAiChatRequest {
            model,
            messages: openai_messages,
            stream: Some(true),
            temperature: options.temperature,
            max_tokens: options.max_tokens,
            top_p: options.top_p,
        };

        let stream_client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to create streaming HTTP client");

        let response = stream_client
            .post(self.chat_url())
            .header("Authorization", self.auth_header())
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Llm(format!("{} stream request failed: {}", self.config.provider_label, e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Llm(format!("{} stream error ({}): {}", self.config.provider_label, status, body)));
        }

        // Clone provider label to move into the closure (avoids lifetime issue with &self)
        let provider_label = self.config.provider_label.clone();

        let stream = response
            .bytes_stream()
            .scan(String::new(), move |buffer, chunk_result| {
                let label = provider_label.clone();
                match chunk_result {
                    Ok(bytes) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));
                        let mut results = Vec::new();

                        // OpenAI SSE format: "data: {...}\n\n"
                        while let Some(pos) = buffer.find("\n\n") {
                            let chunk = buffer[..pos].to_string();
                            buffer.drain(..=pos + 1);

                            for line in chunk.lines() {
                                let line = line.trim();
                                if !line.starts_with("data: ") {
                                    continue;
                                }
                                let json_str = &line[6..];
                                if json_str == "[DONE]" {
                                    results.push(Ok(StreamChunk {
                                        content: String::new(),
                                        done: true,
                                    }));
                                    continue;
                                }

                                match serde_json::from_str::<OpenAiStreamChunk>(json_str) {
                                    Ok(sse_chunk) => {
                                        if let Some(choice) = sse_chunk.choices.first() {
                                            let content = choice.delta.content.clone().unwrap_or_default();
                                            let done = choice.finish_reason.as_deref() == Some("stop");
                                            results.push(Ok(StreamChunk { content, done }));
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!("Failed to parse {} stream chunk: {}", label, e);
                                    }
                                }
                            }
                        }

                        std::future::ready(Some(futures::stream::iter(results)))
                    }
                    Err(e) => {
                        std::future::ready(Some(futures::stream::iter(vec![Err(
                            AppError::Llm(format!("{} stream read error: {}", label, e)),
                        )])))
                    }
                }
            })
            .flatten();

        Ok(Box::pin(stream))
    }

    async fn embed(&self, text: &str, model: Option<&str>) -> Result<Vec<f32>> {
        let model_name = model
            .map(|m| m.to_string())
            .unwrap_or_else(|| "text-embedding-3-small".to_string());

        let request = OpenAiEmbedRequest {
            model: model_name,
            input: text.to_string(),
        };

        let response = self
            .client
            .post(self.embed_url())
            .header("Authorization", self.auth_header())
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Llm(format!("{} embed request failed: {}", self.config.provider_label, e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Llm(format!("{} embed error ({}): {}", self.config.provider_label, status, body)));
        }

        let embed_response: OpenAiEmbedResponse = response
            .json()
            .await
            .map_err(|e| AppError::Llm(format!("Failed to parse {} embed response: {}", self.config.provider_label, e)))?;

        embed_response
            .data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .ok_or_else(|| AppError::Llm(format!("{} returned no embedding data", self.config.provider_label)))
    }

    async fn health_check(&self) -> bool {
        self.client
            .get(self.models_url())
            .header("Authorization", self.auth_header())
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let response = self
            .client
            .get(self.models_url())
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| AppError::Llm(format!("Failed to list {} models: {}", self.config.provider_label, e)))?;

        if !response.status().is_success() {
            return Err(AppError::Llm(format!(
                "{} models endpoint returned {}",
                self.config.provider_label,
                response.status()
            )));
        }

        let models_response: OpenAiModelsResponse = response
            .json()
            .await
            .map_err(|e| AppError::Llm(format!("Failed to parse {} models: {}", self.config.provider_label, e)))?;

        Ok(models_response
            .data
            .into_iter()
            .map(|m| ModelInfo {
                id: m.id.clone(),
                name: m.id,
                size_bytes: None,
                quantization: None,
            })
            .collect())
    }

    fn provider_name(&self) -> &str {
        &self.config.provider_label
    }

    fn is_local(&self) -> bool {
        false
    }
}
