//! LLM adapter module — unified interface for local (Ollama) and cloud (OpenAI/DeepSeek) LLMs

pub mod adapter;
pub mod ollama;
pub mod openai_compat;
pub mod router;

pub use adapter::*;
pub use ollama::{OllamaAdapter, OllamaConfig};
pub use openai_compat::{OpenAiCompatAdapter, OpenAiCompatConfig};
pub use router::LlmRouter;
