# Host Context And Table Data

`ui_action` receives `IntegrationModuleUiActionRequestDto`.

```json
{
  "action_id": "say_hello",
  "context": {
    "app_version": "1.1.1.456",
    "language_code": "en-en",
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

`monitoring_rows` contains selected monitoring rows. If nothing is selected, the list is empty, but displayed row ids and table counters remain available.

## Live `ui_action` Events

During a long `ui_action`, a module may send `IntegrationHostEvent` values through the ABI callback. The public UI-state event is:

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

The host applies these values to the active module in the same way as a final `set_ui_values` command, but before the blocking `ui_action` finishes. Late events are ignored after the host-owned Stop/Close flow invalidates the action. `download_progress` remains an internal provider-download event and is not mapped by the host to a module-specific progress id.

## Host Events

After a module explicitly starts background mode through `start_background`, the host calls the same native C/JSON entrypoint with `action = "background_event"`. Payload shape is `IntegrationModuleBackgroundEventDto`: `event_type`, `created_at_ms`, `context`, and event-specific `payload`.

Common event types:

- `filters.changed`;
- `monitoring.started` / `monitoring.stopped`;
- `monitoring.rows_changed` / `monitoring.rows_deleted` / `monitoring.rows_added` / `monitoring.selection_changed`;
- `tracked_apps.changed`;
- `ui.dialog_result`.

Background subscriptions do not start during discovery/bootstrap. The module must first return `start_background` from an explicit user action. Cloud upload/download and CSV import/export remain manual actions in the main UI.

## Storage

Module-owned data belongs in:

```text
integrations/<module-id>/data/module.sqlite3
```

The main NetStitch SQLite database is not used for module-owned tables or settings.
