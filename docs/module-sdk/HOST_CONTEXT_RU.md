# Host context и данные таблиц

`ui_action` получает `IntegrationModuleUiActionRequestDto`.

```json
{
  "action_id": "say_hello",
  "context": {
    "app_version": "1.1.1.456",
    "language_code": "ru-ru",
    "monitoring_active": false,
    "module_background_active": false,
    "filters": {
      "app_search": "",
      "ip_search": "",
      "domain_search": "",
      "port_search": "",
      "protocol": "All",
      "public_ip": true,
      "observation_filter": "All"
    },
    "tables": [
      {
        "id": "monitoring",
        "total_rows": 1200,
        "displayed_rows": 605,
        "selected_rows": 2
      }
    ],
    "selected_monitoring_row_ids": [1, 2],
    "displayed_monitoring_row_ids": [1, 2, 3],
    "integration_module_count": 1,
    "tracked_app_count": 7,
    "enabled_tracked_app_count": 3
  },
  "monitoring_rows": [],
  "payload": null
}
```

`monitoring_rows` содержит выбранные строки мониторинга. Если строк не выбрано, список пустой, но `displayed_monitoring_row_ids` остаётся доступен как безопасный контекст UI.

## Live-события `ui_action`

Во время долгого `ui_action` модуль может отправлять через ABI callback live-события `IntegrationHostEvent`. Сейчас публично поддержан только UI-state event:

```json
{
  "event": "ui_values",
  "payload": {
    "values": {
      "download-progress": 42,
      "download-status": "Downloading"
    }
  }
}
```

Host применяет эти значения к активному модулю так же, как финальную команду `set_ui_values`, но до завершения blocking `ui_action`. Поздние события не применяются, если host-owned Stop/Close уже инвалидировал action. `download_progress` остаётся внутренним provider-download событием и не маппится host-ом на конкретный progress id модуля.

## Host events

Когда модуль явно запустил background-режим через `start_background`, host вызывает тот же native C/JSON entrypoint с `action = "background_event"`. Payload всегда имеет форму `IntegrationModuleBackgroundEventDto`: `event_type`, `created_at_ms`, `context` и вложенный `payload` с деталями события.

- `filters.changed` - изменённый `UiFiltersDto`;
- `monitoring.started` / `monitoring.stopped` - статус мониторинга, счётчики таблицы и ids выбранных/отображённых строк;
- `monitoring.rows_changed` / `monitoring.rows_deleted` / `monitoring.rows_added` / `monitoring.selection_changed` - изменения строк или выборки monitoring;
- `tracked_apps.changed` - изменение списка или enabled-состояния `Tracked apps`;
- `ui.dialog_result` - результат стандартного диалога, созданного этим же модулем.

Пример `payload` внутри `IntegrationHostRequest`:

```json
{
  "event_type": "monitoring.started",
  "created_at_ms": 1710000000000,
  "context": {
    "app_version": "1.1.1.456",
    "language_code": "ru-ru",
    "monitoring_active": true,
    "module_background_active": true,
    "filters": {
      "app_search": "",
      "ip_search": "",
      "domain_search": "",
      "port_search": "",
      "protocol": "All",
      "public_ip": true,
      "observation_filter": "All"
    },
    "tables": [
      {
        "id": "monitoring",
        "total_rows": 210,
        "displayed_rows": 74,
        "selected_rows": 2
      }
    ],
    "selected_monitoring_row_ids": [12, 19],
    "displayed_monitoring_row_ids": [12, 19, 20],
    "integration_module_count": 1,
    "tracked_app_count": 7,
    "enabled_tracked_app_count": 3
  },
  "payload": {
    "monitoring_active": true,
    "selected_monitoring_row_ids": [12, 19],
    "tables": [
      {
        "id": "monitoring",
        "total_rows": 210,
        "displayed_rows": 74,
        "selected_rows": 2
      }
    ]
  }
}
```

`module_background_active` относится к конкретному модулю, которому отправлен request/event. Два запущенных модуля не делят это состояние и не получают ответы на диалоги друг друга.

Host events не заменяют явные пользовательские действия. Background-подписки не стартуют при discovery/bootstrap: модуль должен сначала вернуть `start_background` из явного `ui_action`, например из header/action первого уровня в открытом overlay. Cloud upload/download и CSV import/export остаются ручными действиями в основном UI. Ответы на module dialogs маршрутизируются только модулю-владельцу диалога.

## Фильтры

Фильтры watcher-owned. Модуль не должен менять их во время набора текста. Если модуль хочет применить фильтр, он возвращает команду `set_filters` с полным `UiFiltersDto`; после этого desktop/browser UI увидят новое состояние из snapshot.

## Локальное сохранение данных модулем

Если модуль хочет сохранить полученные строки, он пишет в `storage_dir`, например:

```text
integrations/ui-entity-showcase-rust/data/module.sqlite3
```

Это portable-путь модуля. Core SQLite NetStitch не используется для module-owned таблиц.
