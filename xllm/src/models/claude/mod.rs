use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ClaudeModels {
    Opus4,
    Sonnet4,
    Sonnet3_7,
    Haiku3_5,
}

impl ClaudeModels {
    pub fn to_string(&self) -> String {
        match self {
            ClaudeModels::Opus4 => "claude-opus-4-20250514".to_string(),
            ClaudeModels::Sonnet4 => "claude-sonnet-4-20250514".to_string(),
            ClaudeModels::Sonnet3_7 => "claude-3-7-sonnet-latest".to_string(),
            ClaudeModels::Haiku3_5 => "claude-3-5-haiku-latest".to_string(),
        }
    }
}

pub fn parse_model(name: Option<&str>) -> Option<ClaudeModels> {
    match name {
        Some("opus4") => Some(ClaudeModels::Opus4),
        Some("sonnet4") => Some(ClaudeModels::Sonnet4),
        Some("sonnet3") => Some(ClaudeModels::Sonnet3_7),
        Some("haiku3") => Some(ClaudeModels::Haiku3_5),
        Some(invalid) => {
            eprintln!(
                "❌ Invalid model '{}'. Available: opus4, sonnet4, sonnet3, haiku3",
                invalid
            );
            std::process::exit(1);
        }
        None => None,
    }
}

#[derive(Serialize)]
pub struct ClaudeRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<Message>,
}

#[derive(Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct ClaudeResponse {
    pub content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
pub struct ContentBlock {
    pub text: String,
}

pub async fn call_claude_api(
    config: &crate::genconfig::ClaudeConfig,
    prompt: &str,
    model_override: Option<ClaudeModels>,
    max_tokens_override: Option<u32>,
) -> Result<String> {
    let client = Client::new();

    let model = if let Some(model_enum) = model_override {
        model_enum.to_string()
    } else {
        config.model.clone()
    };

    let max_tokens = max_tokens_override.unwrap_or(config.max_tokens);

    let request = ClaudeRequest {
        model,
        max_tokens,
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        }],
    };

    // Prepare headers for the API request
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("x-api-key".to_string(), config.anthropic_api_key.clone());
    headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());

    // Perform direct HTTP request to Claude API
    let response = client
        .post(&format!("{}/v1/messages", config.url))
        .headers(reqwest::header::HeaderMap::from_iter(
            headers.iter().map(|(k, v)| {
                (reqwest::header::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                 reqwest::header::HeaderValue::from_str(v).unwrap())
            })
        ))
        .json(&request)
        .send()
        .await
        .context("Failed to send request to Claude API")?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("API request failed: {}", error_text));
    }

    let claude_response: ClaudeResponse = response
        .json()
        .await
        .context("Failed to parse Claude API response")?;

    // Extract text from the first content block
    if let Some(content_block) = claude_response.content.first() {
        Ok(content_block.text.clone())
    } else {
        Err(anyhow::anyhow!("No content in Claude response"))
    }
}
