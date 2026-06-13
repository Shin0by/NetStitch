# Host Commands

Modules return commands in `IntegrationModuleUiActionResponseDto.commands`.

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

Allowed command types:

- `set_filters` - apply a complete `UiFiltersDto`; text filters are committed values, not per-keystroke updates;
- `start_monitoring` / `stop_monitoring` - start or stop monitoring;
- `add_tracked_app` - add a local executable path;
- `set_tracked_app_enabled` - enable or disable one tracked app by id;
- `set_all_tracked_apps_enabled` - change the `Enable all` overlay;
- `delete_tracked_app` - remove a tracked app from the active list;
- `confirm_monitoring_rows` - confirm or unconfirm monitoring rows;
- `delete_monitoring_rows` - delete monitoring rows;
- `select_monitoring_rows` - reserved UI selection command;
- `start_background` / `stop_background` - manage module-owned background work after an explicit user action;
- `set_module_page` - switch the active module overlay page;
- `set_ui_values` - set host-owned control values and active `tabs`;
- `log_event` - write a module event to `system_events`;
- `show_dialog` - show a standard NetStitch module dialog with `buttons = "ok"` or `buttons = "ok_cancel"`.

`core` and `system` are reserved event sources and cannot be used as module id/display name. Severity is one of `info`, `success`, `warning`, `error`.

Modules do not get commands for cloud upload/download or CSV import/export. If a module needs data, it receives local monitoring rows through host context and stores results inside its own `data/` folder.
