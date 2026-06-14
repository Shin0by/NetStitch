# Rust UI Entity Showcase

This example implements the same demo module as the C++ example, but with a Rust native library.

Important manifest points:

- `display_name_key`, `title_key`, `value_key`, `placeholder_key`, `label_key`, and `tooltip_key` are read from module-owned `locales/*.ini`;
- `icon_path = "assets/brand-rust-svgrepo-com.svg"` uses a module-owned SVG icon;
- `icon_path` renders the module-owned SVG icon across the whole module button;
- `button_color = "#c4551c"` gives the module button its Rust color;
- `ui_schema` is shared by desktop and browser shells; the module does not write HTML, CSS, or Dioxus code;
- the `browse_windows` tab demonstrates `browse_window`: the host opens a folder picker or save-target picker and writes only the selected path/status into `payload.ui_values`;
- `simulate_download` sends live `IntegrationHostEvent.event = "ui_values"` events while the blocking `ui_action` is still running and animates the large `showcase-progress` bar from `0` to `100` through `Queued`, `Processing`, and `Done`;
- `table_columns` shows column-level table sizing and text-field containment;
- `clear_button` and `commit_on_enter` show the host-owned input contract.

For local use, prefer the ready archives in `docs/module-sdk/packages/`; they already contain `module.json`, `bin/`, `locales/`, assets, and source files.
