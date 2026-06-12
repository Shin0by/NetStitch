# Команды host-а

Модуль возвращает команды в `IntegrationModuleUiActionResponseDto.commands`.

```json
{
  "message": "Applied filter",
  "severity": "success",
  "refresh": true,
  "commands": [
    {
      "command_type": "set_filters",
      "payload": {
        "app_search": "Firefox",
        "ip_search": "",
        "domain_search": "*.example.com",
        "port_search": "",
        "protocol": "All",
        "public_ip": true,
        "observation_filter": "All"
      }
    }
  ]
}
```

## Разрешённые команды

`set_filters`

Применяет полный `UiFiltersDto`. Host сохраняет `public_ip` в настройках и обновляет watcher-owned фильтры. Текстовые фильтры применяются только как готовое значение, не во время набора.

`start_monitoring`, `stop_monitoring`

Запускают или останавливают мониторинг.

`add_tracked_app`

Payload соответствует `AddTrackedAppRequest`: `exe_path`, `enabled`.

`set_tracked_app_enabled`

Payload соответствует `SetTrackedAppEnabledRequest`: `tracked_app_id`, `enabled`.

`set_all_tracked_apps_enabled`

Payload соответствует `SetAllTrackedAppsEnabledRequest`: `enabled`.

`delete_tracked_app`

Payload соответствует `DeleteTrackedAppRequest`: `tracked_app_id`.

`confirm_monitoring_rows`

Payload соответствует `ConfirmEndpointsRequest`: `endpoint_ids`, `confirmed`.

`delete_monitoring_rows`

Payload соответствует `DeleteObservationsRequest`: `endpoint_ids`.

`select_monitoring_rows`

Зарезервировано для UI-выделения строк. Команда не меняет SQLite.

`start_background`

Запускает module-owned фоновую задачу по явному пользовательскому действию. Команда разрешена в ответе на `ui_action`, включая action `module.open` или `open`, который host вызывает при открытии overlay первого уровня. Задача не стартует автоматически при загрузке NetStitch или обнаружении модуля.

```json
{
  "command_type": "start_background",
  "payload": {
    "subscriptions": [
      "filters.*",
      "ui.controls",
      "ui.tables",
      "monitoring.*",
      "tracked_apps.*"
    ]
  }
}
```

`stop_background`

Останавливает ранее запущенную module-owned задачу.

Ту же остановку может инициировать host-owned кнопка `Остановить` в главном header-е модуля. Эта кнопка предназначена для полного прекращения фоновой работы и подписок выбранного модуля и возвращает пользователя на главный экран NetStitch.

```json
{
  "command_type": "stop_background",
  "payload": {}
}
```

`set_module_page`

Переключает текущий overlay этого модуля на другую страницу UI-схемы. Host меняет только страницу активного модуля; другие модули и их overlay/dialog state не затрагиваются.

```json
{
  "command_type": "set_module_page",
  "payload": {
    "page": "last_rows"
  }
}
```

`set_ui_values`

Обновляет host-owned значения controls текущего модуля. Команда применяется только к UI state активного модуля и не меняет SQLite. Используется для reset-кнопок, программного выбора `tabs`, выставления `input` / `textarea` / `select` и `switch`.

```json
{
  "command_type": "set_ui_values",
  "payload": {
    "values": {
      "module-tabs": "details",
      "mode": "safe",
      "enabled": true,
      "notes": "Default text"
    }
  }
}
```

Значение `null` удаляет ключ из host-owned UI state, после чего control снова использует fallback из manifest (`value` или `checked`).

`log_event`

Пишет сообщение в `system_events`. Host задаёт `source` как отображаемое имя модуля из manifest, а severity принимает только `info`, `success`, `warning`, `error`. `core` и `system` зарезервированы для host/system событий и не допускаются как `id` или `display_name` модуля. Модуль не может удалять или изменять записи лога.

```json
{
  "command_type": "log_event",
  "payload": {
    "message": "Provider refresh completed",
    "severity": "success",
    "details": {
      "updated_rows": 42
    }
  }
}
```

`show_dialog`

Показывает стандартный диалог NetStitch от имени модуля. Диалог использует title/icon модуля из manifest, поддерживает `buttons = "ok"` и `buttons = "ok_cancel"`, а выбранный результат логируется host-ом как module event.

```json
{
  "command_type": "show_dialog",
  "payload": {
    "dialog_id": "apply-preview",
    "buttons": "ok_cancel",
    "message": "This will write module-owned generated files."
  }
}
```

Ответ пользователя приходит в модуль как отдельный `background_event` с `event_type = "ui.dialog_result"` и payload `dialog_id` + `result` (`ok` или `cancel`). Host пишет результат в `system_events` с source выбранного модуля. Ответ получает только тот модуль, который создал диалог.

## Запрещено автоматизировать

Модуль не получает команд для cloud upload/download и CSV import/export. Если ему нужны строки, он получает их через `monitoring_rows` и может сохранить результат в собственной папке `data/`.
