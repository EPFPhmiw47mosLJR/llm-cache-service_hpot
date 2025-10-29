pub struct OpenRouterProvider {
    api_key: String,
    model: String,
}

impl OpenRouterProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self { api_key, model }
    }
}

impl super::traits::LLMProvider for OpenRouterProvider {
    async fn query(&self, input: &str) -> Result<String, super::traits::LLMError> {
        unimplemented!()
    }
}
