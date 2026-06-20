# Создание модуля NetStitch

Этот документ описывает полный путь создания модуля: от папки с исходниками до installable-архива, который пользователь может положить в `integrations/`.

## 1. Выберите основу

В SDK есть два одинаковых примера одного модуля:

- `docs/module-sdk/examples/ui-entity-showcase-rust/` - Rust `cdylib`;
- `docs/module-sdk/examples/ui-entity-showcase-cpp/` - C++ shared library через CMake.

Оба примера используют один и тот же `ui_schema`, одни и те же action id и один C/JSON ABI. Разница только в языке реализации native library. Для нового модуля проще скопировать ближайший пример и заменить `id`, `display_name`, UI-схему и action handler-ы.

## 2. Структура папки модуля

```text
integrations/
  my-module/
    module.json
    locales/
      en-en.ini
      ru-ru.ini
    bin/
      my_module.dll
      libmy_module.so
      libmy_module.dylib
    assets/
    data/
```

- `module.json` - manifest, UI-схема и platform-specific `library_paths`.
- `locales/` - строки UI модуля внутри папки самого модуля; `en-en.ini` используется как fallback, `ru-ru.ini` перекрывает строки для русского UI.
- `bin/` - собранная shared library под нужную платформу.
- `assets/` - необязательные статичные файлы модуля.
- `data/` - runtime-state модуля: SQLite, cache, downloaded files. Это не часть source package и не должно попадать в git.

## 3. Manifest

Минимальные поля:

- `schema = "netstitch.integration.module.v1"`;
- `id` - стабильный lowercase id папки модуля;
- `display_name` - название в панели `Модули`;
- `tooltip`, `icon_label`, `button_color`, `icon_path` - отображение кнопки модуля; если задан `icon_path`, NetStitch показывает module-owned иконку на всю кнопку;
- `display_name_key`, `tooltip_key`, `title_key`, `value_key`, `placeholder_key`, `label_key` - ключи локализации из `locales/*.ini`, если модулю нужны RU/EN строки;
- `header_actions` - квадратные action-кнопки в header модуля;
- `ui_schema` - декларативные UI-сущности;
- `transport = "native_library"`;
- `library_paths` - путь к `.dll`, `.so`, `.dylib` внутри папки модуля.

Windows и Linux используют один manifest и один ABI. Меняется только имя native library:

```json
{
  "library_paths": {
    "windows-x86_64": "bin/my_module.dll",
    "linux-x86_64": "bin/libmy_module.so",
    "macos-aarch64": "bin/libmy_module.dylib",
    "default": "bin/my_module.dll"
  }
}
```

## 4. UI-схема

Модуль не рисует HTML, CSS или Dioxus controls. Он описывает UI через `ui_schema`, а NetStitch одинаково рендерит её в desktop shell и browser shell.

Основные сущности:

- `nested_subpanel` - готовый двухслойный контейнер: внешняя подпанель темнее, внутренняя светлее, как строки в блоке приложений;
- `grid` - организация controls по столбцам;
- `text_input`, `textarea`, `select`, `switch` - host-owned controls;
- `clear_button: true` - опциональная встроенная кнопка очистки для конкретного `text_input` или `textarea`;
- `commit_on_enter: true` - опциональное применение значения по Enter для `text_input` и `textarea` со стандартной host-индикацией;
- `progress` / `progress_stages` - шкала прогресса с опциональными фазами `{ color, percent, name }`, где `percent` задаёт правую границу фазы;
- `table` - TSV-таблица с первой строкой-заголовком;
- `table_columns` - настройки конкретных столбцов таблицы;
- `action_button` - кнопки внутри body; несколько actions по умолчанию идут в один горизонтальный ряд (`button_layout: "row"`), `align` у сущности выравнивает весь ряд, `button_layout: "column"` явно включает вертикальный список;
- `footer` - левая slot-часть стандартного footer-а overlay; host-owned кнопка `Назад` отображается справа по умолчанию, а `hide_host_back_button: true` скрывает её только для полностью кастомного footer-а;
- `help_text`, `separator`, `value`, `status` - текстовые и структурные элементы.

Renderer desktop/browser применяет общие дефолты до отображения: UI-сущность без размеров получает `size: "stretch"`, `width: "100%"`, `min_width: "0"` и `align: "left"`, а `panel`, `subpanel`, `nested_subpanel`, `grid` и `tabs` по умолчанию получают `height: "auto"`, `min_height: "0"` и `scroll: "off"`. Явные поля manifest-а перекрывают эти значения.

Для длинных значений в таблицах используйте `table_columns`:

