#!/bin/bash
# Скрипт для исправления и перезапуска API сервера

echo "=== Исправление API сервера ==="
echo ""

# Остановить контейнер
echo "1. Остановка контейнера..."
docker stop solana-holder-bot-prod 2>/dev/null || true
docker rm solana-holder-bot-prod 2>/dev/null || true

echo ""

# Проверить docker-compose файл
echo "2. Проверка конфигурации..."
if [ -f "docker-compose.prod.yml" ]; then
    echo "   ✓ docker-compose.prod.yml найден"
    
    # Проверить, что API включен в команде
    if grep -q "\-\-api" docker-compose.prod.yml; then
        echo "   ✓ Флаг --api найден в конфигурации"
    else
        echo "   ✗ Флаг --api НЕ найден в конфигурации!"
        echo "   Добавьте --api в команду запуска"
    fi
else
    echo "   ✗ docker-compose.prod.yml не найден!"
fi

echo ""

# Запустить контейнер
echo "3. Запуск контейнера..."
if [ -f "docker-compose.prod.yml" ]; then
    docker-compose -f docker-compose.prod.yml up -d
else
    echo "   Запуск через docker run..."
    docker run -d \
        --name solana-holder-bot-prod \
        --restart unless-stopped \
        -p 56789:56789 \
        -e MINT_ADDRESS="${MINT_ADDRESS:-9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump}" \
        -e RPC_URL="${RPC_URL:-https://api.mainnet-beta.solana.com}" \
        -e INTERVAL="${INTERVAL:-30}" \
        -e CACHE_TTL="${CACHE_TTL:-30}" \
        solana-holder-bot:latest \
        ./solana-holder-bot \
            "${MINT_ADDRESS:-9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump}" \
            --rpc-url "${RPC_URL:-https://api.mainnet-beta.solana.com}" \
            --interval "${INTERVAL:-30}" \
            --api \
            --api-port 56789 \
            --cache-ttl "${CACHE_TTL:-30}"
fi

echo ""

# Подождать немного
echo "4. Ожидание запуска (5 секунд)..."
sleep 5

echo ""

# Проверить работу
echo "5. Проверка работы API..."
if curl -s http://127.0.0.1:56789/health > /dev/null 2>&1; then
    echo "   ✓ API работает!"
    curl -s http://127.0.0.1:56789/health | jq '.' || curl -s http://127.0.0.1:56789/health
else
    echo "   ✗ API не работает!"
    echo "   Проверьте логи: docker logs solana-holder-bot-prod"
fi

echo ""
echo "=== Готово ==="

