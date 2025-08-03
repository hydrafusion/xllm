use anyhow::Result;
use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::{Aead, OsRng, AeadCore};
use serde::{Deserialize, Serialize};

// Pre-shared encryption key for obfuscation
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

async fn handle_client(mut stream: TcpStream) -> Result<()> {
    let peer_addr = stream.peer_addr()?;
    println!("🔗 New connection from: {}", peer_addr);

    // Read the incoming request
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await?;

    if buffer.is_empty() {
        println!("❌ Empty request from {}", peer_addr);
        return Ok(());
    }

    // Deserialize the proxy request
    let proxy_request: ProxyRequest = match serde_json::from_slice(&buffer) {
        Ok(req) => req,
        Err(e) => {
            println!("❌ Failed to deserialize request: {}", e);
            return Ok(());
        }
    };

    println!("🔒 Received encrypted request to proxy: {}", proxy_request.proxy_url);

    // Decrypt the request object
    let http_request = match decrypt_request_object(&proxy_request.request_object) {
        Ok(req) => req,
        Err(e) => {
            println!("❌ Failed to decrypt request: {}", e);
            return Ok(());
        }
    };

    println!("🔄 Decrypted request: {} {}", http_request.method, http_request.url);

    // Execute the actual HTTP request
    let http_response = match execute_http_request(http_request).await {
        Ok(resp) => resp,
        Err(e) => {
            println!("❌ HTTP request failed: {}", e);
            return Ok(());
        }
    };

    // Encrypt the response
    let encrypted_response = match encrypt_response_object(&http_response) {
        Ok(encrypted) => encrypted,
        Err(e) => {
            println!("❌ Failed to encrypt response: {}", e);
            return Ok(());
        }
    };

    // Create proxy response
    let proxy_response = ProxyResponse {
        response_object: encrypted_response,
    };

    // Serialize and send response
    let response_data = serde_json::to_vec(&proxy_response)?;
    stream.write_all(&response_data).await?;

    println!("✅ Request completed and encrypted response sent to {}", peer_addr);
    Ok(())
}

fn decrypt_request_object(encrypted_data: &[u8]) -> Result<HttpRequest> {
    if encrypted_data.len() < 12 {
        return Err(anyhow::anyhow!("Invalid encrypted data: too short"));
    }

    let key = Key::<Aes256Gcm>::from_slice(OBFUSCATION_KEY);
    let cipher = Aes256Gcm::new(key);
    
    let nonce_bytes = &encrypted_data[..12];
    let ciphertext = &encrypted_data[12..];
    let nonce = Nonce::from_slice(nonce_bytes);

    let decrypted = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;

    let http_request: HttpRequest = serde_json::from_slice(&decrypted)
        .map_err(|e| anyhow::anyhow!("Failed to deserialize decrypted request: {}", e))?;

    Ok(http_request)
}

fn encrypt_response_object(http_response: &HttpResponse) -> Result<Vec<u8>> {
    let key = Key::<Aes256Gcm>::from_slice(OBFUSCATION_KEY);
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let response_data = serde_json::to_vec(http_response)
        .map_err(|e| anyhow::anyhow!("Failed to serialize response: {}", e))?;

    let encrypted = cipher.encrypt(&nonce, response_data.as_ref())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;

    let mut result = nonce.to_vec();
    result.extend_from_slice(&encrypted);
    Ok(result)
}

async fn execute_http_request(http_request: HttpRequest) -> Result<HttpResponse> {
    let client = reqwest::Client::new();

    let mut req_builder = match http_request.method.to_uppercase().as_str() {
        "GET" => client.get(&http_request.url),
        "POST" => client.post(&http_request.url),
        "PUT" => client.put(&http_request.url),
        "DELETE" => client.delete(&http_request.url),
        "PATCH" => client.patch(&http_request.url),
        "HEAD" => client.head(&http_request.url),
        method => {
            return Err(anyhow::anyhow!("Unsupported HTTP method: {}", method));
        }
    };

    // Add headers
    for (key, value) in &http_request.headers {
        req_builder = req_builder.header(key, value);
    }

    // Add body if present
    if !http_request.body.is_empty() {
        req_builder = req_builder.body(http_request.body);
    }

    // Execute the request
    let response = req_builder.send().await?;
    let status_code = response.status().as_u16();

    // Extract headers with obfuscation (filter out provider-specific headers)
    let mut headers = HashMap::new();
    for (key, value) in response.headers() {
        if let Ok(value_str) = value.to_str() {
            let key_lower = key.as_str().to_lowercase();
            
            // Only include generic headers, exclude provider-specific ones
            if is_generic_header(&key_lower) {
                headers.insert(key.as_str().to_string(), value_str.to_string());
            }
        }
    }

    let body = response.bytes().await?.to_vec();

    println!("✅ HTTP request completed with status: {} (headers obfuscated)", status_code);

    Ok(HttpResponse {
        status_code,
        headers,
        body,
    })
}

/// Helper function to determine if a header should be included in responses
/// This filters out provider-specific headers to maintain obfuscation
fn is_generic_header(header_name: &str) -> bool {
    match header_name {
        // Allow standard HTTP headers
        "content-type" | "content-length" | "content-encoding" => true,
        "cache-control" | "expires" | "etag" | "last-modified" => true,
        "date" | "server" | "connection" | "keep-alive" => true,
        "strict-transport-security" | "x-content-type-options" => true,
        "x-frame-options" | "x-xss-protection" => true,
        
        // Block provider-specific headers that expose the backend service
        header if header.starts_with("anthropic-") => false,
        header if header.starts_with("openai-") => false,
        header if header.starts_with("x-ratelimit") => false,
        header if header.starts_with("x-request-id") => false,
        "request-id" | "cf-ray" | "cf-cache-status" => false,
        "via" | "x-robots-tag" => false,
        
        // Default: allow other headers but log them for monitoring
        _ => {
            println!("🔍 Allowing unknown header: {}", header_name);
            true
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let host = std::env::var("XLLM_PROXY_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("XLLM_PROXY_PORT").unwrap_or_else(|_| "50051".to_string());
    let addr = format!("{}:{}", host, port);

    let listener = TcpListener::bind(&addr).await?;
    
    println!("🚀 Starting xllm-proxy TCP server on {}", addr);
    println!("� Ready to handle encrypted HTTP requests...");
    println!("🌐 Proxy will obfuscate all provider-specific data");

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream).await {
                        println!("❌ Error handling client: {}", e);
                    }
                });
            }
            Err(e) => {
                println!("❌ Failed to accept connection: {}", e);
            }
        }
    }
}
