#![allow(dead_code, unused_variables)] // TODO: FIX THIS!

use std::pin::Pin;

use serde::{Deserialize, Serialize};
use tracing::{debug, error, instrument};

pub struct GeminiProvider {
    api_key: String,
    model: String,
    system_instruction: String,
    client: reqwest::Client,
    temperature: Option<f32>,
    max_output_tokens: Option<u32>,
    top_p: Option<f32>,
    top_k: Option<u32>,
}

impl GeminiProvider {
    pub fn new(api_key: String, model: String, system_instruction: String) -> Self {
        Self {
            api_key,
            model,
            system_instruction,
            client: reqwest::Client::new(),
            temperature: None,
            max_output_tokens: None,
            top_p: None,
            top_k: None,
        }
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn with_max_output_tokens(mut self, max_output_tokens: u32) -> Self {
        self.max_output_tokens = Some(max_output_tokens);
        self
    }

    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    pub fn with_top_k(mut self, top_k: u32) -> Self {
        self.top_k = Some(top_k);
        self
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
#[serde(rename_all = "camelCase")]
struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<u32>,
}

#[derive(Serialize)]
struct Content<'a> {
    parts: Vec<TextPart<'a>>,
}

#[derive(Serialize)]
struct GeminiRequest<'a> {
    system_instruction: SystemInstruction<'a>,
    contents: Vec<Content<'a>>,
    generation_config: GenerationConfig,
}

// === Response Structs ===
#[derive(Deserialize)]
struct CandidateContent {
    parts: Vec<TextPartOwned>,
}

#[derive(Deserialize)]
struct Candidate {
    content: CandidateContent,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
    index: u32,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
    #[serde(rename = "modelVersion")]
    model_version: String,
    #[serde(rename = "responseId")]
    response_id: String,
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
                generation_config: GenerationConfig {
                    temperature: self.temperature,
                    max_output_tokens: self.max_output_tokens,
                    top_p: self.top_p,
                    top_k: self.top_k,
                },
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
