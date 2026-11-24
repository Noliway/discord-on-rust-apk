# Rustcord

Мини-Discord на Rust: сервер чата (axum) и Android-клиент (egui) в одном репозитории.

- `server/` — HTTP-сервер с каналами и сообщениями (хранит в памяти, последние 500 на канал).
- `client/` — приложение для Android (и десктопная версия для отладки).
- `.github/workflows/build-apk.yml` — собирает APK при каждом пуше.

## Скачать APK

После пуша в `main` APK появляется в разделе **Releases → Latest build** (`rustcord.apk`),
а также в артефактах запуска на вкладке **Actions**.
APK подписан debug-ключом: для установки включите «Установка из неизвестных источников».

## Запуск сервера

```bash
cargo run -p rustcord-server          # слушает 0.0.0.0:8080
BIND=0.0.0.0:3000 cargo run -p rustcord-server
```

В приложении укажите адрес сервера, например `http://192.168.1.10:8080`.
В эмуляторе Android хост-машина доступна как `http://10.0.2.2:8080`.

## Десктопная версия клиента (для отладки)

```bash
cargo run -p rustcord-client --features desktop --bin rustcord-desktop
```

## Локальная сборка APK

```bash
rustup target add aarch64-linux-android
cargo install cargo-apk
# нужны Android SDK и NDK, переменные ANDROID_HOME и ANDROID_NDK_ROOT
cd client && cargo apk build --release --lib
```

## API

- `GET  /channels/:name/messages?after=<id>` — новые сообщения
- `POST /channels/:name/messages` — `{"author": "...", "text": "..."}`

## Ограничения MVP

Нет аккаунтов, паролей, шифрования (HTTP), голоса, вложений и сохранения истории на диск.
