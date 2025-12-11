# Production Deployment Guide

## Настройка GitHub Actions (опционально)

Если хотите использовать автоматический деплой через GitHub Actions:

1. **Добавьте secrets в GitHub репозиторий:**
   - `Settings` → `Secrets and variables` → `Actions`
   - Добавьте (опционально):
     - `DOCKER_USERNAME` - для публикации образа в Docker Hub
     - `DOCKER_PASSWORD` - пароль Docker Hub
   - Добавьте (для деплоя на сервер):
     - `DEPLOY_HOST` - IP адрес сервера
     - `DEPLOY_USER` - пользователь для SSH
     - `DEPLOY_SSH_KEY` - приватный SSH ключ
     - `DEPLOY_PORT` - SSH порт (по умолчанию 22)

2. **Workflow запустится автоматически** при push в ветки `main` или `prod`

**Примечание:** Если secrets не настроены, workflow просто соберет образ без публикации и деплоя.

## Быстрый деплой

### Windows (PowerShell)
```powershell
.\deploy.ps1
```

### Linux/Mac (Bash)
```bash
chmod +x deploy.sh
./deploy.sh
```

## Ручной деплой

### 1. Сборка образа
```bash
docker build -t solana-holder-bot:latest .
```

### 2. Запуск контейнера
```bash
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
    ${MINT_ADDRESS} \
    --rpc-url ${RPC_URL} \
    --interval ${INTERVAL} \
    --api \
    --api-port 56789 \
    --cache-ttl ${CACHE_TTL}
```

### 3. Через docker-compose
```bash
# Установить переменные окружения
export MINT_ADDRESS=9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump
export RPC_URL=https://api.mainnet-beta.solana.com

# Запустить
docker-compose -f docker-compose.prod.yml up -d
```

## Проверка работы

### Проверка контейнера
```bash
docker ps | grep solana-holder-bot
```

### Проверка логов
```bash
docker-compose -f docker-compose.prod.yml logs -f
```

### Проверка API
```bash
curl http://localhost:56789/health
curl http://localhost:56789/holders/9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump
```

## Доступ извне

Порт **56789** проброшен на хост и доступен извне.

### Локальный доступ
```
http://localhost:56789/holders/:mint
```

### Доступ из сети
```
http://YOUR_SERVER_IP:56789/holders/:mint
```

### Настройка firewall (если нужно)

**Ubuntu/Debian:**
```bash
sudo ufw allow 56789/tcp
```

**CentOS/RHEL:**
```bash
sudo firewall-cmd --permanent --add-port=56789/tcp
sudo firewall-cmd --reload
```

## Переменные окружения

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `MINT_ADDRESS` | Адрес токена для мониторинга | Обязательно |
| `RPC_URL` | Solana RPC endpoint | `https://api.mainnet-beta.solana.com` |
| `INTERVAL` | Интервал опроса (секунды) | `30` |
| `CACHE_TTL` | Время жизни кэша (секунды) | `30` |
| `MAX_RETRIES` | Максимум повторных попыток | `3` |
| `TIMEOUT` | Таймаут RPC запроса (секунды) | `30` |
| `RUST_LOG` | Уровень логирования | `info` |

## Управление контейнером

### Остановка
```bash
docker-compose -f docker-compose.prod.yml down
```

### Перезапуск
```bash
docker-compose -f docker-compose.prod.yml restart
```

### Обновление
```bash
docker-compose -f docker-compose.prod.yml pull
docker-compose -f docker-compose.prod.yml up -d --build
```

### Просмотр логов
```bash
docker-compose -f docker-compose.prod.yml logs -f
```

## Production рекомендации

1. **Используйте приватный RPC** для лучшей производительности
2. **Настройте мониторинг** (Prometheus, Grafana)
3. **Настройте логирование** в централизованную систему
4. **Используйте reverse proxy** (nginx, traefik) для HTTPS
5. **Настройте автоматический перезапуск** при сбоях

## Пример с nginx reverse proxy

```nginx
server {
    listen 80;
    server_name api.yourdomain.com;

    location / {
        proxy_pass http://localhost:56789;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

## Troubleshooting

### Контейнер не запускается
```bash
docker-compose -f docker-compose.prod.yml logs
```

### Порт занят
```bash
# Проверить, что использует порт
netstat -tulpn | grep 56789
# Или изменить порт в docker-compose.prod.yml
```

### API не отвечает
```bash
# Проверить логи
docker-compose -f docker-compose.prod.yml logs -f

# Проверить здоровье контейнера
docker ps
docker inspect solana-holder-bot-prod
```

