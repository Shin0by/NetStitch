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
- `browse_window` - open a host-owned file/folder/save picker and write the selected path to a module UI value;
- `log_event` - write a module event to `system_events`;
- `show_dialog` - show a standard NetStitch module dialog with `buttons = "ok"` or `buttons = "ok_cancel"`.

`core` and `system` are reserved event sources and cannot be used as module id/display name. Severity is one of `info`, `success`, `warning`, `error`.

The host-owned Stop button stops background subscriptions for the selected module, invalidates pending `ui_action` results, and returns the user to the main NetStitch shell.

Important: the current `native_library` transport loads a DLL/SO into the NetStitch process. In-process modules must be cooperative: return from `ui_action`, check their own stop flags, and avoid unmanaged destructive work. Guaranteed forced termination of untrusted or hung native code requires a separate module runner process that the host can kill at the OS level; that is the required isolation contour for long-running or destructive modules, not a property of in-process DLL calls.

`browse_window` uses the host UI instead of a module-created window. Desktop NetStitch opens a native dialog and the browser shell opens the server-side picker on the machine where NetStitch is running. Cancellation leaves the target value unchanged.

## Live Events During `ui_action`

When a long `ui_action` needs to update progress or status before the final response returns, the module may call the ABI event callback with `IntegrationHostEvent.event = "ui_values"`.

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

The host applies `payload.values` to the same host-owned UI state as `set_ui_values`: keys match `entity.id`, `null` removes a key, and `progress` entities receive `0..100` as a number or string. The desktop and browser shells poll these events while the blocking `ui_action` is still running and ignore late events after the overlay was closed, stopped, or the action token is stale.

The host does not map `download_progress` to a specific UI id. If a module wants to move a progress bar, it must explicitly name the target key in `ui_values`, for example `"download-progress": 42`. This keeps the contract generic instead of binding NetStitch to one module's ids.

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

Fields:

- `target` is required and names the module UI value key that receives the selected path string.
- `status_target` optionally names another module UI value key that receives `selected_status` after a successful selection.
- `mode` is `folder`, `file_open`, or `file_save`; default is `file_open`.
- `title` defaults to `Choose folder`, `Choose file`, or `Save file`.
- `start_dir` is the initial directory; when omitted, the host falls back to the current `target` value if present.
- `filters` groups file extensions without leading dots. Folder mode ignores filters.
- `default_name` is the proposed file name for `file_save`.
- `default_extension` is appended to a `file_save` result when the selected name has no extension.
- `confirm_label` is used by host pickers that support custom confirmation text.
- `selected_status` is an optional status string for `status_target`, useful for labels such as `Save target selected; file was not written`.
- `overwrite_policy` is `prompt` by default, or `allow` / `deny` for existing `file_save` targets.
- `can_create_directories` lets native dialogs create folders when the platform supports it; the browser shell uses existing runtime-host directories.

`browse_window` selects a path only. It does not create, read, overwrite, import, export, or upload files; the module must perform any write as a separate explicit action inside its allowed storage/root boundary.

Modules do not get commands for cloud upload/download or CSV import/export. If a module needs data, it receives local monitoring rows through host context and stores results inside its own `data/` folder.
