use crate::genconfig::Config;
use crate::models::claude::{ClaudeRequest, Message};
use anyhow::{Context, Result};
use std::collections::HashMap;
use tonic::transport::Channel;
use prost::Message as ProstMessage;

// Import from the generated proto module
use xllm_proto::{HttpRequest, HttpResponse, ProxyRequest, ProxyResponse};
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

/// Call Claude API through gRPC proxy
pub async fn call_claude_via_grpc_proxy(
    claude_config: &crate::genconfig::ClaudeConfig,
    global_config: &Config,
    prompt: &str,
    model_override: Option<crate::models::claude::ClaudeModels>,
    max_tokens_override: Option<u32>,
) -> Result<String> {
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

    // Serialize the request to JSON
    let body = serde_json::to_vec(&request).context("Failed to serialize Claude request")?;

    // Call the gRPC proxy directly instead of going through the conversion layer
    let proxy_url = get_proxy_url(global_config)?
        .ok_or_else(|| anyhow::anyhow!("Proxy URL not configured"))?;

    eprintln!("📡 Connecting to gRPC proxy: {}", proxy_url);

    // Create gRPC client connection
    let channel = Channel::from_shared(proxy_url.clone())
        .context("Invalid proxy URL")?
        .connect()
        .await
        .context("Failed to connect to gRPC proxy")?;

    let mut client = ProxyServiceClient::new(channel);

    // Create the internal HTTP request structure
    let http_request = HttpRequest {
        method: "POST".to_string(),
        url: format!("{}/v1/messages", claude_config.url),
        headers,
        body,
    };

    // Serialize the HTTP request into bytes for obfuscation
    let request_package = http_request.encode_to_vec();

    // Create the obfuscated gRPC request - only proxy URL visible
    let grpc_request = tonic::Request::new(ProxyRequest {
        proxy_url: proxy_url.clone(), // Only this is visible in network traffic
        request_package,              // All sensitive data is binary protobuf
    });

    eprintln!("🔒 Sending obfuscated request via gRPC (Anthropic URL, API keys, and data fully hidden)");

    // Send the request through the new obfuscated gRPC method
    let grpc_response = client
        .forward_obfuscated_request(grpc_request)
        .await
        .context("Failed to send obfuscated gRPC request to proxy")?;

    let proxy_response = grpc_response.into_inner();

    // Deserialize the response package
    let http_response = HttpResponse::decode(&proxy_response.response_package[..])
        .context("Failed to decode response package from proxy")?;

    eprintln!("✅ Successfully received response from gRPC proxy");
    eprintln!("📊 Response status: {}", http_response.status_code);

    // Check if the response was successful
    if http_response.status_code < 200 || http_response.status_code >= 300 {
        let error_text = String::from_utf8_lossy(&http_response.body);
        return Err(anyhow::anyhow!("API request failed with status {}: {}", http_response.status_code, error_text));
    }

    // Parse the response body as JSON
    let claude_response: crate::models::claude::ClaudeResponse = 
        serde_json::from_slice(&http_response.body)
            .context("Failed to parse Claude API response")?;

    // Extract text from the first content block
    if let Some(content_block) = claude_response.content.first() {
        Ok(content_block.text.clone())
    } else {
        Err(anyhow::anyhow!("No content in Claude response"))
    }
}

