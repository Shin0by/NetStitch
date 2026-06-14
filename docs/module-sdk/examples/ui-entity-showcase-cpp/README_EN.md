# C++ UI Entity Showcase

This example implements the same demo module as the Rust example, but with a C++ native library.

Important manifest points:

- `display_name_key`, `title_key`, `value_key`, `placeholder_key`, `label_key`, and `tooltip_key` are read from module-owned `locales/*.ini`;
- there is no `icon_path` on purpose, so the module button uses `icon_label`;
- `button_color = "#1287d8"` gives the module button its blue color;
- `ui_schema` is shared by desktop and browser shells;
- the `browse_windows` tab demonstrates `browse_window`: the host opens a folder picker or save-target picker and writes only the selected path/status into `payload.ui_values`;
- `simulate_download` sends live `IntegrationHostEvent.event = "ui_values"` events while the blocking `ui_action` is still running and animates the large `showcase-progress` bar from `0` to `100` through `Queued`, `Processing`, and `Done`;
- the module uses the same C/JSON ABI as the Rust example.

For local use, prefer the ready archives in `docs/module-sdk/packages/`; they already contain `module.json`, `bin/`, `locales/`, and source files.