```json
{
  "id": "latest-table",
  "entity_type": "table",
  "scroll": "both",
  "height": "220px",
  "table_columns": [
    { "index": 0, "text_field": true, "width": "24%" },
    { "index": 2, "text_field": true, "width": "38%" },
    { "index": 3, "width": "10%", "align": "right" }
  ],
  "value": "App\tIP\tDomain\tPort\nCode.exe\t203.0.113.10\tmain.example.test\t443"
}
```

`text_field: true` включает для конкретной колонки стандартный однострочный `path-field` контейнер. Он удерживает длинный текст внутри ячейки и не даёт ему налезать на соседние столбцы.

Для локализации видимых строк оставляйте английский fallback прямо в `module.json`, а рядом добавляйте ключ. Ключи принадлежат модулю, поэтому используйте собственный префикс вроде `my_module.*`; не добавляйте строки модуля в `resources/language/*` приложения.

```json
{
  "id": "intro",
  "entity_type": "help_text",
  "value": "Shown when locale files are missing.",
  "value_key": "my_module.entity.intro.value"
}
```

`locales/en-en.ini`:

```ini
[strings]
my_module.entity.intro.value=Shown in English UI.
```

`locales/ru-ru.ini`:

```ini
[strings]
my_module.entity.intro.value=Показано в русском интерфейсе.
```

## 5. Native ABI

Модуль экспортирует две функции:

```c
int32_t netstitch_integration_call(
    const uint8_t* request_ptr,
    uintptr_t request_len,
    void (*event_callback)(const uint8_t*, uintptr_t, void*),
    void* event_user_data,
    NetStitchAbiBuffer* out_response
);

void netstitch_integration_free(uint8_t* ptr, uintptr_t len);
```

Host передаёт UTF-8 JSON request. Модуль возвращает UTF-8 JSON response. Память ответа выделяет модуль, а освобождает host через `netstitch_integration_free`.

## 6. Actions и host-команды

Основной action для UI - `ui_action`. В request приходит:

- `action_id`;
- host context;
- selected/displayed Monitoring row ids;
- выбранные rows;
- `payload.ui_values`;
- `payload.module_background_active`.

Модуль отвечает `IntegrationHostResponse` и может вернуть whitelisted commands:

- `set_ui_values` - обновить значения controls, активную вкладку `tabs` или состояние action-кнопок;
- `show_dialog` - открыть стандартный dialog;
- `log_event` - записать событие в `system_events`;
- `start_background` / `stop_background` - управлять фоновыми подписками;
- `set_filters`, `start_monitoring`, `stop_monitoring`, операции tracked apps и monitoring rows.

Если долгий action должен обновлять UI до финального ответа, используйте ABI callback `IntegrationHostEvent.event = "ui_values"` с `payload.values`. Для controls ключ обычно равен `entity.id`; для кнопок используйте `action.<action_id>.enabled` или `action.<action_id>.disabled`, чтобы host одинаково отключил кнопку визуально и заблокировал dispatch `ui_action` в desktop/browser shell. Не рассчитывайте на `download_progress` как на module UI event: модуль явно указывает target id сущности, например `"download-progress": 42`.

Cloud upload/download и CSV import/export не автоматизируются модулями. Они остаются ручными действиями пользователя в основном UI.

## 7. Background events

Background не запускается при discovery или старте NetStitch. Модуль должен вернуть `start_background` из явного пользовательского action:

```json
{
  "command_type": "start_background",
  "payload": {
    "subscriptions": ["ui.controls", "ui.tables", "monitoring.*", "filters.*"]
  }
}
```

После этого host может вызывать тот же native entrypoint с `action = "background_event"`. Модуль может обновлять свой state, писать log events или вернуть refresh-команды, но не должен выполнять cloud/CSV automation.

## 8. Сборка и архивы

После изменения примеров или manifest-а обязательно обновить готовые архивы:

```powershell
scripts\package_module_sdk_examples.ps1
```

Скрипт собирает:

- `ui-entity-showcase-rust-windows-x86_64.zip`;
- `ui-entity-showcase-rust-linux-x86_64.zip`;
- `ui-entity-showcase-cpp-windows-x86_64.zip`;
- `ui-entity-showcase-cpp-linux-x86_64.zip`.

Linux-сборка выполняется только в отдельном тестовом WSL `NetStitch-Linux-Test`. Другие WSL-дистрибутивы не нужны для SDK packaging.

## 9. Checklist перед публикацией модуля

- `module.json` валиден JSON и не содержит секретов.
- `id` не равен `core` или `system`.
- `library_paths` совпадают с фактическими файлами в `bin/`.
- UI использует стандартные `ui_schema` сущности, а не собственный web/native UI.
- Длинные значения в таблицах ограничены через `table_columns[].text_field`.
- Runtime state не попал в архив, кроме пустой или generated `data/` при необходимости.
- Архив пересобран после изменения исходников.
- Модуль проверен в NetStitch на Windows и, если заявлена поддержка, на Linux.
