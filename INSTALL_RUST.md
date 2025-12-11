# Установка Rust для Windows

## Быстрая установка

1. **Скачайте установщик Rust:**
   - Перейдите на https://rustup.rs/
   - Или скачайте напрямую: https://win.rustup.rs/x86_64

2. **Запустите установщик:**
   - Запустите `rustup-init.exe`
   - Нажмите Enter для установки по умолчанию
   - Дождитесь завершения установки

3. **Перезапустите терминал** (или выполните в текущем):
   ```powershell
   $env:Path += ";$env:USERPROFILE\.cargo\bin"
   ```

4. **Проверьте установку:**
   ```bash
   cargo --version
   rustc --version
   ```

## После установки

Соберите проект:
```bash
cargo build --release
```

Запустите бота:
```bash
cargo run --release -- 9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump --rpc-url https://api.mainnet-beta.solana.com --interval 30
```

## Альтернатива: Использование готового бинарника

Если не хотите устанавливать Rust, можно использовать готовый бинарник (если доступен) или Docker.

## Проблемы?

- Убедитесь, что добавили `%USERPROFILE%\.cargo\bin` в PATH
- Перезапустите терминал после установки
- Проверьте, что установлен Visual Studio Build Tools (требуется для компиляции)

