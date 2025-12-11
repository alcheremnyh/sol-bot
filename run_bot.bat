@echo off
echo Запуск бота для мониторинга токена 9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump
echo.
echo Используется публичный RPC: https://api.mainnet-beta.solana.com
echo Интервал опроса: 30 секунд
echo.
echo Для остановки нажмите Ctrl+C
echo.

cargo run --release -- 9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump --rpc-url https://api.mainnet-beta.solana.com --interval 30

pause

