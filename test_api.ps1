# PowerShell скрипт для тестирования API

$apiUrl = "http://localhost:56789"
$mint = "9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump"

Write-Host "=== Тестирование Solana Holder Bot API ===" -ForegroundColor Cyan
Write-Host ""

# Проверка здоровья
Write-Host "1. Проверка здоровья API..." -ForegroundColor Yellow
try {
    $health = Invoke-RestMethod -Uri "$apiUrl/health" -Method Get
    Write-Host "   Status: $($health.status)" -ForegroundColor Green
    Write-Host "   Service: $($health.service)" -ForegroundColor Green
}
catch {
    Write-Host "   Ошибка: $_" -ForegroundColor Red
    exit 1
}

Write-Host ""

# Получение количества держателей
Write-Host "2. Получение количества держателей для $mint..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "$apiUrl/holders/$mint" -Method Get
    Write-Host "   Mint: $($response.mint)" -ForegroundColor Green
    Write-Host "   Holders: $($response.holders)" -ForegroundColor Green
    Write-Host "   Timestamp: $($response.timestamp)" -ForegroundColor Green
    Write-Host "   Cached: $($response.cached)" -ForegroundColor Green
}
catch {
    Write-Host "   Ошибка: $_" -ForegroundColor Red
    if ($_.Exception.Response.StatusCode -eq 400) {
        Write-Host "   Неверный mint адрес!" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "=== Тест завершен ===" -ForegroundColor Cyan

