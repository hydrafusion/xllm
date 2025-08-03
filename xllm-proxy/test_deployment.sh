#!/bin/bash

# Test script for xllm-proxy Docker deployment

set -e

echo "🧪 Testing xllm-proxy Docker deployment..."

# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker and try again."
    exit 1
fi

# Navigate to xllm-proxy directory
cd /home/bourbon/HydraFusion/xllm/xllm-proxy

echo "📦 Starting xllm-proxy container..."
docker-compose up -d

echo "⏳ Waiting for proxy to be ready (this may take a few minutes for first run)..."
sleep 60

echo "🔍 Checking container status..."
docker-compose ps

echo "📋 Container logs:"
docker-compose logs --tail=20

echo "🧪 Testing proxy connectivity..."
# Simple test using curl to check if the port is open
if nc -z localhost 50051; then
    echo "✅ Port 50051 is open - proxy seems to be running!"
else
    echo "❌ Port 50051 is not accessible"
fi

echo ""
echo "🎯 Deployment Summary:"
echo "- Container: xllm-proxy"
echo "- Port: 50051"
echo "- Protocol: gRPC"
echo ""
echo "Commands:"
echo "  View logs:    docker-compose logs -f"
echo "  Stop proxy:   docker-compose down"
echo "  Restart:      docker-compose restart"
