# Демонстрационный модуль C++

Это готовый пример внешнего native-модуля NetStitch на C++. Он функционально совпадает с Rust-примером: тот же UI, те же action id, те же host-команды и та же C/JSON ABI boundary.

## Что смотреть в первую очередь

- `module.json` - manifest модуля. В нём объявлены `id`, `display_name`, `header_actions`, `ui_schema` и `library_paths`.
- `locales/en-en.ini` и `locales/ru-ru.ini` - строки manifest UI для английского fallback и русского интерфейса.
- `ui_entity_showcase_module.cpp` - C ABI entrypoint `netstitch_integration_call`, освобождение памяти `netstitch_integration_free` и обработка `ui_action` / `background_event`.
- `CMakeLists.txt` - минимальная сборка shared library без сторонних библиотек.

## Важные места в module.json

- В самом `module.json` комментарии невозможны: это обычный JSON, поэтому пояснения вынесены в этот README и в комментарии `ui_entity_showcase_module.cpp`.
- `header_actions[0].id = "start_showcase_background"` - кнопка в header модуля. Она вызывает C++ handler через стандартный `ui_action`.
- `display_name_key`, `title_key`, `value_key`, `placeholder_key`, `label_key` и `tooltip_key` - ключи из `locales/*.ini`; обычные поля рядом остаются fallback-ом, если локаль не найдена.
- `button_color = "#1287d8"` - синий цвет кнопки C++ модуля в панели `Модули`.
- `ui_schema` - декларативный UI. Host сам рисует controls одинаково в desktop и browser shell.
- вкладка `browse_windows` показывает `browse_window`: host открывает окно выбора папки или save target, а модуль получает только выбранный путь и статус в `payload.ui_values`.
- `grid` - panel-like контейнер с `columns`, `gap`, `grid_column`.
- `text_input` и `textarea` - host-owned controls. Модуль получает их значения в `payload.ui_values`; в примере `showcase-input` использует `clear_button: true`, а `showcase-textarea` включает `commit_on_enter: true`, чтобы показать Enter-применение с host-индикацией.
- `simulate_download` - кнопка, которая во время blocking `ui_action` шлёт live `IntegrationHostEvent.event = "ui_values"` и плавно проводит большой `showcase-progress` от `0` до `100` через фазы `В очереди`, `Обработка`, `Готово`.
- `table` - TSV-таблица с заголовком в первой строке `value`.
- `table_columns` - per-column настройки. `text_field: true` включает ограничивающий текстовый контейнер для длинных значений в выбранной колонке.
- `library_paths` - platform-specific путь к `.dll`, `.so` или `.dylib` внутри установленного модуля.

## Как работает код

1. Host вызывает `netstitch_integration_call` и передаёт request JSON.
2. Пример читает `action` и `action_id`.
3. Пример читает `context.language_code` и выбирает RU/EN runtime-сообщения для status, log_event и dialog.
4. Для `inspect_ui_values` модуль возвращает `set_ui_values`, чтобы host сбросил значения controls.
5. Для `browse_folder_window` и `browse_save_window` модуль возвращает `browse_window`; host открывает picker и обновляет label-и пути/статуса без записи файлов.
6. Для `simulate_download` модуль вызывает ABI event callback и обновляет `showcase-progress` live через `ui_values`, пока `ui_action` ещё выполняется.
7. Для `start_showcase_background` модуль возвращает `start_background` или `stop_background`; это действие запускается кнопкой в главном header-е модуля.
8. Для `show_notice` модуль возвращает `show_dialog`, а dialog рисует NetStitch.
9. Ответ выделяется через `new[]`; host обязан вызвать `netstitch_integration_free`, где память освобождается через `delete[]`.

## Что менять при копировании

- Меняйте `id`, `display_name`, `display_name_key`, `icon_label` и имена библиотек одновременно в `module.json`, `locales/*.ini` и `CMakeLists.txt`.
- Оставляйте `netstitch_integration_call` и `netstitch_integration_free` без переименования: host ищет именно эти export symbols.
- Добавляйте новые кнопки через `header_actions` или `action_button.actions`, затем обрабатывайте их по `action_id` в `ui_action_response`.
- Для сложных payload-ов подключите JSON-библиотеку вроде `nlohmann/json`; строковые helper-ы в примере оставлены только ради минимальности.
- При изменении исходников или manifest обязательно пересоберите архивы `scripts\package_module_sdk_examples.ps1`, потому что пользовательские zip включают эти файлы.

## Локальная сборка

```powershell
cmake -S docs\module-sdk\examples\ui-entity-showcase-cpp -B temp\module-sdk-build\windows\cpp -DCMAKE_BUILD_TYPE=Release
cmake --build temp\module-sdk-build\windows\cpp --config Release
```

`CMakeLists.txt` включает `/utf-8` для MSVC, чтобы русские runtime-строки в исходнике C++ стабильно компилировались на Windows.

Для пользовательской установки обычно брать готовый архив из `docs/module-sdk/packages`, потому что в нём уже есть `module.json`, `bin/` и исходники.
