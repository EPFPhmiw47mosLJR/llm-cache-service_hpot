use serde::{Deserialize, Serialize};

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
    async fn query(&self, input: &str) -> Result<String, super::traits::LLMError> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            self.model
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
            .header("x-goog-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| super::traits::LLMError::Other(format!("{:?}", e)))?;

        if !response.status().is_success() {
            return Err(super::traits::LLMError::Network(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let parsed: GeminiResponse = response
            .json()
            .await
            .map_err(|e| super::traits::LLMError::InvalidResponse(format!("{:?}", e)))?;

        let result = parsed
            .candidates
            .get(0)
            .and_then(|c| c.content.parts.get(0))
            .map(|p| p.text.clone())
            .ok_or_else(|| {
                super::traits::LLMError::InvalidResponse("No response from Gemini".to_string())
            })?;

        Ok(result)
    }
}
