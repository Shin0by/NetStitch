# ABI и JSON-контракт

Модуль экспортирует две C ABI функции. Этот ABI остаётся неизменным для расширения Module API: новые возможности передаются только через JSON `action`, `payload` и `commands`.

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

`request_ptr/request_len` содержит UTF-8 JSON `IntegrationHostRequest`.
`out_response` должен получить UTF-8 JSON `IntegrationHostResponse`.
Память ответа освобождается host-ом через `netstitch_integration_free`.

## IntegrationHostRequest

```json
{
  "abi_version": 1,
  "module_id": "ui-entity-showcase-rust",
  "storage_dir": "C:\\...\\integrations\\ui-entity-showcase-rust\\data",
  "action": "ui_action",
  "payload": {
    "ui_values": {
      "note": "typed text",
      "mode": "safe",
      "enabled": true
    },
    "module_background_active": false,
    "module": {
      "background_active": false
    }
  }
}
```

Основные `action`:

- `ui_action` - пользователь нажал header action или action внутри UI-схемы модуля;
- профильные/export/download действия могут быть добавлены модулем, но должны оставаться явными действиями пользователя.

## IntegrationHostResponse

Успешный ответ:

```json
{
  "ok": true,
  "result": {
    "message": "Notice from module",
    "severity": "info",
    "refresh": false,
    "commands": []
  }
}
```

Ошибка:

```json
{
  "ok": false,
  "error": "explanation"
}
```

`severity` использует системные уровни `info`, `success`, `warning`, `error`. Текст `message` попадёт в системный лог NetStitch и в footer `Сообщение`.

## module.json

```json
{
  "schema": "netstitch.integration.module.v1",
  "id": "ui-entity-showcase-rust",
  "display_name": "Rust Demo Module",
  "display_name_key": "ui_entity_showcase_rust.module.display_name",
  "tooltip": "Rust example module for standard NetStitch UI entities",
  "tooltip_key": "ui_entity_showcase_rust.module.tooltip",
  "icon_label": "R",
  "icon_path": "assets/brand-rust-svgrepo-com.svg",
  "button_color": "#c4551c",
  "header_actions": [
    {
      "id": "start_showcase_background",
      "label": "{context.module.background_action_label}",
      "tooltip": "{context.module.background_status}",
      "enabled": true,
      "pulse_when_background_active": true
    },
    {
      "id": "show_notice",
      "label": "Notice",
      "tooltip": "Show a standard module dialog",
      "enabled": true
    }
  ],
  "ui_schema": [
    {
      "id": "intro",
      "page": "main",
      "entity_type": "help_text",
      "value": "Edit controls and press Notice.",
      "value_key": "ui_entity_showcase_rust.entity.intro.value",
      "opacity": "100%"
    },
    {
      "id": "summary",
      "page": "main",
      "entity_type": "value_label",
      "title": "Monitoring rows",
      "value": "{context.tables.monitoring.selected_total}",
      "align": "left"
    },
    {
      "id": "note",
      "page": "main",
      "entity_type": "text_input",
      "title": "Note",
      "title_key": "ui_entity_showcase_rust.entity.note.title",
      "value": "",
      "placeholder": "Optional text",
      "placeholder_key": "ui_entity_showcase_rust.entity.note.placeholder",
      "clear_button": true,
      "size": "stretch",
      "opacity": "100%"
    },
    {
      "id": "details",
      "page": "main",
      "entity_type": "textarea",
      "title": "Details",
      "value": "",
      "placeholder": "Multiple lines",
      "clear_button": true,
      "commit_on_enter": true,
      "height": "96px"
    },
    {
      "id": "mode",
      "page": "main",
      "entity_type": "select",
      "title": "Mode",
      "value": "safe",
      "options": [
        {
          "value": "safe",
          "label": "Safe"
        },
        {
          "value": "fast",
          "label": "Fast"
        }
      ]
    },
    {
      "id": "enabled",
      "page": "main",
      "entity_type": "switch",
      "title": "Enabled",
      "checked": true
    },
    {
      "id": "phase-progress",
      "page": "main",
      "entity_type": "progress",
      "title": "Progress",
      "value": "68",
      "progress_stages": [
        {
          "color": "accent",
          "percent": 30,
          "name": "Queued"
        },
        {
          "color": "rust",
          "percent": 70,
          "name": "Processing"
        },
        {
          "color": "success",
          "percent": 100,
          "name": "Done"
        }
      ],
      "size": "stretch",
      "width": "100%",
      "grid_column": "1 / -1"
    },
    {
      "id": "actions",
      "page": "main",
      "entity_type": "action_button",
      "actions": [
        {
          "id": "show_notice",
          "label": "Notice",
          "tooltip": "Open a standard module dialog",
          "style": "primary",
          "align": "left",
          "pulse": true,
          "enabled": true
        },
        {
          "id": "disabled_action",
          "label": "Disabled",
          "align": "center",
          "enabled": false
        }
      ]
    },
    {
      "id": "background-table",
      "page": "background_subscription",
      "entity_type": "table",
      "scroll": "both",
      "size": "stretch",
      "height": "180px",
      "table_columns": [
        {
          "index": 0,
          "text_field": true,
          "width": "24%"
        },
        {
          "index": 2,
          "text_field": true,
          "width": "38%"
        },
        {
          "index": 3,
          "width": "10%",
          "align": "right"
        }
      ],
      "value": "{context.monitoring.latest_rows}"
    }
  ],
  "transport": "native_library",
  "library_paths": {
    "windows-x86_64": "bin/ui_entity_showcase_rust.dll",
    "linux-x86_64": "bin/libui_entity_showcase_rust.so",
    "macos-aarch64": "bin/libui_entity_showcase_rust.dylib",
    "default": "bin/ui_entity_showcase_rust.dll"
  }
}
```

