use std::pin::Pin;

use serde::{Deserialize, Serialize};
use tracing::{debug, error, instrument};

pub struct GeminiProvider {
    api_key: String,
    model: String,
    system_instruction: String,
    client: reqwest::Client,
}

impl GeminiProvider {
    pub fn new(api_key: String, model: String, system_instruction: String) -> Self {
        Self {
            api_key,
            model,
            system_instruction,
            client: reqwest::Client::new(),
        }
    }
}

// === Request Structs ===
#[derive(Serialize)]
struct TextPart<'a> {
    text: &'a str,
}

#[derive(Serialize)]
struct SystemInstruction<'a> {
    parts: Vec<TextPart<'a>>,
}

#[derive(Serialize)]
struct Content<'a> {
    parts: Vec<TextPart<'a>>,
}

#[derive(Serialize)]
struct GeminiRequest<'a> {
    system_instruction: SystemInstruction<'a>,
    contents: Vec<Content<'a>>,
}

// === Response Structs ===
#[derive(Deserialize)]
struct CandidateContent {
    parts: Vec<TextPartOwned>,
}

#[derive(Deserialize)]
struct Candidate {
    content: CandidateContent,
    finishReason: Option<String>,
    index: u32,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
    modelVersion: String,
    responseId: String,
}

#[derive(Deserialize)]
struct TextPartOwned {
    text: String,
}

// === LLMProvider Implementation ===
impl super::traits::LLMProvider for GeminiProvider {
    #[instrument(skip(self))]
    fn query<'a>(
        &'a self,
        input: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String, super::traits::LLMError>> + Send + 'a>> {
        Box::pin(async move {
        debug!("Querying Gemini LLM with input: {}", input);

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let request_body = GeminiRequest {
            system_instruction: SystemInstruction {
                parts: vec![TextPart {
                    text: &self.system_instruction,
                }],
            },
            contents: vec![Content {
                parts: vec![TextPart { text: input }],
            }],
        };

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                error!("Error sending request to Gemini LLM: {:?}", e);
                super::traits::LLMError::Other(format!("{:?}", e))
            })?;

        if !response.status().is_success() {
            error!("HTTP error from Gemini LLM: {}", response.status());
            return Err(super::traits::LLMError::Network(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let parsed: GeminiResponse = response.json().await.map_err(|e| {
            error!("Error parsing response from Gemini LLM: {:?}", e);
            super::traits::LLMError::InvalidResponse(format!("{:?}", e))
        })?;

        let result = parsed
            .candidates
            .get(0)
            .and_then(|c| c.content.parts.get(0))
            .map(|p| p.text.clone())
            .ok_or_else(|| {
                error!("No response from Gemini LLM");
                super::traits::LLMError::InvalidResponse("No response from Gemini".to_string())
            })?;

        debug!("Received response from Gemini LLM: {}", result);
        Ok(result)
        })
    }
}
