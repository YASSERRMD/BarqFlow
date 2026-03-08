#!/bin/bash
set -e

echo "=========================================="
echo "🚀 BarqFlow Deployment Script"
echo "=========================================="

echo "1. Stopping existing containers..."
docker-compose -f docker/docker-compose.yml down

echo "2. Rebuilding fresh BarqFlow containers..."
docker-compose -f docker/docker-compose.yml build --no-cache

echo "3. Starting isolated production environment..."
docker-compose -f docker/docker-compose.yml up -d

echo "=========================================="
echo "✅ Deployment Successful!"
echo "📍 Access BarqFlow Engine at: http://localhost:3000"
echo "=========================================="
