#!/bin/bash
# Быстрая проверка API

echo "=== Проверка API ==="
echo ""

# Health check
echo "1. Health check:"
curl -s http://127.0.0.1:56789/health | jq '.' || curl -s http://127.0.0.1:56789/health
echo ""

# Получить holders
echo "2. Получить holders:"
curl -s http://127.0.0.1:56789/holders/9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump | jq '.' || curl -s http://127.0.0.1:56789/holders/9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump
echo ""

# Проверка через nginx (если настроен)
echo "3. Проверка через nginx (если настроен):"
curl -s https://sminem.fun/api-holders/health 2>/dev/null | jq '.' || echo "Nginx не настроен или недоступен"
echo ""

echo "=== Готово ==="

