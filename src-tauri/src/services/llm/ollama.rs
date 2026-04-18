//! Ollama local LLM adapter

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;

use super::adapter::*;
use crate::errors::{AppError, Result};

/// Ollama API request for chat completion
#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,
}

/// Ollama chat response (non-streaming)
#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaMessageResponse,
    #[allow(dead_code)]
    done: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaMessageResponse {
    content: String,
}

/// Ollama streaming response chunk
#[derive(Debug, Deserialize)]
struct OllamaStreamChunk {
    message: OllamaMessageResponse,
    done: bool,
}

/// Ollama models list response
#[derive(Debug, Deserialize)]
struct OllamaModelsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
    #[serde(rename = "size")]
    size_bytes: Option<u64>,
    #[allow(dead_code)]
    quantization_level: Option<String>,
}

/// Ollama embedding request
#[derive(Debug, Serialize)]
struct OllamaEmbedRequest {
    model: String,
    prompt: String,
}

/// Ollama embedding response
#[derive(Debug, Deserialize)]
struct OllamaEmbedResponse {
    embedding: Vec<f32>,
}

/// Ollama adapter configuration
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub base_url: String,
    pub default_model: String,
    pub timeout_secs: u64,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            default_model: "llama3".to_string(),
            timeout_secs: 30,
        }
    }
}

/// Ollama local LLM adapter
pub struct OllamaAdapter {
    client: Client,
    config: OllamaConfig,
}

impl OllamaAdapter {
    pub fn new(config: OllamaConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    pub fn with_default_config() -> Self {
        Self::new(OllamaConfig::default())
    }

    fn chat_url(&self) -> String {
        format!("{}/api/chat", self.config.base_url)
    }

    fn models_url(&self) -> String {
        format!("{}/api/tags", self.config.base_url)
    }

    fn embed_url(&self) -> String {
        format!("{}/api/embeddings", self.config.base_url)
    }

    fn convert_messages(&self, messages: Vec<Message>) -> Vec<OllamaMessage> {
        messages
            .into_iter()
            .map(|m| OllamaMessage {
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
impl LlmAdapter for OllamaAdapter {
    async fn complete(&self, messages: Vec<Message>, options: CompletionOptions) -> Result<String> {
        let model = options.model.clone().unwrap_or_else(|| self.config.default_model.clone());
        let ollama_messages = self.convert_messages(messages);

        let ollama_options = OllamaOptions {
            temperature: options.temperature,
            num_predict: options.max_tokens,
            top_p: options.top_p,
        };

        let request = OllamaChatRequest {
            model,
            messages: ollama_messages,
            stream: Some(false),
            options: Some(ollama_options),
        };

        let response = self
            .client
            .post(self.chat_url())
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Llm(format!("Ollama request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Llm(format!("Ollama error ({}): {}", status, body)));
        }

        let chat_response: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| AppError::Llm(format!("Failed to parse Ollama response: {}", e)))?;

        Ok(chat_response.message.content)
    }

    async fn complete_stream(
        &self,
        messages: Vec<Message>,
        options: CompletionOptions,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>> {
        let model = options.model.clone().unwrap_or_else(|| self.config.default_model.clone());
        let ollama_messages = self.convert_messages(messages);

        let ollama_options = OllamaOptions {
            temperature: options.temperature,
            num_predict: options.max_tokens,
            top_p: options.top_p,
        };

        let request = OllamaChatRequest {
            model,
            messages: ollama_messages,
            stream: Some(true),
            options: Some(ollama_options),
        };

        // Use a longer timeout for streaming requests
        let stream_client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to create streaming HTTP client");

        let response = stream_client
            .post(self.chat_url())
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Llm(format!("Ollama stream request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Llm(format!("Ollama stream error ({}): {}", status, body)));
        }

        let stream = response
            .bytes_stream()
            .scan(String::new(), |buffer, chunk_result| {
                match chunk_result {
                    Ok(bytes) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));
                        let mut results = Vec::new();

                        // Ollama streams newline-delimited JSON
                        while let Some(pos) = buffer.find('\n') {
                            let line = buffer[..pos].trim().to_string();
                            buffer.drain(..=pos);

                            if line.is_empty() {
                                continue;
                            }

                            match serde_json::from_str::<OllamaStreamChunk>(&line) {
                                Ok(chunk) => {
                                    results.push(Ok(StreamChunk {
                                        content: chunk.message.content,
                                        done: chunk.done,
                                    }));
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to parse Ollama stream chunk: {} - line: {}", e, line);
                                }
                            }
                        }

                        std::future::ready(Some(futures::stream::iter(results)))
                    }
                    Err(e) => {
                        std::future::ready(Some(futures::stream::iter(vec![Err(
                            AppError::Llm(format!("Ollama stream read error: {}", e)),
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
            .unwrap_or_else(|| "nomic-embed-text".to_string());

        let request = OllamaEmbedRequest {
            model: model_name,
            prompt: text.to_string(),
        };

        let response = self
            .client
            .post(self.embed_url())
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Llm(format!("Ollama embed request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Llm(format!("Ollama embed error ({}): {}", status, body)));
        }

        let embed_response: OllamaEmbedResponse = response
            .json()
            .await
            .map_err(|e| AppError::Llm(format!("Failed to parse Ollama embed response: {}", e)))?;

        Ok(embed_response.embedding)
    }

    async fn health_check(&self) -> bool {
        self.client
            .get(self.models_url())
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
            .send()
            .await
            .map_err(|e| AppError::Llm(format!("Failed to list Ollama models: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Llm(format!(
                "Ollama models endpoint returned {}",
                response.status()
            )));
        }

        let models_response: OllamaModelsResponse = response
            .json()
            .await
            .map_err(|e| AppError::Llm(format!("Failed to parse Ollama models: {}", e)))?;

        Ok(models_response
            .models
            .into_iter()
            .map(|m| ModelInfo {
                id: m.name.clone(),
                name: m.name,
                size_bytes: m.size_bytes,
                quantization: m.quantization_level,
            })
            .collect())
    }

    fn provider_name(&self) -> &str {
        "ollama"
    }

    fn is_local(&self) -> bool {
        true
    }
}
