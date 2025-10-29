pub struct OpenAIProvider {
    api_key: String,
    model: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self { api_key, model }
    }
}

impl super::traits::LLMProvider for OpenAIProvider {
    async fn query(&self, input: &str) -> Result<String, super::traits::LLMError> {
        unimplemented!()
    }
}
