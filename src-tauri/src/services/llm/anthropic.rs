//! Anthropic API adapter — independent adapter with non-OpenAI-compatible API format
//!
//! Key differences from OpenAI:
//! - Auth: `x-api-key` header (not `Authorization: Bearer`)
//! - Endpoint: `/v1/messages` (not `/v1/chat/completions`)
//! - System prompt: top-level `system` field (not in messages array)
//! - `max_tokens` is required (not optional)
//! - SSE events: `content_block_delta` / `message_stop` (not `choices[0].delta`)
//! - No embedding endpoint

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;

use super::adapter::*;
use crate::errors::{AppError, Result};

/// Anthropic Messages API request
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    max_tokens: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Anthropic Messages API response
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContentBlock>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

/// Anthropic streaming event types
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum AnthropicStreamEvent {
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { delta: AnthropicDelta },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "message_start")]
    MessageStart,
    #[serde(rename = "content_block_start")]
    ContentBlockStart,
    #[serde(rename = "content_block_stop")]
    ContentBlockStop,
    #[serde(rename = "ping")]
    Ping,
}

#[derive(Debug, Deserialize)]
struct AnthropicDelta {
    #[serde(rename = "type")]
    delta_type: Option<String>,
    text: Option<String>,
}

/// Anthropic adapter configuration
#[derive(Debug, Clone)]
pub struct AnthropicConfig {
    /// API base URL (e.g., "https://api.anthropic.com")
    pub base_url: String,
    /// API key
    pub api_key: String,
    /// Default model name
    pub default_model: String,
    /// Anthropic API version header
    pub api_version: String,
    /// Request timeout in seconds
    pub timeout_secs: u64,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.anthropic.com".to_string(),
            api_key: String::new(),
            default_model: "claude-sonnet-4-20250514".to_string(),
            api_version: "2023-06-01".to_string(),
            timeout_secs: 30,
        }
    }
}

/// Anthropic API adapter
pub struct AnthropicAdapter {
    client: Client,
    config: AnthropicConfig,
}

impl AnthropicAdapter {
    pub fn new(config: AnthropicConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");
        Self { client, config }
    }

    fn messages_url(&self) -> String {
        format!("{}/v1/messages", self.config.base_url)
    }

    fn convert_messages(&self, messages: Vec<Message>) -> (Vec<AnthropicMessage>, Option<String>) {
        let mut system_prompt = None;
        let mut anthropic_messages = Vec::new();

        for msg in messages {
            match msg.role {
                Role::System => {
                    system_prompt = Some(msg.content);
                }
                _ => {
                    anthropic_messages.push(AnthropicMessage {
                        role: match msg.role {
                            Role::User => "user".to_string(),
                            Role::Assistant => "assistant".to_string(),
                            Role::Tool => "user".to_string(), // Anthropic doesn't have tool role, map to user
                            _ => "user".to_string(),
                        },
                        content: msg.content,
                    });
                }
            }
        }

        (anthropic_messages, system_prompt)
    }
}

