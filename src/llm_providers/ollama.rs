#![allow(dead_code, unused_variables)] // TODO: FIX THIS!

use std::pin::Pin;

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
    fn query<'a>(
        &'a self,
        input: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String, super::traits::LLMError>> + Send + 'a>> {
        unimplemented!()
    }
}