`library_paths` выбирается по `<os>-<arch>`, затем `<os>`, затем `default`.

Windows и Linux не получают отдельный ABI или отдельный формат manifest: меняется только platform library path (`.dll` для Windows, `.so` для Linux).

## Reserved module names

Имена `core` и `system` зарезервированы регистронезависимо для `id` и `display_name` модуля, потому что `system_events.source` различает host/system события и события внешних модулей.

## Host event subscriptions

Подписки на события host-а включаются только после явного пользовательского действия, когда модуль возвращает host-команду `start_background` с массивом `subscriptions`:

```json
{
  "command_type": "start_background",
  "payload": {
    "subscriptions": [
        "ui.controls",
        "ui.tables",
        "filters.*",
        "monitoring.*",
        "tracked_apps.*"
    ]
  }
}
```

Подписка может быть точным event type (`filters.changed`) или wildcard-prefix (`monitoring.*`). Пустой список подписок нормализуется в `*`. Host отправляет такие события тем же C/JSON entrypoint-ом как `IntegrationHostRequest` с `action = "background_event"`. В `payload` лежит `IntegrationModuleBackgroundEventDto`:

```json
{
  "abi_version": 1,
  "module_id": "ui-entity-showcase-rust",
  "storage_dir": "C:\\...\\integrations\\ui-entity-showcase-rust\\data",
  "action": "background_event",
  "payload": {
    "event_type": "filters.changed",
    "created_at_ms": 1710000000000,
    "context": {
      "app_version": "1.1.1.456",
      "language_code": "ru-ru",
      "monitoring_active": false,
      "module_background_active": true,
      "filters": {
        "app_search": "",
        "ip_search": "",
        "domain_search": "*.example.com",
        "port_search": "",
        "protocol": "All",
        "public_ip": true,
        "observation_filter": "All"
      },
      "tables": [],
      "selected_monitoring_row_ids": [],
      "displayed_monitoring_row_ids": [],
      "integration_module_count": 1,
      "tracked_app_count": 0,
      "enabled_tracked_app_count": 0
    },
    "payload": {
      "source": "module_command"
    }
  }
}
```

Подписки нужны для обновления module UI, фоновой module-owned работы и module-owned state. Они не дают модулю права автоматически запускать cloud upload/download или CSV import/export. `start_background` разрешён только как host-команда из явного пользовательского `ui_action`. Открытие overlay первого уровня считается таким действием только для модулей, которые явно объявили action `module.open` или `open`; discovery/bootstrap модуля не стартует background и не подписывает модуль на события. При закрытии overlay фоновая работа может продолжаться, пока пользователь или сам модуль не вернёт `stop_background`. Стандартная host-кнопка `Остановить` в главном header-е модуля вызывает stop-контур host-а, снимает подписки выбранного модуля, останавливает его background state и закрывает все overlay этого модуля.

Для второго уровня module UI используется команда `set_module_page`:

```json
{
  "command_type": "set_module_page",
  "payload": {
    "page": "last_rows"
  }
}
```

## Severity и system log

`severity` всегда один из четырёх типов: `info`, `success`, `warning`, `error`.

Системный лог хранит источник события в отдельном поле `source`:

- `core` - события основного runtime/host-а;
- отображаемое имя модуля из manifest - события конкретного модуля.

Источники `core` и `system` зарезервированы регистронезависимо и не могут быть `id` или `display_name` модуля.
