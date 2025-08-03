use crate::genconfig::Config;
use crate::models::claude::{ClaudeRequest, Message};
use anyhow::{Context, Result};
use reqwest::Client;
use std::collections::HashMap;
use tonic::transport::Channel;

// Import from the generated proto module
use xllm_proto::HttpRequest;
use xllm_proto::HttpResponse;
use xllm_proto::proxy_service_client::ProxyServiceClient;

/// Determines if we should use proxy based on config
pub fn proxy_config_true(config: &Config) -> bool {
    config.global.as_ref().map_or(false, |global| global.proxy)
}

/// Gets the proxy URL from config, returns error if proxy is enabled but URL is missing
pub fn get_proxy_url(config: &Config) -> Result<Option<String>> {
    match &config.global {
        Some(global) if global.proxy => {
            if global.proxy_url.is_empty() {
                return Err(anyhow::anyhow!(
                    "Proxy is enabled but proxy_url is empty in config"
                ));
            }
            Ok(Some(global.proxy_url.clone()))
        }
        _ => Ok(None),
    }
}

/// Main function to handle HTTP requests through gRPC proxy only
/// This function assumes proxy is enabled and sends requests via gRPC
pub async fn make_request(
    _client: &Client,
    config: &Config,
    method: &str,
    url: &str,
    headers: HashMap<String, String>,
    body: Vec<u8>,
) -> Result<reqwest::Response> {
    make_grpc_proxy_request(config, method, url, headers, body).await
}

async fn make_grpc_proxy_request(
    config: &Config,
    method: &str,
    url: &str,
    headers: HashMap<String, String>,
    body: Vec<u8>,
) -> Result<reqwest::Response> {
    let proxy_url =
        get_proxy_url(config)?.ok_or_else(|| anyhow::anyhow!("Proxy URL not configured"))?;

    eprintln!("📡 Connecting to gRPC proxy: {}", proxy_url);

    // Create gRPC client connection
    let channel = Channel::from_shared(proxy_url.clone())
        .context("Invalid proxy URL")?
        .connect()
        .await
        .context("Failed to connect to gRPC proxy")?;

    let mut client = ProxyServiceClient::new(channel);

    // Create the gRPC request
    let grpc_request = tonic::Request::new(HttpRequest {
        method: method.to_string(),
        url: url.to_string(),
        headers,
        body,
    });

    eprintln!("🔒 Sending request via gRPC (URL and data are in protobuf)");

    // Send the request through gRPC
    let grpc_response = client
        .forward_request(grpc_request)
        .await
        .context("Failed to send gRPC request to proxy")?;

    let http_response = grpc_response.into_inner();

    eprintln!("✅ Successfully received response from gRPC proxy");
    eprintln!("📊 Response status: {}", http_response.status_code);

    // Convert gRPC response back to reqwest::Response
    convert_grpc_response_to_reqwest_response(http_response).await
}

/// Convert gRPC HttpResponse back to reqwest::Response
async fn convert_grpc_response_to_reqwest_response(
    grpc_response: HttpResponse,
) -> Result<reqwest::Response> {
    // For now, we'll use a workaround since reqwest::Response doesn't have a public constructor
    // In a real implementation, you might want to return a custom response type or use a different approach

    let status_code = grpc_response.status_code;

    // This is a limitation - we can't easily create a reqwest::Response from scratch
    // For now, return an error with the response data
    // TODO: Consider using a different response type or HTTP client that allows response construction

    Err(anyhow::anyhow!(
        "gRPC Response received successfully but conversion to reqwest::Response needs improvement. Status: {}, Body: {}",
        status_code,
        String::from_utf8_lossy(
            &grpc_response.body[..std::cmp::min(100, grpc_response.body.len())]
        )
    ))
}

/// Helper function specifically for API requests with JSON payloads
/// This is a convenience wrapper for the common case of JSON API calls
pub async fn make_api_request<T: serde::Serialize>(
    client: &Client,
    config: &Config,
    method: &str,
    url: &str,
    mut headers: HashMap<String, String>,
    json_payload: Option<&T>,
) -> Result<reqwest::Response> {
    // Ensure Content-Type is set for JSON requests
    if json_payload.is_some() {
        headers.insert("Content-Type".to_string(), "application/json".to_string());
    }

    // Serialize JSON payload if provided
    let body = if let Some(payload) = json_payload {
        serde_json::to_vec(payload).context("Failed to serialize JSON payload")?
    } else {
        Vec::new()
    };

    make_request(client, config, method, url, headers, body).await
}

/// Call Claude API through gRPC proxy
pub async fn call_claude_via_grpc_proxy(
    claude_config: &crate::genconfig::ClaudeConfig,
    global_config: &Config,
    prompt: &str,
    model_override: Option<crate::models::claude::ClaudeModels>,
    max_tokens_override: Option<u32>,
) -> Result<String> {
    let client = Client::new();

    let model = if let Some(model_enum) = model_override {
        model_enum.to_string()
    } else {
        claude_config.model.clone()
    };

    let max_tokens = max_tokens_override.unwrap_or(claude_config.max_tokens);

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
    headers.insert("x-api-key".to_string(), claude_config.anthropic_api_key.clone());
    headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());

    // Use the proxy module to handle the request through gRPC
    let response = make_api_request(
        &client,
        global_config,
        "POST",
        &format!("{}/v1/messages", claude_config.url),
        headers,
        Some(&request),
    )
    .await
    .context("Failed to send request to Claude API via proxy")?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("API request failed: {}", error_text));
    }

    let claude_response: crate::models::claude::ClaudeResponse = response
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

