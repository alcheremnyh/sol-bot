@echo off
echo ========================================
echo Solana Token Holder Monitoring Bot
echo ========================================
echo.
echo Token: 9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump
echo RPC: https://api.mainnet-beta.solana.com
echo Interval: 30 seconds
echo.
echo Press Ctrl+C to stop and view metrics
echo ========================================
echo.

.\target\release\solana-holder-bot.exe 5AgiTyu3StT3PQYWAkHkg24ugW3TPqYRmQwhVu2fpump --rpc-url https://api.mainnet-beta.solana.com --interval 30

pause

