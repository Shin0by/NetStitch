# NetStitch Module SDK

[Русский](README_RU.md)

This section explains how to build external native modules for NetStitch. A module lives next to the portable app, is loaded as a shared library, and talks to the host through the stable C ABI plus JSON payloads. Module API extensions add host events, background tasks, standard dialogs, logging, and UI entities without changing the native ABI: Windows `.dll`, Linux `.so`, and macOS `.dylib` use the same entrypoint.

Modules do not ship their own desktop or web UI. One declarative `ui_schema` in `module.json` and host commands from native action handlers are rendered by NetStitch in the same way in the desktop and browser shells: panels, subpanels, two-layer `nested_subpanel` containers, tabs, header/footer, tables, value/status labels, text input/textarea with optional `clear_button`, Enter commits through `commit_on_enter`, dropdown/select, switch, buttons, progress bars with optional `progress_stages`, and standard dialogs. Multiple `action_button` actions default to one horizontal `button_layout: "row"`; entity `align` aligns the whole row, and `button_layout: "column"` explicitly opts into a vertical list. Current control values, including active `tabs`, are passed in `payload.ui_values`; during a long `ui_action`, a module can update those values live with `IntegrationHostEvent.event = "ui_values"`.

UI entities have centralized defaults: regular entities occupy the available width, panel/grid/tab containers grow vertically from their content by default, and a `footer` entity becomes the left slot of the standard module overlay footer. The host adds the `Back` button on the right; it is hidden only by an explicit `hide_host_back_button: true` on the active `footer` entity.

## Quick Start

1. Take an archive from `docs/module-sdk/packages/`.
2. Unpack it next to the portable app so the path becomes `integrations/<module-id>/module.json`.
3. Restart NetStitch; the module button appears in the `Modules` panel.

For source development:

1. Copy one of the examples from `docs/module-sdk/examples/`.
2. Update `module.json`: `id`, display name, UI schema, icon options, and `library_paths`.
3. Build the shared library into `bin/` for the target platform.
4. If you changed a tracked example, refresh archives with `scripts\package_module_sdk_examples.ps1`.
5. Install the module folder into `integrations/` and restart NetStitch.

## Runtime Layout

```text
integrations/
  ui-entity-showcase-rust/
    module.json
    locales/
      en-en.ini
      ru-ru.ini
    bin/
      ui_entity_showcase_rust.dll
      libui_entity_showcase_rust.so
      libui_entity_showcase_rust.dylib
    assets/
      icon.svg
    data/
      module.sqlite3
```

`data/` belongs to the module. The main NetStitch SQLite database does not store module-owned runtime data. `locales/` also belongs to the module: module strings live in the module's own `locales/en-en.ini` and `locales/ru-ru.ini`, not in the global app language files.

## Boundaries

The module receives local working context only: app version, UI language, monitoring state, filters, table counters, selected/displayed monitoring row ids, selected monitoring rows, and tracked-app counts. It does not receive cloud credentials, Google metadata, provider tokens, user secrets, or direct access to the main SQLite database.

Background tasks are not autostarted. A module may subscribe to host events only by returning `start_background` from an explicit user `ui_action`, for example a header action or a declared `module.open` action.

Modules can use `browse_window` to ask the host to open a folder/open-file/save-file picker and write the selected path into module UI state. Modules should not create their own desktop/web picker windows.

Modules cannot automate cloud upload/download or CSV import/export. Those remain explicit user actions in the main UI.

## Reference

- [Creating A Module](CREATING_MODULES_EN.md)
- [Examples And Packages](EXAMPLES_EN.md)
- [ABI And JSON Contract](REFERENCE_EN.md)
- [UI Entities](UI_ENTITIES_EN.md)
- [Host Context](HOST_CONTEXT_EN.md)
- [Host Commands](COMMANDS_EN.md)
- [UI Entity Map](assets/ui-entities.svg)
