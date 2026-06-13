# NetStitch Module SDK

[Русский](README_RU.md)

This section explains how to build external native modules for NetStitch. A module lives next to the portable app, is loaded as a shared library, and talks to the host through the stable C ABI plus JSON payloads. Module API extensions add host events, background tasks, standard dialogs, logging, and UI entities without changing the native ABI: Windows `.dll`, Linux `.so`, and macOS `.dylib` use the same entrypoint.

Modules do not ship their own desktop or web UI. The author describes one declarative `ui_schema` in `module.json`, returns host commands from native action handlers, and NetStitch renders the same controls in the desktop and browser shells.

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

Modules cannot automate cloud upload/download or CSV import/export. Those remain explicit user actions in the main UI.

## Reference

- [Creating A Module](CREATING_MODULES_EN.md)
- [Examples And Packages](EXAMPLES_EN.md)
- [ABI And JSON Contract](REFERENCE_EN.md)
- [UI Entities](UI_ENTITIES_EN.md)
- [Host Context](HOST_CONTEXT_EN.md)
- [Host Commands](COMMANDS_EN.md)
- [UI Entity Map](assets/ui-entities.svg)
