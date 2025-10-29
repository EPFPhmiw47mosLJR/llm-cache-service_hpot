use thiserror::Error;

pub trait LLMProvider: Send + Sync {
    fn query(
        &self,
        input: &str,
    ) -> impl std::future::Future<Output = Result<String, LLMError>> + Send;
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
