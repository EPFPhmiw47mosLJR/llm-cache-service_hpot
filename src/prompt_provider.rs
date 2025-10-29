use std::sync::Arc;

use crate::{
    cache::{CacheError, CacheLayer, manager::CacheManager},
    llm_providers::{LLMProvider, traits::LLMError},
};

#[derive(thiserror::Error, Debug)]
pub enum PromptProviderError {
    #[error("CacheError: {0}")]
    CacheError(#[from] CacheError),

    #[error("LLMError: {0}")]
    LLMError(#[from] LLMError),
}

pub struct PromptProvider<P: LLMProvider, L1: CacheLayer, L2: CacheLayer> {
    llm_provider: Arc<P>,
    cache_manager: Arc<CacheManager<L1, L2>>,
}

impl<P: LLMProvider, L1: CacheLayer, L2: CacheLayer> PromptProvider<P, L1, L2> {
    pub fn new(llm_provider: Arc<P>, cache_manager: Arc<CacheManager<L1, L2>>) -> Self {
        Self {
            llm_provider,
            cache_manager,
        }
    }
}

impl<P: LLMProvider, L1: CacheLayer, L2: CacheLayer> PromptProvider<P, L1, L2> {
    pub async fn get_response(&self, prompt: &str) -> Result<String, PromptProviderError> {
        if let Some(cached_response) = self.cache_manager.get(prompt).await? {
            return Ok(cached_response);
        }

        let response = self.llm_provider.query(prompt).await?;

        self.cache_manager.set(prompt, &response).await?;

        Ok(response)
    }
}
