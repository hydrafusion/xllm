# xllm-proxy

A TCP reverse proxy server that accepts encrypted requests and forwards them as HTTP requests to APIs.

## Overview

The `xllm-proxy` receives TCP requests with HTTP details encrypted using AES-256-GCM, performs the actual HTTP request to the target API, and returns the encrypted response back. This provides:

- **Full Obfuscation**: Only proxy URL visible in network traffic
- **AES-256-GCM Encryption**: Military-grade encryption for all request/response data
- **Header Filtering**: Strips provider-specific headers (anthropic-*, openai-*, etc.)
- **Performance**: Simple TCP protocol with efficient binary serialization
- **Security**: Pre-shared key encryption prevents man-in-the-middle attacks

## Architecture

```
┌─────────┐    TCP + AES-256-GCM    ┌─────────────┐    HTTP/JSON    ┌─────────────┐
│  xllm   │ ─────────────────────► │ xllm-proxy  │ ──────────────► │  LLM API    │
│(client) │                        │ (TCP server)│                 │ (Claude/etc)│
└─────────┘                        └─────────────┘                 └─────────────┘
```

## What Gets Obfuscated

### ✅ Completely Hidden in Network Traffic
- **API Endpoints** (`api.anthropic.com`, `api.openai.com`)
- **API Keys** (`x-api-key`, `Authorization` headers)
- **Request/Response Bodies** (prompts, responses, model parameters)
- **Provider Headers** (`anthropic-*`, `openai-*`, rate limits, etc.)
- **Request Metadata** (user agents, versions, etc.)

### 👁️ Only Visible in Network
- **Proxy URL** (`your-proxy-server:50051`)
- **Encrypted Binary Data** (meaningless without the pre-shared key)

## Running the Proxy

### Start the server

```bash
# From workspace root
cargo run -p xllm-proxy

# Or from xllm-proxy directory  
cd xllm-proxy
cargo run
```

The server will start on `0.0.0.0:50051`

### Test with xllm client

```bash
# Configure your client to use the proxy
# In config.toml:
[global]
proxy = true
proxy_url = "http://your-proxy-server:50051"

# Then run xllm normally
cargo run -p xllm -- -m haiku3 "Hello world"
```

## Protocol Definition

The proxy uses the following encrypted message structure:

```rust
// Only this struct is visible in network traffic
ProxyRequest {
    proxy_url: String,           // The proxy endpoint URL (visible)
    request_object: Vec<u8>,     // AES-256-GCM encrypted HTTP request
}

// Internal structure (encrypted):
HttpRequest {
    method: String,              // HTTP method (POST, GET, etc.)
    url: String,                 // Target API URL (hidden)
    headers: HashMap<String, String>,  // API keys, auth headers (hidden)
    body: Vec<u8>,               // Request payload (hidden)
}
```

## Usage with xllm

When you enable proxy mode in your xllm config:

```toml
[global]
proxy = true
proxy_url = "http://your-proxy-server:50051"
```

All HTTP requests to LLM APIs will be routed through the encrypted proxy:

1. `xllm` encrypts HTTP request details using AES-256-GCM
2. Sends encrypted TCP request to `xllm-proxy`  
3. `xllm-proxy` decrypts and makes actual HTTP request to API
4. API response is encrypted and sent back to client
5. `xllm` decrypts and displays the response

## Features

- ✅ **Full Encryption**: AES-256-GCM with pre-shared key
- ✅ **Header Obfuscation**: Strips `anthropic-*`, `openai-*`, and other provider headers
- ✅ **All HTTP Methods**: GET, POST, PUT, DELETE, PATCH, HEAD
- ✅ **Binary Support**: Handles both text and binary payloads
- ✅ **Error Handling**: Proper error propagation and logging
- ✅ **Network Obfuscation**: Only proxy URL visible in network traffic

## Development

The proxy is built with:

- **Tokio**: Async TCP server framework
- **AES-GCM**: Military-grade encryption (aes-gcm crate)  
- **Reqwest**: HTTP client for making API calls
- **Serde**: JSON serialization for internal structures

## Security Considerations

- The proxy runs on all interfaces by default (`0.0.0.0:50051`) for Docker deployment
- **AES-256-GCM encryption** protects all request/response data
- **Pre-shared key** prevents unauthorized access (change the key in production!)
- **Header filtering** removes provider-specific information
- Consider TLS termination at load balancer level for additional security
- Validate and sanitize forwarded URLs in production environments
