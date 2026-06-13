# Демонстрационный модуль Rust

Это готовый пример внешнего native-модуля NetStitch на Rust. Он показывает типовые UI-сущности, вкладки, `grid`, таблицы, editable controls, header action, стандартный dialog и background-подписку на события host-а.

## Что смотреть в первую очередь

- `module.json` - manifest модуля. В нём объявлены `id`, `display_name`, `header_actions`, `ui_schema` и `library_paths`.
- `locales/en-en.ini` и `locales/ru-ru.ini` - строки manifest UI для английского fallback и русского интерфейса.
- `src/lib.rs` - C ABI entrypoint `netstitch_integration_call`, освобождение памяти `netstitch_integration_free` и обработка `ui_action` / `background_event`.
- `Cargo.toml` - минимальная сборка `cdylib` без внешних зависимостей.

## Важные места в module.json

- В самом `module.json` комментарии невозможны: это обычный JSON, поэтому пояснения вынесены в этот README и в комментарии `src/lib.rs`.
- `header_actions[0].id = "start_showcase_background"` - кнопка в header модуля. Она вызывает тот же `ui_action`, что и кнопки внутри схемы.
- `display_name_key`, `title_key`, `value_key`, `placeholder_key`, `label_key` и `tooltip_key` - ключи из `locales/*.ini`; обычные поля рядом остаются fallback-ом, если локаль не найдена.
- `icon_path = "assets/brand-rust-svgrepo-com.svg"` - module-owned SVG-иконка Rust-примера; C++ пример намеренно оставлен без `icon_path`, чтобы показать оба варианта.
- `icon_path` показывает module-owned SVG-иконку на всю кнопку модуля.
- `button_color = "#c4551c"` - яркий ржавый цвет кнопки Rust-модуля в панели `Модули`.
- `ui_schema` - один декларативный UI для desktop и browser shell. Модуль не пишет HTML, CSS или Dioxus-код.
- `entity_type = "grid"` - контейнер для колонок и `grid_column` placement дочерних controls.
- `entity_type = "text_input"` / `"textarea"` - host-owned поля; в примере `showcase-input` использует `clear_button: true`, а `showcase-textarea` включает `commit_on_enter: true`, чтобы показать Enter-применение с host-индикацией.
- `entity_type = "table"` - TSV-таблица. Первая строка `value` считается заголовком.
- `table_columns` - настройки конкретных колонок таблицы. `text_field: true` включает для этой колонки тот же ограничивающий `path-field` контейнер, который используется в основных таблицах NetStitch.
- `library_paths` - platform-specific путь к собранной библиотеке внутри архива модуля.

## Как работает код

1. Host вызывает `netstitch_integration_call` и передаёт request JSON.
2. Пример читает `action` и `action_id`.
3. Пример читает `context.language_code` и выбирает RU/EN runtime-сообщения для status, log_event и dialog.
4. Для `inspect_ui_values` модуль возвращает `set_ui_values`, чтобы host сбросил значения controls.
5. Для `start_showcase_background` модуль возвращает `start_background` или `stop_background`; это действие запускается кнопкой в главном header-е модуля.
6. Для `show_notice` модуль возвращает `show_dialog`, а сам dialog рисует NetStitch.
7. Ответ выделяется в памяти модуля и освобождается host-ом через `netstitch_integration_free`.

## Что менять при копировании

- Меняйте `id`, `display_name`, `display_name_key`, `icon_label` и имена библиотек одновременно в `module.json`, `locales/*.ini` и `Cargo.toml`.
- Оставляйте `netstitch_integration_call` и `netstitch_integration_free` без переименования: host ищет именно эти export symbols.
- Добавляйте новые кнопки через `header_actions` или `action_button.actions`, затем обрабатывайте их по `action_id` в `ui_action_response`.
- Для сложных payload-ов подключите `serde_json` и разберите request в структуры; строковые helper-ы в примере оставлены только ради минимальности.
- При изменении исходников или manifest обязательно пересоберите архивы `scripts\package_module_sdk_examples.ps1`, потому что пользовательские zip включают эти файлы.

## Локальная сборка

```powershell
cargo build --manifest-path docs\module-sdk\examples\ui-entity-showcase-rust\Cargo.toml --release
```

Для пользовательской установки обычно брать готовый архив из `docs/module-sdk/packages`, потому что в нём уже есть `module.json`, `bin/` и исходники.
