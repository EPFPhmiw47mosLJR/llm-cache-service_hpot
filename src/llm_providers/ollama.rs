pub struct OllamaProvider {
    api_key: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self { api_key, model }
    }
}

impl super::traits::LLMProvider for OllamaProvider {
    async fn query(&self, input: &str) -> Result<String, super::traits::LLMError> {
        unimplemented!()
    }
}