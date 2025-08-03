# xllm-proxy Docker Deployment Guide

## 🚀 Ready-to-Deploy Setup

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
2. ✅ Install protobuf compiler & git  
3. ✅ Clone only xllm-proxy, xllm-proto, and Cargo.toml
4. ✅ Build and run the proxy server
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

## 🎉 Ready for Production!

Your xllm-proxy is now ready to:
- ✅ Accept gRPC requests with protobuf
- ✅ Forward HTTP requests to LLM APIs
- ✅ Return responses as protobuf
- ✅ Auto-restart on failure
- ✅ Scale horizontally if needed

The deployment is clean, minimal, and production-ready! 🚀
