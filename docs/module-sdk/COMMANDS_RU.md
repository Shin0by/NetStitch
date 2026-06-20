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

Ту же остановку может инициировать host-owned кнопка `Остановить` в главном header-е модуля. Эта кнопка предназначена для прекращения фоновой работы и подписок выбранного модуля, отсечения pending результатов `ui_action` и возврата пользователя на главный экран NetStitch.

Важно: текущий `native_library` transport загружает DLL/SO в процесс NetStitch. Такой in-process модуль обязан вести себя кооперативно: быстро возвращаться из `ui_action`, проверять собственные stop-флаги и не запускать неуправляемые destructive операции. Гарантированное принудительное завершение недоверенного или зависшего native-кода возможно только через отдельный module runner process, который host может завершить на уровне ОС; это отдельный isolation contour для long-running/destructive модулей, а не свойство in-process DLL.

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

Обновляет host-owned значения controls текущего модуля. Команда применяется только к UI state активного модуля и не меняет SQLite. Используется для reset-кнопок, программного выбора `tabs`, выставления `input` / `textarea` / `select` и `switch`, а также для динамического состояния action-кнопок по стабильным ключам `action.<action_id>.enabled` и `action.<action_id>.disabled`.

```json
{
  "command_type": "set_ui_values",
  "payload": {
    "values": {
      "module-tabs": "details",
      "mode": "safe",
      "enabled": true,
      "notes": "Default text",
      "action.apply_profile_export.enabled": false,
      "action.backup_profile_export.disabled": true
    }
  }
}
```

Значение `null` удаляет ключ из host-owned UI state, после чего control снова использует fallback из manifest (`value` или `checked`), а action-кнопка снова использует manifest `enabled`. `action.<id>.enabled = false` выключает action, `true` включает; `action.<id>.disabled = true` выключает action, `false` включает. Если заданы оба ключа для одного action id, `disabled` применяется последним и считается более безопасным финальным override. Disabled action-кнопки визуально отключены и не отправляют `ui_action`; правило одинаково работает для header actions и actions внутри `action_button`, tabs, nested panels и fullscreen pages в desktop и browser shell.

## Live-события во время `ui_action`

Если долгий `ui_action` должен обновлять progress/status до возврата финального response, модуль может вызвать ABI event callback с `IntegrationHostEvent.event = "ui_values"`.

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

Host применяет `payload.values` к тому же host-owned UI state текущего модуля, что и команда `set_ui_values`: обычные control-ключи равны `entity.id`; строка, число и boolean JSON primitives становятся значениями controls; `null` удаляет ключ; объекты и массивы не считаются прямым control value. `progress` получает процент как число или строку `0..100`. Action-кнопки можно менять теми же stable keys `action.<action_id>.enabled` и `action.<action_id>.disabled`, включая live-события во время долгого `ui_action`. Desktop и browser shell опрашивают эти события параллельно с blocking `ui_action` и игнорируют поздние события, если overlay закрыт, остановлен или action-token больше не актуален.

Host не маппит `download_progress` на конкретный UI id. Если модулю нужно двигать progress bar, он должен явно указать нужный ключ в `ui_values`, например `"download-progress": 42`. Это сохраняет контракт универсальным и не привязывает NetStitch к id конкретного модуля.

`browse_window`

Открывает host-owned окно выбора пути и записывает результат в `payload.ui_values[target]` текущего модуля. Модуль не создаёт собственное native/web окно: desktop shell показывает системный dialog, browser shell показывает server-side picker машины, где запущен NetStitch. Если пользователь нажал отмену, значение `target` не меняется.

```json
{
  "command_type": "browse_window",
  "payload": {
    "target": "export_path",
    "status_target": "export_status",
    "mode": "file_save",
    "title": "Save generated profile",
    "start_dir": "",
    "default_name": "netstitch-profile",
    "default_extension": "csv",
    "confirm_label": "Save",
    "selected_status": "Save target selected; file was not written",
    "overwrite_policy": "prompt",
    "can_create_directories": true,
    "filters": [
      {
        "name": "CSV files",
        "extensions": ["csv"]
      },
      {
        "name": "Text files",
        "extensions": ["txt", "conf"]
      }
    ]
  }
}
```

Поля:

- `target` - обязательный id UI-control/state-ключа модуля, куда host запишет выбранный путь строкой.
- `status_target` - опциональный id UI-control/state-ключа, куда host запишет `selected_status` после успешного выбора.
- `mode` - `folder`, `file_open` или `file_save`; по умолчанию `file_open`.
- `title` - заголовок окна; по умолчанию зависит от режима: `Choose folder`, `Choose file`, `Save file`.
- `start_dir` - стартовая папка; если пусто, host использует текущий путь из `target`, если он уже есть.
- `filters` - список групп расширений для файлов; расширения пишутся без точки, например `csv`, `txt`, `conf`. Для `folder` игнорируются.
- `default_name` - имя файла по умолчанию для `file_save`.
- `default_extension` - расширение, которое host добавит к результату `file_save`, если пользователь ввёл имя без расширения.
- `confirm_label` - подпись кнопки подтверждения в host-owned picker-е, если оболочка позволяет её менять.
- `selected_status` - опциональный текст статуса для `status_target`; удобен для label-а вида `Путь сохранения выбран; файл не записан`.
- `overwrite_policy` - `prompt` по умолчанию, `allow` или `deny`; для `file_save` управляет выбором уже существующего файла.
- `can_create_directories` - разрешает native save/open dialog создавать папки, если платформа это поддерживает; browser shell работает только с уже существующими папками runtime-хоста.

`browse_window` только выбирает путь и не создаёт, не читает и не перезаписывает файл. Любая запись остаётся отдельным явным действием модуля в его разрешённом storage/root контуре.

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
