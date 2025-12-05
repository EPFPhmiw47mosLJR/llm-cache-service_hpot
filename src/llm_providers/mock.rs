use std::pin::Pin;

pub struct MockLLMProvider;

impl MockLLMProvider {
    pub fn new() -> Self {
        Self {}
    }
}

impl super::traits::LLMProvider for MockLLMProvider {
    fn query<'a>(
        &'a self,
        input: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String, super::traits::LLMError>> + Send + 'a>> {
        Box::pin(async move { Ok(format!("Mock response for input: {}", input)) })
    }
}
