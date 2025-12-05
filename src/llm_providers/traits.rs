use std::pin::Pin;

use thiserror::Error;

pub trait LLMProvider: Send + Sync {
    fn query<'a>(
        &'a self,
        input: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String, LLMError>> + Send + 'a>>;
}

#[derive(Error, Debug)]
pub enum LLMError {
    #[error("LLMError: {0}")]
    Network(String),

    #[error("LLMError: {0}")]
    InvalidResponse(String),

    #[error("LLMError: {0}")]
    Other(String),
}
