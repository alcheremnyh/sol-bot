# Быстрый запуск

## Для токена: 9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump

### Вариант 1: Прямой запуск

```bash
cargo run --release -- 9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump --rpc-url https://api.mainnet-beta.solana.com --interval 30
```

### Вариант 2: После сборки

```bash
# Сборка
cargo build --release

# Запуск
./target/release/solana-holder-bot 9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump --rpc-url https://api.mainnet-beta.solana.com --interval 30
```

### Вариант 3: Windows (через run_bot.bat)

Просто запустите `run_bot.bat` двойным кликом или из командной строки.

## Публичные RPC endpoints для Solana

Если основной RPC не работает, попробуйте альтернативы:

1. **Основной публичный**: `https://api.mainnet-beta.solana.com`
2. **Ankr**: `https://rpc.ankr.com/solana`
3. **Project Serum**: `https://solana-api.projectserum.com`
4. **QuickNode (требует регистрации)**: `https://your-endpoint.solana-mainnet.quiknode.pro/YOUR_KEY/`

## Параметры

- `--interval 30` - интервал опроса в секундах (по умолчанию 30)
- `--rpc-url` - URL RPC endpoint
- `--json-log` - включить JSON логирование
- `--max-retries 5` - количество повторных попыток при ошибках
- `--timeout 60` - таймаут запроса в секундах

## Пример с альтернативным RPC

```bash
cargo run --release -- 9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump --rpc-url https://rpc.ankr.com/solana --interval 30
```

## Что делать если RPC не отвечает?

1. Попробуйте другой публичный RPC из списка выше
2. Увеличьте таймаут: `--timeout 60`
3. Увеличьте количество retry: `--max-retries 5`
4. Используйте приватный RPC (Helius, QuickNode и т.д.)

