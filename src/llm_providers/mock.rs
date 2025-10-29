pub struct MockLLMProvider;

impl MockLLMProvider {
    pub fn new() -> Self {
        Self {}
    }
}

impl super::traits::LLMProvider for MockLLMProvider {
    async fn query(&self, input: &str) -> Result<String, super::traits::LLMError> {
        Ok(format!("Mock response for input: {}", input))
    }
}
