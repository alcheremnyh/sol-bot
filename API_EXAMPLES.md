# Примеры использования API

## 1. Запуск бота с API сервером

```bash
# Запустите бота с включенным API
.\target\release\solana-holder-bot.exe 9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump --api --api-port 56789 --cache-ttl 30
```

После запуска вы увидите:

```
🚀 API server enabled on port 56789 (cache refresh: 30s)
API server started on http://0.0.0.0:56789
Endpoints:
  GET /holders/:mint - Get holder count for token
  GET /health - Health check
```

## 2. Получение количества держателей

### Через curl (Windows PowerShell)

```powershell
# Получить количество держателей
curl http://localhost:56789/holders/9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump

# Или с форматированием JSON
curl http://localhost:56789/holders/9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump | ConvertFrom-Json
```

### Через браузер

Просто откройте в браузере:

```
http://localhost:56789/holders/9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump
```

### Через PowerShell Invoke-WebRequest

```powershell
$response = Invoke-WebRequest -Uri "http://localhost:56789/holders/9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump"
$response.Content | ConvertFrom-Json
```

### Пример ответа:

```json
{
  "mint": "9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump",
  "holders": 1234,
  "timestamp": 1702324800,
  "cached": true
}
```

## 3. Проверка здоровья API

```powershell
curl http://localhost:56789/health
```

Ответ:

```json
{
  "status": "ok",
  "service": "solana-holder-bot-api"
}
```

## 4. Использование в Python

```python
import requests

# Получить количество держателей
response = requests.get("http://localhost:56789/holders/9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump")
data = response.json()

print(f"Token: {data['mint']}")
print(f"Holders: {data['holders']}")
print(f"Timestamp: {data['timestamp']}")
```

## 5. Использование в JavaScript/Node.js

```javascript
// Fetch API
fetch(
  "http://localhost:56789/holders/9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump"
)
  .then((response) => response.json())
  .then((data) => {
    console.log(`Token: ${data.mint}`);
    console.log(`Holders: ${data.holders}`);
  });

// Или с async/await
async function getHolders(mint) {
  const response = await fetch(`http://localhost:8080/holders/${mint}`);
  const data = await response.json();
  return data.holders;
}
```

## 6. Мониторинг нескольких токенов

API автоматически кэширует все запрошенные токены:

```powershell
# Запросить первый токен (будет закэширован)
curl http://localhost:56789/holders/9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump

# Запросить второй токен (тоже будет закэширован)
curl http://localhost:56789/holders/So11111111111111111111111111111111111111112

# Оба токена теперь в кэше и обновляются каждые 30 секунд
```

## 7. Полный пример запуска

```bash
# Запуск с API на порту 8080, кэш обновляется каждые 30 секунд
.\target\release\solana-holder-bot.exe 9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump \
    --rpc-url https://api.mainnet-beta.solana.com \
    --interval 30 \
    --api \
    --api-port 56789 \
    --cache-ttl 30
```

## 8. Обработка ошибок

### Неверный mint адрес

```powershell
curl http://localhost:56789/holders/invalid_address
# Вернет: 400 Bad Request
```

### Токен не найден в кэше (первый запрос)

API автоматически запросит данные у RPC и закэширует их.

## 9. Производительность

- **Кэшированные запросы**: <1ms (мгновенно)
- **Первые запросы**: ~2-5 секунд (зависит от RPC)
- **Обновление кэша**: в фоне, каждые 30 секунд
- **RPC нагрузка**: минимизирована (1 запрос в 30 сек на токен)

## 10. Интеграция в другие приложения

API можно использовать в:

- Веб-приложениях
- Telegram ботах
- Discord ботах
- Мобильных приложениях
- Скриптах мониторинга
- Дашбордах
