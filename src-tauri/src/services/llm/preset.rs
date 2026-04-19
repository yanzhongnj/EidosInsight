//! Provider presets — predefined configurations for all supported LLM providers

use serde::{Deserialize, Serialize};

/// A provider preset defines the default configuration for a specific LLM provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPreset {
    /// Unique provider identifier (e.g., "openai", "anthropic", "deepseek")
    pub id: String,
    /// Display name
    pub name: String,
    /// Adapter type: "ollama" | "openai_compat" | "anthropic"
    pub adapter_type: String,
    /// Default API base URL
    pub base_url: String,
    /// Default model name
    pub default_model: String,
    /// Whether this provider requires an API key
    pub requires_api_key: bool,
    /// Whether this is a local provider
    pub is_local: bool,
    /// Available models for selection
    pub available_models: Vec<String>,
    /// Description
    pub description: String,
}

/// Get all supported provider presets
pub fn get_all_presets() -> Vec<ProviderPreset> {
    vec![
        ProviderPreset {
            id: "ollama".to_string(),
            name: "Ollama (本地)".to_string(),
            adapter_type: "ollama".to_string(),
            base_url: "http://localhost:11434/v1".to_string(),
            default_model: "llama3".to_string(),
            requires_api_key: false,
            is_local: true,
            available_models: vec![
                "llama3".to_string(),
                "llama3.1".to_string(),
                "mistral".to_string(),
                "qwen2".to_string(),
                "gemma2".to_string(),
                "phi3".to_string(),
            ],
            description: "本地运行的开源大语言模型，无需 API Key".to_string(),
        },
        ProviderPreset {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            adapter_type: "openai_compat".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            default_model: "gpt-4o-mini".to_string(),
            requires_api_key: true,
            is_local: false,
            available_models: vec![
                "gpt-4o".to_string(),
                "gpt-4o-mini".to_string(),
                "gpt-4-turbo".to_string(),
                "gpt-3.5-turbo".to_string(),
            ],
            description: "OpenAI GPT 系列模型".to_string(),
        },
        ProviderPreset {
            id: "anthropic".to_string(),
            name: "Anthropic Claude".to_string(),
            adapter_type: "anthropic".to_string(),
            base_url: "https://api.anthropic.com".to_string(),
            default_model: "claude-sonnet-4-20250514".to_string(),
            requires_api_key: true,
            is_local: false,
            available_models: vec![
                "claude-sonnet-4-20250514".to_string(),
                "claude-3-5-sonnet-20241022".to_string(),
                "claude-3-5-haiku-20241022".to_string(),
                "claude-3-opus-20240229".to_string(),
            ],
            description: "Anthropic Claude 系列模型（独立 API 格式）".to_string(),
        },
        ProviderPreset {
            id: "deepseek".to_string(),
            name: "DeepSeek".to_string(),
            adapter_type: "openai_compat".to_string(),
            base_url: "https://api.deepseek.com/v1".to_string(),
            default_model: "deepseek-chat".to_string(),
            requires_api_key: true,
            is_local: false,
            available_models: vec![
                "deepseek-chat".to_string(),
                "deepseek-reasoner".to_string(),
            ],
            description: "DeepSeek 深度求索大模型".to_string(),
        },
        ProviderPreset {
            id: "qwen".to_string(),
            name: "通义千问 (Qwen)".to_string(),
            adapter_type: "openai_compat".to_string(),
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            default_model: "qwen-plus".to_string(),
            requires_api_key: true,
            is_local: false,
            available_models: vec![
                "qwen-max".to_string(),
                "qwen-plus".to_string(),
                "qwen-turbo".to_string(),
                "qwen-long".to_string(),
            ],
            description: "阿里云通义千问大模型".to_string(),
        },
        ProviderPreset {
            id: "zhipu".to_string(),
            name: "智谱 GLM".to_string(),
            adapter_type: "openai_compat".to_string(),
            base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
            default_model: "glm-4-plus".to_string(),
            requires_api_key: true,
            is_local: false,
            available_models: vec![
                "glm-4-plus".to_string(),
                "glm-4".to_string(),
                "glm-4-flash".to_string(),
                "glm-4-long".to_string(),
            ],
            description: "智谱 AI GLM 系列大模型".to_string(),
        },
        ProviderPreset {
            id: "moonshot".to_string(),
            name: "Moonshot (Kimi)".to_string(),
            adapter_type: "openai_compat".to_string(),
            base_url: "https://api.moonshot.cn/v1".to_string(),
            default_model: "moonshot-v1-8k".to_string(),
            requires_api_key: true,
            is_local: false,
            available_models: vec![
                "moonshot-v1-8k".to_string(),
                "moonshot-v1-32k".to_string(),
                "moonshot-v1-128k".to_string(),
            ],
            description: "月之暗面 Kimi 大模型".to_string(),
        },
        ProviderPreset {
            id: "siliconflow".to_string(),
            name: "SiliconFlow".to_string(),
            adapter_type: "openai_compat".to_string(),
            base_url: "https://api.siliconflow.cn/v1".to_string(),
            default_model: "Qwen/Qwen2.5-7B-Instruct".to_string(),
            requires_api_key: true,
            is_local: false,
            available_models: vec![
                "Qwen/Qwen2.5-7B-Instruct".to_string(),
                "Qwen/Qwen2.5-72B-Instruct".to_string(),
                "deepseek-ai/DeepSeek-V2.5".to_string(),
                "meta-llama/Meta-Llama-3.1-8B-Instruct".to_string(),
            ],
            description: "SiliconFlow 云端推理平台".to_string(),
        },
        ProviderPreset {
            id: "gemini".to_string(),
            name: "Google Gemini".to_string(),
            adapter_type: "openai_compat".to_string(),
            base_url: "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
            default_model: "gemini-2.0-flash".to_string(),
            requires_api_key: true,
            is_local: false,
            available_models: vec![
                "gemini-2.0-flash".to_string(),
                "gemini-1.5-pro".to_string(),
                "gemini-1.5-flash".to_string(),
            ],
            description: "Google Gemini 大模型（OpenAI 兼容模式）".to_string(),
        },
        ProviderPreset {
            id: "custom".to_string(),
            name: "自定义 (Custom)".to_string(),
            adapter_type: "openai_compat".to_string(),
            base_url: String::new(),
            default_model: String::new(),
            requires_api_key: true,
            is_local: false,
            available_models: vec![],
            description: "自定义 OpenAI 兼容 API 端点".to_string(),
        },
    ]
}

/// Find a preset by ID
pub fn find_preset(id: &str) -> Option<ProviderPreset> {
    get_all_presets().into_iter().find(|p| p.id == id)
}
