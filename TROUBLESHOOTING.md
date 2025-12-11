# Решение проблем с API

## Ошибка: "No such file or directory"

Эта ошибка обычно означает, что:

1. **Контейнер не запущен**
2. **API сервер не запущен внутри контейнера**
3. **Проблема с пробросом порта**

## Быстрая диагностика

### 1. Проверить контейнер

```bash
# Проверить, запущен ли контейнер
docker ps | grep solana-holder-bot-prod

# Если не запущен, запустить
docker-compose -f docker-compose.prod.yml up -d
```

### 2. Проверить логи

```bash
# Посмотреть последние логи
docker logs solana-holder-bot-prod --tail 50

# Искать сообщения об API
docker logs solana-holder-bot-prod | grep -i "api"
```

**Ожидаемые сообщения в логах:**
```
🚀 API server enabled on port 56789 (cache refresh: 30s)
API server started on http://0.0.0.0:56789
Endpoints:
  GET /holders/:mint - Get holder count for token
  GET /health - Health check
```

### 3. Проверить порт

```bash
# Проверить, слушается ли порт
netstat -tuln | grep 56789
# или
ss -tuln | grep 56789
```

### 4. Проверить API изнутри контейнера

```bash
# Health check изнутри контейнера
docker exec solana-holder-bot-prod curl http://localhost:56789/health

# Получить holders изнутри контейнера
docker exec solana-holder-bot-prod curl http://localhost:56789/holders/9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump
```

### 5. Проверить API с хоста

```bash
# Health check с хоста
curl http://127.0.0.1:56789/health

# Получить holders с хоста
curl http://127.0.0.1:56789/holders/9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump
```

## Автоматическая диагностика

Запустите скрипт диагностики:

```bash
chmod +x diagnose-server.sh
./diagnose-server.sh
```

## Исправление проблемы

### Вариант 1: Перезапуск через docker-compose

```bash
# Остановить
docker-compose -f docker-compose.prod.yml down

# Запустить заново
docker-compose -f docker-compose.prod.yml up -d

# Проверить логи
docker logs solana-holder-bot-prod -f
```

### Вариант 2: Использовать скрипт исправления

```bash
chmod +x fix-api-server.sh
./fix-api-server.sh
```

### Вариант 3: Ручной запуск

```bash
# Остановить старый контейнер
docker stop solana-holder-bot-prod
docker rm solana-holder-bot-prod

# Запустить новый с правильными параметрами
docker run -d \
    --name solana-holder-bot-prod \
    --restart unless-stopped \
    -p 56789:56789 \
    -e MINT_ADDRESS=9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump \
    -e RPC_URL=https://api.mainnet-beta.solana.com \
    -e INTERVAL=30 \
    -e CACHE_TTL=30 \
    solana-holder-bot:latest \
    ./solana-holder-bot \
        9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump \
        --rpc-url https://api.mainnet-beta.solana.com \
        --interval 30 \
        --api \
        --api-port 56789 \
        --cache-ttl 30
```

## Проверка конфигурации docker-compose

Убедитесь, что в `docker-compose.prod.yml` есть флаг `--api`:

```yaml
command:
  - ./solana-holder-bot
  - ${MINT_ADDRESS}
  - --rpc-url
  - ${RPC_URL}
  - --interval
  - ${INTERVAL}
  - --api          # ← Этот флаг обязателен!
  - --api-port
  - "56789"
  - --cache-ttl
  - ${CACHE_TTL}
```

## Частые проблемы

### Проблема: API не запускается

**Решение:** Проверьте логи на наличие ошибок RPC:
```bash
docker logs solana-holder-bot-prod | grep -i error
```

### Проблема: Порт не пробрасывается

**Решение:** Убедитесь, что в docker-compose есть:
```yaml
ports:
  - "56789:56789"
```

### Проблема: API запускается, но не отвечает

**Решение:** Проверьте, что контейнер не падает:
```bash
docker ps -a | grep solana-holder-bot-prod
```

Если статус `Exited`, проверьте логи для причины.

## Проверка после исправления

```bash
# 1. Health check
curl http://127.0.0.1:56789/health

# 2. Получить holders
curl http://127.0.0.1:56789/holders/9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump

# 3. Через nginx (если настроен)
curl https://sminem.fun/api-holders/health
```

