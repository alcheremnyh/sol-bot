# PowerShell скрипт для быстрой проверки API

Write-Host "=== Проверка API ===" -ForegroundColor Cyan
Write-Host ""

# Health check
Write-Host "1. Health check:" -ForegroundColor Yellow
try {
    $health = Invoke-RestMethod -Uri "http://127.0.0.1:56789/health" -Method Get
    Write-Host "   Status: $($health.status)" -ForegroundColor Green
    Write-Host "   Service: $($health.service)" -ForegroundColor Green
} catch {
    Write-Host "   Ошибка: $_" -ForegroundColor Red
}
Write-Host ""

# Получить holders
Write-Host "2. Получить holders:" -ForegroundColor Yellow
$mint = "9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump"
try {
    $response = Invoke-RestMethod -Uri "http://127.0.0.1:56789/holders/$mint" -Method Get
    Write-Host "   Mint: $($response.mint)" -ForegroundColor Green
    Write-Host "   Holders: $($response.holders)" -ForegroundColor Green
    Write-Host "   Timestamp: $($response.timestamp)" -ForegroundColor Green
    Write-Host "   Cached: $($response.cached)" -ForegroundColor Green
} catch {
    Write-Host "   Ошибка: $_" -ForegroundColor Red
}
Write-Host ""

# Проверка через nginx (если настроен)
Write-Host "3. Проверка через nginx (если настроен):" -ForegroundColor Yellow
try {
    $nginxHealth = Invoke-RestMethod -Uri "https://sminem.fun/api-holders/health" -Method Get -ErrorAction SilentlyContinue
    Write-Host "   Status: $($nginxHealth.status)" -ForegroundColor Green
} catch {
    Write-Host "   Nginx не настроен или недоступен" -ForegroundColor Gray
}
Write-Host ""

Write-Host "=== Готово ===" -ForegroundColor Cyan

