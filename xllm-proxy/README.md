# xllm-proxy

A gRPC reverse proxy server that accepts protobuf requests and forwards them as HTTP requests to APIs.

## Overview

The `xllm-proxy` receives gRPC requests with HTTP details encoded in protobuf, performs the actual HTTP request to the target API, and returns the response back as protobuf. This provides:

- **Encryption**: gRPC traffic is more secure than plain HTTP
- **Compression**: Protobuf is more efficient than JSON over the wire
- **Type Safety**: Strongly typed message definitions
- **Performance**: gRPC streaming and multiplexing

## Architecture

```
┌─────────┐    gRPC/Protobuf    ┌─────────────┐    HTTP/JSON    ┌─────────────┐
│  xllm   │ ──────────────────► │ xllm-proxy  │ ──────────────► │  LLM API    │
│ (client)│                     │  (server)   │                 │ (Claude/etc)│
└─────────┘                     └─────────────┘                 └─────────────┘
```

## Running the Proxy

### Start the server:
```bash
# From workspace root
cargo run -p xllm-proxy

# Or from xllm-proxy directory  
cd xllm-proxy
cargo run
```

The server will start on `127.0.0.1:50051`

### Test the proxy:
```bash
# Run the test example
cargo run -p xllm-proxy --example test_proxy
```

## Protocol Definition

The proxy uses the following protobuf messages:

```protobuf
service ProxyService {
  rpc ForwardRequest(HttpRequest) returns (HttpResponse);
}

message HttpRequest {
  string method = 1;           // HTTP method (GET, POST, etc.)
  string url = 2;              // Target URL to forward to
  map<string, string> headers = 3;  // HTTP headers
  bytes body = 4;              // Request body
}

message HttpResponse {
  int32 status_code = 1;       // HTTP status code
  map<string, string> headers = 2;  // Response headers
  bytes body = 3;              // Response body
}
```

## Usage with xllm

When you enable proxy mode in your xllm config:

```toml
[global]
proxy = true
proxy_url = "http://127.0.0.1:50051"
```

All HTTP requests to LLM APIs will be routed through the proxy:

1. `xllm` serializes HTTP request details to protobuf
2. Sends gRPC request to `xllm-proxy`  
3. `xllm-proxy` deserializes and makes actual HTTP request
4. API response is serialized back to protobuf
5. `xllm` receives the gRPC response

## Features

- ✅ **All HTTP Methods**: GET, POST, PUT, DELETE, PATCH, HEAD
- ✅ **Header Forwarding**: All request headers are preserved
- ✅ **Body Support**: Binary and text request/response bodies
- ✅ **Error Handling**: Proper gRPC status codes for failures
- ✅ **Logging**: Request and response logging for debugging

## Development

The proxy is built with:
- **Tonic**: gRPC framework for Rust
- **Reqwest**: HTTP client for making API calls  
- **Tokio**: Async runtime
- **Protobuf**: Message serialization

## Security Considerations

- The proxy runs locally by default (`127.0.0.1:50051`)
- Consider TLS for production deployments
- Validate and sanitize forwarded URLs in production
- Implement authentication/authorization as needed
