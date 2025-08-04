# xllm-proxy Docker Deployment Guide

## 🚀 Ready-to-Deploy TCP Proxy with AES-256-GCM Encryption

Your xllm-proxy is **production-ready** and can be deployed with a single command!

## 📁 Files Needed for Deployment

Copy these files to your server:

```
xllm-proxy/
├── docker-compose.yml    # ← Only file needed for deployment!
└── test_deployment.sh    # ← Optional: for testing
```

## 🎯 One-Command Deployment

On your server, run:

```bash
docker-compose up -d
```

That's it! The container will:

1. ✅ Pull Rust image
2. ✅ Install git
3. ✅ Clone only xllm-proxy and Cargo.toml
4. ✅ Build and run the encrypted TCP proxy server
5. ✅ Expose port 50051

## 🧪 Testing the Deployment

### Local Testing
```bash
# Run the test script
./test_deployment.sh

# Or manually:
cd xllm-proxy
docker-compose up -d
docker-compose logs -f
```

### Server Testing
```bash
# Check if port is open
nc -z localhost 50051

# View container status
docker-compose ps

# View logs
docker-compose logs -f xllm-proxy
```

## 📊 Management Commands

```bash
# Start proxy
docker-compose up -d

# Stop proxy  
docker-compose down

# Restart proxy
docker-compose restart

# View logs
docker-compose logs -f

# Update to latest code
docker-compose down
docker-compose up -d --force-recreate
```

## 🔧 Configuration

### Environment Variables
The proxy uses these environment variables (set in docker-compose.yml):
- `XLLM_PROXY_HOST=0.0.0.0` - Listen on all interfaces
- `XLLM_PROXY_PORT=50051` - gRPC port
- `RUST_LOG=info` - Log level

### Port Mapping
- **Host**: `50051` → **Container**: `50051`
- Protocol: gRPC/HTTP2

## 🌐 Client Configuration

To use the proxy with xllm, configure your client:

```toml
# config.toml
[global]
proxy = true
proxy_url = "http://your-server:50051"
```

## 🔒 Production Considerations

1. **Firewall**: Open port 50051 on your server
2. **SSL/TLS**: Consider adding nginx for HTTPS termination
3. **Monitoring**: Use `docker-compose logs` for monitoring
4. **Updates**: Recreate container to pull latest code

## 🎉 Ready for Production

Your xllm-proxy is now ready to:

- ✅ Accept encrypted TCP requests with AES-256-GCM
- ✅ Forward HTTP requests to LLM APIs  
- ✅ Return encrypted responses with obfuscated headers
- ✅ Hide all provider-specific data in network traffic
- ✅ Auto-restart on failure
- ✅ Scale horizontally if needed

## 🔒 Security Features

- **Full Network Obfuscation**: Only proxy URL visible in packet captures
- **AES-256-GCM Encryption**: Military-grade encryption for all data
- **Header Filtering**: Strips anthropic-*, openai-*, and other provider headers
- **Pre-shared Key**: Prevents unauthorized access to proxy services
