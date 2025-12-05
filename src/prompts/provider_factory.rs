use std::sync::Arc;

use crate::{
    cache::{CacheLayer, TenantCache, manager::CacheManager},
    llm_providers::gemini::GeminiProvider,
    prompts::{PromptProfile, prompt_provider::PromptProvider},
};

pub struct ProviderFactory<L1: CacheLayer + 'static, L2: CacheLayer + 'static> {
    pub default_provider: String,
    pub default_model: String,
    pub api_key: String,
    pub l1_cache: Arc<L1>,
    pub l2_cache: Arc<L2>,
}
impl<L1: CacheLayer + 'static, L2: CacheLayer + 'static> ProviderFactory<L1, L2> {
    pub fn build_provider(&self, profile: &PromptProfile) -> Arc<PromptProvider<L1, L2>> {
        let model = &profile.config.model;
        let cache_mgr = Arc::new(CacheManager::new(
            Arc::new(TenantCache::new(
                profile.name.clone(),
                self.l1_cache.clone(),
            )),
            Arc::new(TenantCache::new(
                profile.name.clone(),
                self.l2_cache.clone(),
            )),
        ));

        let provider = match profile.config.provider.as_str() {
            "gemini" => {
                let mut provider = GeminiProvider::new(
                    profile.config.api_key.clone(),
                    model.clone(),
                    profile.system_prompt.clone(),
                );
                if let Some(temperature) = profile.config.temperature {
                    provider = provider.with_temperature(temperature);
                }
                if let Some(top_p) = profile.config.top_p {
                    provider = provider.with_top_p(top_p);
                }
                if let Some(max_output_tokens) = profile.config.max_output_tokens {
                    provider = provider.with_max_output_tokens(max_output_tokens);
                }
                provider
            }

            _ => panic!("Unsupported LLM provider: {}", profile.config.provider),
        };

        Arc::new(PromptProvider::new(Arc::new(provider), cache_mgr))
    }
}
