# Настройка контейнера в режиме Host Network

## Что изменилось

Контейнер теперь использует `network_mode: host`, что означает:
- Контейнер использует сеть хоста напрямую
- Порт 56789 доступен напрямую на хосте без проброса
- Не нужно указывать `-p 56789:56789` в docker run
- Более простая конфигурация сети

## Преимущества

1. **Простота** - не нужно настраивать проброс портов
2. **Производительность** - нет overhead от NAT
3. **Прямой доступ** - порт доступен как будто приложение запущено на хосте

## Использование

### Через docker-compose

```bash
docker-compose -f docker-compose.prod.yml up -d
```

### Через docker run

```bash
docker run -d \
    --name solana-holder-bot-prod \
    --restart unless-stopped \
    --network host \
    -e MINT_ADDRESS=9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump \
    -e RPC_URL=https://api.mainnet-beta.solana.com \
    solana-holder-bot:latest \
    ./solana-holder-bot \
        9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump \
        --rpc-url https://api.mainnet-beta.solana.com \
        --api \
        --api-port 56789
```

## Проверка работы

```bash
# Health check
curl http://127.0.0.1:56789/health

# Получить holders
curl http://127.0.0.1:56789/holders/9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump

# Через nginx (если настроен)
curl https://sminem.fun/api-holders/health
```

## Важные замечания

1. **Безопасность**: В режиме host network контейнер имеет полный доступ к сети хоста
2. **Порты**: Убедитесь, что порт 56789 не занят другим приложением
3. **Firewall**: Настройте firewall для порта 56789, если нужно ограничить доступ

## Проверка порта

```bash
# Проверить, занят ли порт
netstat -tuln | grep 56789
# или
ss -tuln | grep 56789
```

## Ограничения

- Режим host network работает только на Linux
- На macOS и Windows Docker Desktop использует виртуальную машину, поэтому host network может работать не так, как ожидается
- В production на Linux это оптимальный вариант

