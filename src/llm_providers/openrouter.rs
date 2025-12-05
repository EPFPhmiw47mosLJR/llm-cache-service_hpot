use std::pin::Pin;

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
    fn query<'a>(
        &'a self,
        input: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String, super::traits::LLMError>> + Send + 'a>> {
        unimplemented!()
    }
}