#[async_trait]
impl LlmAdapter for AnthropicAdapter {
    async fn complete(&self, messages: Vec<Message>, options: CompletionOptions) -> Result<String> {
        let model = options.model.clone().unwrap_or_else(|| self.config.default_model.clone());
        let (anthropic_messages, system_prompt) = self.convert_messages(messages);

        let request = AnthropicRequest {
            model,
            messages: anthropic_messages,
            system: system_prompt,
            max_tokens: options.max_tokens.unwrap_or(4096),
            stream: None,
            temperature: options.temperature,
            top_p: options.top_p,
        };

        let response = self
            .client
            .post(self.messages_url())
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", &self.config.api_version)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Llm(format!("Anthropic request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Llm(format!("Anthropic error ({}): {}", status, body)));
        }

        let api_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| AppError::Llm(format!("Failed to parse Anthropic response: {}", e)))?;

        api_response
            .content
            .into_iter()
            .filter_map(|block| {
                if block.block_type == "text" {
                    block.text
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("")
            .into_result("Anthropic returned no content")
    }

    async fn complete_stream(
        &self,
        messages: Vec<Message>,
        options: CompletionOptions,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>> {
        let model = options.model.clone().unwrap_or_else(|| self.config.default_model.clone());
        let (anthropic_messages, system_prompt) = self.convert_messages(messages);

        let request = AnthropicRequest {
            model,
            messages: anthropic_messages,
            system: system_prompt,
            max_tokens: options.max_tokens.unwrap_or(4096),
            stream: Some(true),
            temperature: options.temperature,
            top_p: options.top_p,
        };

        let stream_client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to create streaming HTTP client");

        let response = stream_client
            .post(self.messages_url())
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", &self.config.api_version)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Llm(format!("Anthropic stream request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Llm(format!("Anthropic stream error ({}): {}", status, body)));
        }

        let stream = response
            .bytes_stream()
            .scan(String::new(), |buffer, chunk_result| {
                match chunk_result {
                    Ok(bytes) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));
                        let mut results = Vec::new();

                        // Anthropic SSE format: "event: xxx\ndata: {...}\n\n"
                        while let Some(pos) = buffer.find("\n\n") {
                            let chunk = buffer[..pos].to_string();
                            buffer.drain(..=pos + 1);

                            for line in chunk.lines() {
                                let line = line.trim();
                                if !line.starts_with("data: ") {
                                    continue;
                                }
                                let json_str = &line[6..];

                                match serde_json::from_str::<AnthropicStreamEvent>(json_str) {
                                    Ok(AnthropicStreamEvent::ContentBlockDelta { delta }) => {
                                        let content = delta.text.unwrap_or_default();
                                        if !content.is_empty() {
                                            results.push(Ok(StreamChunk {
                                                content,
                                                done: false,
                                            }));
                                        }
                                    }
                                    Ok(AnthropicStreamEvent::MessageStop) => {
                                        results.push(Ok(StreamChunk {
                                            content: String::new(),
                                            done: true,
                                        }));
                                    }
                                    Ok(_) => {
                                        // Other events (ping, start, etc.) — ignore
                                    }
                                    Err(e) => {
                                        tracing::warn!("Failed to parse Anthropic stream event: {}", e);
                                    }
                                }
                            }
                        }

                        std::future::ready(Some(futures::stream::iter(results)))
                    }
                    Err(e) => {
                        std::future::ready(Some(futures::stream::iter(vec![Err(
                            AppError::Llm(format!("Anthropic stream read error: {}", e)),
                        )])))
                    }
                }
            })
            .flatten();

        Ok(Box::pin(stream))
    }

    async fn embed(&self, _text: &str, _model: Option<&str>) -> Result<Vec<f32>> {
        // Anthropic does not provide an embedding endpoint
        Err(AppError::Llm("Anthropic does not support embeddings".to_string()))
    }

    async fn health_check(&self) -> bool {
        // Anthropic doesn't have a lightweight health check endpoint
        // We just check if the API key is set
        !self.config.api_key.is_empty()
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        // Anthropic doesn't have a models list endpoint
        // Return known models
        Ok(vec![
            ModelInfo {
                id: "claude-sonnet-4-20250514".to_string(),
                name: "Claude Sonnet 4".to_string(),
                size_bytes: None,
                quantization: None,
            },
            ModelInfo {
                id: "claude-3-5-sonnet-20241022".to_string(),
                name: "Claude 3.5 Sonnet".to_string(),
                size_bytes: None,
                quantization: None,
            },
            ModelInfo {
                id: "claude-3-5-haiku-20241022".to_string(),
                name: "Claude 3.5 Haiku".to_string(),
                size_bytes: None,
                quantization: None,
            },
            ModelInfo {
                id: "claude-3-opus-20240229".to_string(),
                name: "Claude 3 Opus".to_string(),
                size_bytes: None,
                quantization: None,
            },
        ])
    }

    fn provider_name(&self) -> &str {
        "anthropic"
    }

    fn is_local(&self) -> bool {
        false
    }
}

/// Helper trait to convert Option<String> into Result
trait IntoResult {
    fn into_result(self, msg: &str) -> Result<String>;
}

impl IntoResult for Option<String> {
    fn into_result(self, msg: &str) -> Result<String> {
        self.ok_or_else(|| AppError::Llm(msg.to_string()))
    }
}

impl IntoResult for String {
    fn into_result(self, _msg: &str) -> Result<String> {
        Ok(self)
    }
}
