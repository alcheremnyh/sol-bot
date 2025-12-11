#!/bin/bash

# Production deployment script for Solana Holder Bot
# Usage: ./deploy.sh [MINT_ADDRESS] [RPC_URL]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
MINT_ADDRESS=${1:-"9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump"}
RPC_URL=${2:-"https://api.mainnet-beta.solana.com"}
IMAGE_NAME="solana-holder-bot"
CONTAINER_NAME="solana-holder-bot-prod"
API_PORT=56789

echo -e "${GREEN}=== Solana Holder Bot Production Deployment ===${NC}"
echo ""

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}Error: Docker is not running!${NC}"
    exit 1
fi

# Check if docker-compose is available
if ! command -v docker-compose &> /dev/null; then
    echo -e "${RED}Error: docker-compose is not installed!${NC}"
    exit 1
fi

echo -e "${YELLOW}Configuration:${NC}"
echo "  Mint Address: $MINT_ADDRESS"
echo "  RPC URL: $RPC_URL"
echo "  API Port: $API_PORT"
echo "  Image: $IMAGE_NAME:latest"
echo ""

# Export environment variables
export MINT_ADDRESS
export RPC_URL
export INTERVAL=30
export CACHE_TTL=30
export MAX_RETRIES=3
export TIMEOUT=30

# Stop existing container if running
if docker ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo -e "${YELLOW}Stopping existing container...${NC}"
    docker-compose -f docker-compose.prod.yml down
fi

# Build image
echo -e "${YELLOW}Building Docker image...${NC}"
docker-compose -f docker-compose.prod.yml build --no-cache

# Start container
echo -e "${YELLOW}Starting container...${NC}"
docker-compose -f docker-compose.prod.yml up -d

# Wait for health check
echo -e "${YELLOW}Waiting for service to be healthy...${NC}"
sleep 5

# Check if container is running
if docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo -e "${GREEN}✓ Container is running${NC}"
else
    echo -e "${RED}✗ Container failed to start${NC}"
    docker-compose -f docker-compose.prod.yml logs
    exit 1
fi

# Test API endpoint
echo -e "${YELLOW}Testing API endpoint...${NC}"
sleep 3

if curl -f -s http://localhost:${API_PORT}/health > /dev/null; then
    echo -e "${GREEN}✓ API is responding${NC}"
else
    echo -e "${RED}✗ API is not responding${NC}"
    docker-compose -f docker-compose.prod.yml logs
    exit 1
fi

echo ""
echo -e "${GREEN}=== Deployment Successful ===${NC}"
echo ""
echo "API Endpoints:"
echo "  Health: http://localhost:${API_PORT}/health"
echo "  Holders: http://localhost:${API_PORT}/holders/${MINT_ADDRESS}"
echo ""
echo "Useful commands:"
echo "  View logs: docker-compose -f docker-compose.prod.yml logs -f"
echo "  Stop: docker-compose -f docker-compose.prod.yml down"
echo "  Restart: docker-compose -f docker-compose.prod.yml restart"
echo ""

