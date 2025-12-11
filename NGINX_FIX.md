# Исправление конфигурации nginx

## Проблема

В вашей конфигурации:

```nginx
location /api-sol/ {
    proxy_pass http://127.0.0.1:56789;  # ❌ Без trailing slash
}
```

Это означает, что запрос `https://sminem.fun/api-sol/holders/TOKEN` будет проксирован как `http://127.0.0.1:56789/api-sol/holders/TOKEN` (неправильно!)

## Решение

Добавьте **trailing slash** в `proxy_pass`:

```nginx
location /api-sol/ {
    proxy_pass http://127.0.0.1:56789/;  # ✅ С trailing slash
}
```

Теперь запрос `https://sminem.fun/api-sol/holders/TOKEN` будет проксирован как `http://127.0.0.1:56789/holders/TOKEN` (правильно!)

## Полная исправленная конфигурация

Замените блок `location /api-sol/` на:

```nginx
location /api-sol/ {
    proxy_pass http://127.0.0.1:56789/;  # ⚠️ ВАЖНО: trailing slash!

    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
    proxy_cache_bypass $http_upgrade;

    # Таймауты для долгих запросов
    proxy_connect_timeout 60s;
    proxy_send_timeout 60s;
    proxy_read_timeout 60s;

    # CORS заголовки
    add_header Access-Control-Allow-Origin * always;
    add_header Access-Control-Allow-Methods "GET, OPTIONS" always;
    add_header Access-Control-Allow-Headers "Content-Type" always;
}
```

## Применение изменений

```bash
# 1. Проверить синтаксис
sudo nginx -t

# 2. Перезагрузить nginx
sudo systemctl reload nginx
```

## Проверка

После перезагрузки проверьте:

```bash
# Health check
curl https://sminem.fun/api-sol/health

# Holders
curl https://sminem.fun/api-sol/holders/9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump
```

## Как это работает

- **Запрос:** `https://sminem.fun/api-sol/holders/TOKEN`
- **Проксируется в:** `http://127.0.0.1:56789/holders/TOKEN` (префикс `/api-sol/` убран благодаря trailing slash)
