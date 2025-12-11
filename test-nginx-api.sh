#!/bin/bash
# Bash скрипт для тестирования API через nginx на sminem.fun

DOMAIN="sminem.fun"
MINT="9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump"

echo "=== Тестирование API через nginx (sminem.fun) ==="
echo ""

# Проверка здоровья через nginx
echo "1. Проверка здоровья API через nginx..."
curl -s "https://$DOMAIN/api-holders-health/" | jq '.' || echo "Ошибка: API недоступен"
echo ""

# Получение количества держателей через nginx
echo "2. Получение количества держателей для $MINT..."
curl -s "https://$DOMAIN/api-holders/holders/$MINT" | jq '.' || echo "Ошибка: не удалось получить данные"
echo ""

echo "=== Тест завершен ==="
echo ""
echo "Примеры запросов:"
echo "  Health: https://$DOMAIN/api-holders-health/"
echo "  Holders: https://$DOMAIN/api-holders/holders/$MINT"

