# ABI And JSON Contract

External modules use one native C ABI and exchange JSON requests/responses with the host. Extending the Module API should add JSON fields, actions, commands, or entities without changing the native ABI.

## Manifest Example

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
  "transport": "native_library",
  "library_paths": {
    "windows-x86_64": "bin/ui_entity_showcase_rust.dll",
    "linux-x86_64": "bin/libui_entity_showcase_rust.so",
    "macos-aarch64": "bin/libui_entity_showcase_rust.dylib",
    "default": "bin/ui_entity_showcase_rust.dll"
  }
}
```

## Native Entry

The host calls the module with a JSON request and expects a JSON response. The stable ABI symbols are documented by the shared model constants:

- `INTEGRATION_ABI_VERSION`;
- `INTEGRATION_ABI_CALL_EXPORT`;
- `INTEGRATION_ABI_FREE_EXPORT`.

Module implementations should treat unknown JSON fields as forward-compatible and should return clear `error`/`message` text rather than panicking across the FFI boundary.

## Request Actions

Common actions:

- `status`;
- `providers`;
- `ui_action`;
- `background_event`;
- module-specific operations declared by the module.

`ui_action` receives host context, selected monitoring rows, displayed row ids, filters, and `payload.ui_values`.

## Response

Responses may include:

- `message`;
- `severity`: `info`, `success`, `warning`, `error`;
- `refresh`;
- `commands`;
- module-specific payload.

Commands are described in `COMMANDS_EN.md`.
