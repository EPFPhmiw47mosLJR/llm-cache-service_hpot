use serde::Deserialize;

pub mod loader;
pub mod provider_factory;
pub mod prompt_provider;

pub struct PromptProfile {
    pub name: String,
    pub system_prompt: String,
    pub config: PromptConfig,
}

#[derive(Debug, Deserialize)]
pub struct PromptConfig {
    pub model: String,
    pub provider: String,
    pub api_key: String,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub max_output_tokens: Option<u32>,
    #[serde(default)]
    pub cache_ttl: Option<u64>,
}