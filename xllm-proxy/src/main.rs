use anyhow::Result;
use std::collections::HashMap;
use tonic::{transport::Server, Request, Response, Status};
use xllm_proto::{
    proxy_service_server::{ProxyService, ProxyServiceServer},
    HttpRequest, HttpResponse,
};

#[derive(Debug, Default)]
pub struct ProxyServiceImpl {}

#[tonic::async_trait]
impl ProxyService for ProxyServiceImpl {
    async fn forward_request(
        &self,
        request: Request<HttpRequest>,
    ) -> Result<Response<HttpResponse>, Status> {
        let http_request = request.into_inner();
        
        println!("🔄 Received request to proxy: {} {}", http_request.method, http_request.url);
        
        // Create HTTP client
        let client = reqwest::Client::new();
        
        // Build the HTTP request
        let mut req_builder = match http_request.method.to_uppercase().as_str() {
            "GET" => client.get(&http_request.url),
            "POST" => client.post(&http_request.url),
            "PUT" => client.put(&http_request.url),
            "DELETE" => client.delete(&http_request.url),
            "PATCH" => client.patch(&http_request.url),
            "HEAD" => client.head(&http_request.url),
            method => {
                return Err(Status::invalid_argument(format!("Unsupported HTTP method: {}", method)));
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
        match req_builder.send().await {
            Ok(response) => {
                let status_code = response.status().as_u16() as i32;
                
                // Extract headers
                let mut headers = HashMap::new();
                for (key, value) in response.headers() {
                    if let Ok(value_str) = value.to_str() {
                        headers.insert(key.as_str().to_string(), value_str.to_string());
                    }
                }

                // Extract body
                let body = match response.bytes().await {
                    Ok(bytes) => bytes.to_vec(),
                    Err(e) => {
                        return Err(Status::internal(format!("Failed to read response body: {}", e)));
                    }
                };

                println!("✅ Request completed with status: {}", status_code);

                let http_response = HttpResponse {
                    status_code,
                    headers,
                    body,
                };

                Ok(Response::new(http_response))
            }
            Err(e) => {
                println!("❌ Request failed: {}", e);
                Err(Status::internal(format!("HTTP request failed: {}", e)))
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Configure from environment variables for Docker deployment
    let host = std::env::var("XLLM_PROXY_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("XLLM_PROXY_PORT").unwrap_or_else(|_| "50051".to_string());
    let addr = format!("{}:{}", host, port).parse()?;
    
    let proxy_service = ProxyServiceImpl::default();

    println!("🚀 Starting xllm-proxy gRPC server on {}", addr);
    println!("📡 Ready to proxy HTTP requests via protobuf...");
    println!("🐳 Docker deployment mode: listening on all interfaces");

    Server::builder()
        .add_service(ProxyServiceServer::new(proxy_service))
        .serve(addr)
        .await?;

    Ok(())
}
