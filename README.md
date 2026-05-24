# Image Processor with FFI Plugins

CLI-приложение на Rust для обработки изображений с поддержкой динамически загружаемых FFI-плагинов (`.so`, `.dll`, `.dylib`).

Проект демонстрирует:

- безопасную работу с `unsafe` и FFI;
- динамическую загрузку библиотек через `libloading`;
- обработку изображений в формате RGBA8;
- архитектуру Rust Workspace;
- разделение приложения и плагинов на независимые крейты;
- unit + integration testing;
- CI с `clippy`, `fmt` и тестами.

---

## Возможности

- Загрузка PNG-изображений и преобразование в `RGBA8`
- Динамическая загрузка FFI-плагинов
- Передача параметров плагину через JSON
- Сохранение результата в PNG
- Кроссплатформенная поддержка:
  - Linux (`.so`)
  - Windows (`.dll`)
  - macOS (`.dylib`)
- Интеграционные и модульные тесты
- CI-пайплайн через GitHub Actions

---

## Доступные плагины

### `mirror_plugin`

Плагин зеркального отражения изображения.

Поддерживаемые параметры:

| Параметр     | Тип    | Описание                     |
|--------------|---------|------------------------------|
| horizontal   | bool    | Отражение по горизонтали     |
| vertical     | bool    | Отражение по вертикали       |

Пример:

```json
{
  "horizontal": true,
  "vertical": false
}
```

---

### `blur_plugin`

Плагин размытия изображения.

Поддерживаемые параметры:

| Параметр   | Тип | Описание                         |
|------------|------|----------------------------------|
| radius     | u32  | Радиус размытия                  |
| iterations | u32  | Количество проходов размытия     |

Пример:

```json
{
  "radius": 3,
  "iterations": 2
}
```

---

## Требования

- Rust 1.70+
- Cargo

Проверка версии:

```bash
rustc --version
cargo --version
```

---

## Установка и сборка

### Клонирование репозитория

```bash
git clone https://github.com/yourusername/image_ffi_project.git
cd image_ffi_project
```

### Сборка workspace

```bash
cargo build --workspace
```

### Release-сборка

```bash
cargo build --workspace --release
```

---

## Скомпилированные плагины

После сборки динамические библиотеки находятся в:

### Debug

```text
target/debug/
```

### Release

```text
target/release/
```

Названия библиотек зависят от ОС:

| ОС      | Формат библиотеки              |
|---------|--------------------------------|
| Linux   | `libmirror_plugin.so`          |
| Windows | `mirror_plugin.dll`            |
| macOS   | `libmirror_plugin.dylib`       |

Аналогично для `blur_plugin`.

---

## Использование

```bash
cargo run --bin image_processor -- \
  <INPUT> \
  <OUTPUT> \
  <PLUGIN> \
  <PARAMS_FILE> \
  [--plugin-path <DIR>]
```

### Аргументы CLI

| Аргумент       | Описание                              |
|----------------|---------------------------------------|
| `INPUT`        | Входное PNG-изображение               |
| `OUTPUT`       | Выходной PNG-файл                     |
| `PLUGIN`       | Имя плагина без расширения            |
| `PARAMS_FILE`  | JSON-файл с параметрами               |
| `--plugin-path`| Директория с библиотеками плагинов    |

По умолчанию:

```text
--plugin-path = target/debug
```

---

# Примеры

## Горизонтальное отражение

### `mirror.json`

```json
{
  "horizontal": true,
  "vertical": false
}
```

### Запуск

```bash
cargo run --bin image_processor -- \
  input.png \
  output.png \
  mirror_plugin \
  mirror.json \
  --plugin-path target/debug
```

---

## Размытие изображения

### `blur.json`

```json
{
  "radius": 3,
  "iterations": 2
}
```

### Запуск

```bash
cargo run --bin image_processor -- \
  input.png \
  output.png \
  blur_plugin \
  blur.json
```

---

## Тестирование

### Все тесты

```bash
cargo test --workspace
```

### Проверка форматирования

```bash
cargo fmt --all -- --check
```

### Clippy

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

---

## Структура проекта

```text
image_ffi_project/
├── Cargo.toml                    # Rust workspace
├── README.md
├── .github/
│   └── workflows/
│       └── ci.yaml               # GitHub Actions CI
│
├── image_processor/              # Основное CLI-приложение
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs                # Публичная логика
│   │   ├── main.rs               # CLI entrypoint
│   │   ├── error.rs              # Типы ошибок
│   │   └── plugin_loader.rs      # Обёртка над libloading
│   └── tests/
│       └── integration.rs        # Интеграционные тесты
│
├── mirror_plugin/                # FFI-плагин отражения
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
│
├── blur_plugin/                  # FFI-плагин размытия
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
│
└── RUST_SPECIFICATION.md         # Чек-лист ревью Rust-проектов
```

---

## Архитектура

### Основной поток выполнения

1. CLI получает аргументы
2. Загружается PNG
3. Изображение преобразуется в `RGBA8`
4. Загружается динамическая библиотека
5. Через FFI вызывается `process_image`
6. Плагин модифицирует буфер изображения
7. Результат сохраняется в PNG

---

## FFI API

Каждый плагин обязан экспортировать функцию:

```rust
#[no_mangle]
pub unsafe extern "C" fn process_image(
    width: u32,
    height: u32,
    rgba_data: *mut u8,
    params: *const std::os::raw::c_char,
)
```

### Параметры

| Параметр    | Описание                          |
|-------------|-----------------------------------|
| `width`     | Ширина изображения                |
| `height`    | Высота изображения                |
| `rgba_data` | Указатель на RGBA-буфер           |
| `params`    | JSON-строка с параметрами         |

---

## Безопасность FFI

Проект минимизирует использование `unsafe` и изолирует его в небольших участках кода.

### Реализованные меры безопасности

- `unsafe` обёрнут в безопасные абстракции
- Размер буфера вычисляется как:

```text
width * height * 4
```

- Используется `CString` / `CStr`
- Плагины работают только внутри выделенного буфера
- `Library` хранится внутри `Plugin`, предотвращая dangling symbols
- Буферы реконструируются через `ImageBuffer::from_raw`

---

## Используемые технологии

| Библиотека  | Назначение                         |
|-------------|------------------------------------|
| `image`     | Работа с изображениями             |
| `clap`      | CLI                                |
| `libloading`| Динамическая загрузка библиотек    |
| `serde`     | JSON-сериализация                  |
| `serde_json`| Работа с JSON                      |
| `thiserror` | Типизированные ошибки              |
| `anyhow`    | Удобная обработка ошибок           |
| `log`       | Логирование                        |
| `env_logger`| Logger implementation              |

---

## CI

Проект использует GitHub Actions для:

- проверки форматирования;
- linting через Clippy;
- запуска тестов;
- проверки сборки workspace.

Файл CI:

```text
.github/workflows/ci.yaml
```

## Лицензия

MIT