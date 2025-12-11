# PowerShell скрипт для тестирования API через nginx на sminem.fun

$domain = "sminem.fun"
$mint = "9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump"

Write-Host "=== Тестирование API через nginx (sminem.fun) ===" -ForegroundColor Cyan
Write-Host ""

# Проверка здоровья через nginx
Write-Host "1. Проверка здоровья API через nginx (/api-sol/)..." -ForegroundColor Yellow
try {
    $health = Invoke-RestMethod -Uri "https://$domain/api-sol/health" -Method Get
    Write-Host "   Status: $($health.status)" -ForegroundColor Green
    Write-Host "   Service: $($health.service)" -ForegroundColor Green
} catch {
    Write-Host "   Ошибка: $_" -ForegroundColor Red
    Write-Host "   Попробуйте: http://$domain/api-sol/health" -ForegroundColor Yellow
}

Write-Host ""

# Получение количества держателей через nginx
Write-Host "2. Получение количества держателей для $mint..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "https://$domain/api-sol/holders/$mint" -Method Get
    Write-Host "   Mint: $($response.mint)" -ForegroundColor Green
    Write-Host "   Holders: $($response.holders)" -ForegroundColor Green
    Write-Host "   Timestamp: $($response.timestamp)" -ForegroundColor Green
    Write-Host "   Cached: $($response.cached)" -ForegroundColor Green
} catch {
    Write-Host "   Ошибка: $_" -ForegroundColor Red
    if ($_.Exception.Response.StatusCode -eq 400) {
        Write-Host "   Неверный mint адрес!" -ForegroundColor Red
    }
    Write-Host "   Попробуйте: http://$domain/api-sol/holders/$mint" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "=== Тест завершен ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "Примеры запросов:" -ForegroundColor Cyan
Write-Host "  Health: https://$domain/api-sol/health" -ForegroundColor White
Write-Host "  Holders: https://$domain/api-sol/holders/$mint" -ForegroundColor White

