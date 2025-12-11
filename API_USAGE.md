# API Usage Guide

## Запуск с API сервером

```bash
.\target\release\solana-holder-bot.exe <MINT_ADDRESS> --api --api-port 56789 --cache-ttl 30
```

## Параметры API

- `--api` - включить API сервер
- `--api-port <PORT>` - порт для API (по умолчанию 56789)
- `--cache-ttl <SECONDS>` - интервал обновления кэша (по умолчанию 30 секунд)

## Endpoints

### GET /holders/:mint

Получить количество держателей токена.

**Пример запроса:**
```bash
curl http://localhost:56789/holders/9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump
```

**Ответ:**
```json
{
  "mint": "9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump",
  "holders": 1234,
  "timestamp": 1702324800,
  "cached": true
}
```

### GET /health

Проверка здоровья сервиса.

**Пример запроса:**
```bash
curl http://localhost:56789/health
```

**Ответ:**
```json
{
  "status": "ok",
  "service": "solana-holder-bot-api"
}
```

## Как работает кэширование

1. **Первый запрос** - данные запрашиваются у RPC и сохраняются в кэш
2. **Последующие запросы** - данные возвращаются из кэша (мгновенно)
3. **Автообновление** - каждые 30 секунд (или указанный `--cache-ttl`) кэш обновляется в фоне
4. **Множественные токены** - кэш поддерживает несколько токенов одновременно

## Примеры использования

### Мониторинг одного токена + API
```bash
.\target\release\solana-holder-bot.exe 9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump \
    --rpc-url https://api.mainnet-beta.solana.com \
    --interval 30 \
    --api \
    --api-port 56789 \
    --cache-ttl 30
```

### Только API сервер (без мониторинга в консоли)
```bash
.\target\release\solana-holder-bot.exe So11111111111111111111111111111111111111112 \
    --api \
    --api-port 56789 \
    --cache-ttl 30
```

## Производительность

- **Кэшированные запросы**: <1ms
- **Обновление кэша**: происходит в фоне, не блокирует API
- **RPC запросы**: только 1 раз в 30 секунд на токен (независимо от количества API запросов)

