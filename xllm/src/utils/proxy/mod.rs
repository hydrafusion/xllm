use crate::genconfig::Config;
use crate::models::claude::{ClaudeRequest, Message};
use anyhow::{Context, Result};
use std::collections::HashMap;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use aes_gcm::{Aes256Gcm, Key, KeyInit};
use aes_gcm::aead::{Aead, OsRng, AeadCore};
use serde::{Deserialize, Serialize};

// Pre-shared encryption key - must match proxy server
const OBFUSCATION_KEY: &[u8; 32] = b"xllm_secure_proxy_key_2024_v1.0!";

#[derive(Serialize, Deserialize, Debug)]
struct ProxyRequest {
    proxy_url: String,
    request_object: Vec<u8>, // Encrypted HTTP request data
}

#[derive(Serialize, Deserialize, Debug)]
struct HttpRequest {
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
struct HttpResponse {
    status_code: u16,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ProxyResponse {
    response_object: Vec<u8>, // Encrypted HTTP response data
}

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

/// Call Claude API through TCP proxy with encryption
pub async fn call_claude_via_tcp_proxy(
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

    // Create the internal HTTP request structure
    let http_request = HttpRequest {
        method: "POST".to_string(),
        url: format!("{}/v1/messages", claude_config.url),
        headers,
        body,
    };

    // Encrypt the HTTP request
    let encrypted_request = encrypt_request_object(&http_request)?;

    // Get proxy URL and extract host/port
    let proxy_url = get_proxy_url(global_config)?
        .ok_or_else(|| anyhow::anyhow!("Proxy URL not configured"))?;

    let proxy_addr = parse_proxy_url(&proxy_url)?;

    println!("📡 Connecting to TCP proxy: {}", proxy_addr);

    // Create the obfuscated proxy request - only proxy URL visible
    let proxy_request = ProxyRequest {
        proxy_url: proxy_url.clone(), // Only this is visible in network traffic
        request_object: encrypted_request,  // Fully encrypted binary data
    };

    // Connect to proxy server
    let mut stream = TcpStream::connect(&proxy_addr).await
        .context("Failed to connect to TCP proxy")?;

    println!("🔒 Sending encrypted request via TCP (Anthropic URL, API keys, and data fully hidden)");

    // Send the encrypted request
    let request_data = serde_json::to_vec(&proxy_request)
        .context("Failed to serialize proxy request")?;
    
    stream.write_all(&request_data).await
        .context("Failed to send request to proxy")?;
    
    // Signal end of request
    stream.shutdown().await
        .context("Failed to shutdown write stream")?;

    // Read the encrypted response
    let mut response_buffer = Vec::new();
    stream.read_to_end(&mut response_buffer).await
        .context("Failed to read response from proxy")?;

    // Deserialize the proxy response
    let proxy_response: ProxyResponse = serde_json::from_slice(&response_buffer)
        .context("Failed to deserialize proxy response")?;

    // Decrypt the response
    let http_response = decrypt_response_object(&proxy_response.response_object)
        .context("Failed to decrypt response from proxy")?;

    println!("✅ Successfully received and decrypted response from TCP proxy");
    println!("📊 Response status: {}", http_response.status_code);

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

fn encrypt_request_object(http_request: &HttpRequest) -> Result<Vec<u8>> {
    let key = Key::<Aes256Gcm>::from_slice(OBFUSCATION_KEY);
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let request_data = serde_json::to_vec(http_request)
        .map_err(|e| anyhow::anyhow!("Failed to serialize request: {}", e))?;

    let encrypted = cipher.encrypt(&nonce, request_data.as_ref())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;

    let mut result = nonce.to_vec();
    result.extend_from_slice(&encrypted);
    Ok(result)
}

fn decrypt_response_object(encrypted_data: &[u8]) -> Result<HttpResponse> {
    if encrypted_data.len() < 12 {
        return Err(anyhow::anyhow!("Invalid encrypted response: too short"));
    }

    let key = Key::<Aes256Gcm>::from_slice(OBFUSCATION_KEY);
    let cipher = Aes256Gcm::new(key);
    
    let nonce_bytes = &encrypted_data[..12];
    let ciphertext = &encrypted_data[12..];
    let nonce = aes_gcm::Nonce::from_slice(nonce_bytes);

    let decrypted = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;

    let http_response: HttpResponse = serde_json::from_slice(&decrypted)
        .map_err(|e| anyhow::anyhow!("Failed to deserialize decrypted response: {}", e))?;

    Ok(http_response)
}

fn parse_proxy_url(proxy_url: &str) -> Result<String> {
    // Parse URL like "http://learn.hydrafusion.dev:50051" to "learn.hydrafusion.dev:50051"
    let url = url::Url::parse(proxy_url)
        .context("Invalid proxy URL format")?;
    
    let host = url.host_str()
        .ok_or_else(|| anyhow::anyhow!("No host in proxy URL"))?;
    
    let port = url.port()
        .ok_or_else(|| anyhow::anyhow!("No port in proxy URL"))?;
    
    Ok(format!("{}:{}", host, port))
}
