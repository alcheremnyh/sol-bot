# Настройка nginx для API на sminem.fun с путем /api-sol/

## Конфигурация nginx

Добавьте в конфигурацию nginx для домена `sminem.fun`:

```nginx
location /api-sol/ {
    proxy_pass http://127.0.0.1:56789/;  # Trailing slash убирает /api-sol/ из пути
    
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
    proxy_cache_bypass $http_upgrade;
    
    # Таймауты
    proxy_connect_timeout 60s;
    proxy_send_timeout 60s;
    proxy_read_timeout 60s;
    
    # CORS
    add_header Access-Control-Allow-Origin * always;
    add_header Access-Control-Allow-Methods "GET, OPTIONS" always;
    add_header Access-Control-Allow-Headers "Content-Type" always;
}
```

## Как работает

- Запрос: `https://sminem.fun/api-sol/holders/TOKEN`
- Проксируется в: `http://127.0.0.1:56789/holders/TOKEN` (префикс `/api-sol/` убран)

## Тестовые запросы

### 1. Health Check

```bash
# Через curl
curl https://sminem.fun/api-sol/health

# Через PowerShell
Invoke-RestMethod -Uri "https://sminem.fun/api-sol/health"

# Через браузер
https://sminem.fun/api-sol/health
```

**Ожидаемый ответ:**
```json
{
  "status": "ok",
  "service": "solana-holder-bot-api"
}
```

### 2. Получить количество держателей

```bash
# Через curl
curl https://sminem.fun/api-sol/holders/9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump

# Через PowerShell
$mint = "9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump"
Invoke-RestMethod -Uri "https://sminem.fun/api-sol/holders/$mint"

# Через браузер
https://sminem.fun/api-sol/holders/9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump
```

**Ожидаемый ответ:**
```json
{
  "mint": "9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump",
  "holders": 1234,
  "timestamp": 1702324800,
  "cached": true
}
```

## Проверка конфигурации nginx

```bash
# Проверить синтаксис
sudo nginx -t

# Перезагрузить nginx
sudo systemctl reload nginx
# или
sudo nginx -s reload
```

## Отладка

### Проверить логи nginx

```bash
# Логи доступа
sudo tail -f /var/log/nginx/access.log

# Логи ошибок
sudo tail -f /var/log/nginx/error.log
```

### Проверить, что API работает локально

```bash
# Должен вернуть JSON
curl http://127.0.0.1:56789/health
```

### Проверить проксирование

```bash
# Через nginx (с verbose для отладки)
curl -v https://sminem.fun/api-sol/health
```

## Важные замечания

1. **Trailing slash важен!** 
   - `proxy_pass http://127.0.0.1:56789/;` (с `/`) - убирает префикс `/api-sol/`
   - `proxy_pass http://127.0.0.1:56789;` (без `/`) - сохраняет префикс (неправильно!)

2. **SSL/HTTPS:** Убедитесь, что у вас настроен SSL сертификат для домена

3. **Firewall:** Убедитесь, что порт 56789 доступен только локально (127.0.0.1)

4. **CORS:** Если нужно использовать API из браузера, CORS заголовки уже добавлены

