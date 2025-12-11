# PowerShell deployment script for Solana Holder Bot
# Usage: .\deploy.ps1 [MINT_ADDRESS] [RPC_URL]

param(
    [string]$MintAddress = "9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump",
    [string]$RpcUrl = "https://api.mainnet-beta.solana.com"
)

$ErrorActionPreference = "Stop"

# Configuration
$ContainerName = "solana-holder-bot-prod"
$ApiPort = 56789

Write-Host "=== Solana Holder Bot Production Deployment ===" -ForegroundColor Green
Write-Host ""

# Check if Docker is running
try {
    docker info | Out-Null
} catch {
    Write-Host "Error: Docker is not running!" -ForegroundColor Red
    exit 1
}

# Check if docker-compose is available
if (-not (Get-Command docker-compose -ErrorAction SilentlyContinue)) {
    Write-Host "Error: docker-compose is not installed!" -ForegroundColor Red
    exit 1
}

Write-Host "Configuration:" -ForegroundColor Yellow
Write-Host "  Mint Address: $MintAddress"
Write-Host "  RPC URL: $RpcUrl"
Write-Host "  API Port: $ApiPort"
Write-Host ""

# Set environment variables
$env:MINT_ADDRESS = $MintAddress
$env:RPC_URL = $RpcUrl
$env:INTERVAL = "30"
$env:CACHE_TTL = "30"
$env:MAX_RETRIES = "3"
$env:TIMEOUT = "30"

# Stop existing container if running
$existing = docker ps -a --format '{{.Names}}' | Select-String -Pattern "^${ContainerName}$"
if ($existing) {
    Write-Host "Stopping existing container..." -ForegroundColor Yellow
    docker-compose -f docker-compose.prod.yml down
}

# Build image
Write-Host "Building Docker image..." -ForegroundColor Yellow
docker-compose -f docker-compose.prod.yml build --no-cache

if ($LASTEXITCODE -ne 0) {
    Write-Host "Error: Build failed!" -ForegroundColor Red
    exit 1
}

# Start container
Write-Host "Starting container..." -ForegroundColor Yellow
docker-compose -f docker-compose.prod.yml up -d

if ($LASTEXITCODE -ne 0) {
    Write-Host "Error: Failed to start container!" -ForegroundColor Red
    docker-compose -f docker-compose.prod.yml logs
    exit 1
}

# Wait for health check
Write-Host "Waiting for service to be healthy..." -ForegroundColor Yellow
Start-Sleep -Seconds 5

# Check if container is running
$running = docker ps --format '{{.Names}}' | Select-String -Pattern "^${ContainerName}$"
if ($running) {
    Write-Host "✓ Container is running" -ForegroundColor Green
} else {
    Write-Host "✗ Container failed to start" -ForegroundColor Red
    docker-compose -f docker-compose.prod.yml logs
    exit 1
}

# Test API endpoint
Write-Host "Testing API endpoint..." -ForegroundColor Yellow
Start-Sleep -Seconds 3

try {
    $response = Invoke-WebRequest -Uri "http://localhost:${ApiPort}/health" -TimeoutSec 5 -UseBasicParsing
    if ($response.StatusCode -eq 200) {
        Write-Host "✓ API is responding" -ForegroundColor Green
    } else {
        throw "API returned status code $($response.StatusCode)"
    }
} catch {
    Write-Host "✗ API is not responding" -ForegroundColor Red
    docker-compose -f docker-compose.prod.yml logs
    exit 1
}

Write-Host ""
Write-Host "=== Deployment Successful ===" -ForegroundColor Green
Write-Host ""
Write-Host "API Endpoints:"
Write-Host "  Health: http://localhost:${ApiPort}/health"
Write-Host "  Holders: http://localhost:${ApiPort}/holders/${MintAddress}"
Write-Host ""
Write-Host "Useful commands:"
Write-Host "  View logs: docker-compose -f docker-compose.prod.yml logs -f"
Write-Host "  Stop: docker-compose -f docker-compose.prod.yml down"
Write-Host "  Restart: docker-compose -f docker-compose.prod.yml restart"
Write-Host ""

