//! LLM adapter module — unified interface for local (Ollama) and cloud (OpenAI/DeepSeek/Anthropic) LLMs

pub mod adapter;
pub mod anthropic;
pub mod ollama;
pub mod openai_compat;
pub mod preset;
pub mod router;

pub use adapter::*;
pub use anthropic::{AnthropicAdapter, AnthropicConfig};
pub use ollama::{OllamaAdapter, OllamaConfig};
pub use openai_compat::{OpenAiCompatAdapter, OpenAiCompatConfig};
pub use preset::{ProviderPreset, get_all_presets, find_preset};
pub use router::LlmRouter;
