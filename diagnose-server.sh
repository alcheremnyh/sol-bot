#!/bin/bash
# Скрипт диагностики API на сервере

echo "=== Диагностика Solana Holder Bot API ==="
echo ""

# 1. Проверка контейнера
echo "1. Проверка контейнера Docker..."
if docker ps | grep -q solana-holder-bot-prod; then
    echo "   ✓ Контейнер запущен"
    docker ps | grep solana-holder-bot-prod
else
    echo "   ✗ Контейнер НЕ запущен!"
    echo "   Запустите: docker-compose -f docker-compose.prod.yml up -d"
    exit 1
fi

echo ""

# 2. Проверка логов
echo "2. Последние 20 строк логов контейнера:"
docker logs solana-holder-bot-prod --tail 20

echo ""

# 3. Проверка порта
echo "3. Проверка порта 56789..."
if netstat -tuln | grep -q ":56789"; then
    echo "   ✓ Порт 56789 слушается"
    netstat -tuln | grep ":56789"
else
    echo "   ✗ Порт 56789 НЕ слушается!"
fi

echo ""

# 4. Проверка API изнутри контейнера
echo "4. Проверка API изнутри контейнера..."
if docker exec solana-holder-bot-prod curl -s http://localhost:56789/health > /dev/null 2>&1; then
    echo "   ✓ Health check работает изнутри контейнера"
    docker exec solana-holder-bot-prod curl -s http://localhost:56789/health
else
    echo "   ✗ Health check НЕ работает изнутри контейнера!"
    echo "   Возможно, API сервер не запущен"
fi

echo ""

# 5. Проверка API с хоста
echo "5. Проверка API с хоста (127.0.0.1)..."
if curl -s http://127.0.0.1:56789/health > /dev/null 2>&1; then
    echo "   ✓ Health check работает с хоста"
    curl -s http://127.0.0.1:56789/health | jq '.' || curl -s http://127.0.0.1:56789/health
else
    echo "   ✗ Health check НЕ работает с хоста!"
    echo "   Проверьте проброс порта в docker run: -p 56789:56789"
fi

echo ""

# 6. Проверка процесса внутри контейнера
echo "6. Процессы внутри контейнера:"
docker exec solana-holder-bot-prod ps aux | grep -E "solana-holder-bot|curl" || echo "   Не удалось проверить процессы"

echo ""

# 7. Проверка переменных окружения
echo "7. Переменные окружения контейнера:"
docker exec solana-holder-bot-prod env | grep -E "MINT|RPC|API|CACHE" || echo "   Переменные окружения не найдены"

echo ""
echo "=== Диагностика завершена ==="

