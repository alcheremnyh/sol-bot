# PowerShell скрипт для сборки проекта
# Проверяет наличие Rust и собирает проект

Write-Host "Проверка установки Rust..." -ForegroundColor Cyan

# Проверка cargo в стандартных местах
$cargoPaths = @(
    "$env:USERPROFILE\.cargo\bin\cargo.exe",
    "C:\Users\$env:USERNAME\.cargo\bin\cargo.exe",
    "$env:ProgramFiles\Rust stable MSVC 1.xx\bin\cargo.exe"
)

$cargoFound = $false
$cargoPath = $null

foreach ($path in $cargoPaths) {
    if (Test-Path $path) {
        $cargoPath = $path
        $cargoFound = $true
        Write-Host "Найден cargo: $path" -ForegroundColor Green
        break
    }
}

# Проверка в PATH
if (-not $cargoFound) {
    try {
        $cargoVersion = & cargo --version 2>&1
        if ($LASTEXITCODE -eq 0) {
            $cargoFound = $true
            $cargoPath = "cargo"
            Write-Host "Cargo найден в PATH: $cargoVersion" -ForegroundColor Green
        }
    }
    catch {
        # Cargo не найден
    }
}

if (-not $cargoFound) {
    Write-Host "`n❌ Rust/Cargo не установлен!" -ForegroundColor Red
    Write-Host "`nУстановите Rust:" -ForegroundColor Yellow
    Write-Host "1. Скачайте с https://rustup.rs/" -ForegroundColor White
    Write-Host "2. Запустите rustup-init.exe" -ForegroundColor White
    Write-Host "3. Перезапустите терминал" -ForegroundColor White
    Write-Host "`nИли добавьте cargo в PATH вручную" -ForegroundColor Yellow
    exit 1
}

Write-Host "`nНачинаю сборку проекта..." -ForegroundColor Cyan
Write-Host "Это может занять несколько минут при первой сборке..." -ForegroundColor Yellow

if ($cargoPath -eq "cargo") {
    & cargo build --release
}
else {
    & $cargoPath build --release
}

if ($LASTEXITCODE -eq 0) {
    Write-Host "`n✅ Сборка завершена успешно!" -ForegroundColor Green
    Write-Host "`nБинарник находится в: target\release\solana-holder-bot.exe" -ForegroundColor Cyan
    Write-Host "`nЗапуск:" -ForegroundColor Yellow
    Write-Host ".\target\release\solana-holder-bot.exe 9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump --rpc-url https://api.mainnet-beta.solana.com --interval 30" -ForegroundColor White
}
else {
    Write-Host "`n❌ Ошибка при сборке!" -ForegroundColor Red
    exit 1
}

