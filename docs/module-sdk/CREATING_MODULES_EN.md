# Creating Modules

## Folder

```text
integrations/
  my-module/
    module.json
    locales/
      en-en.ini
      ru-ru.ini
    bin/
      my_module.dll
      libmy_module.so
    assets/
      icon.svg
    data/
```

## Manifest

Minimum fields:

- `schema = "netstitch.integration.module.v1"`;
- `id` - stable lowercase module folder id;
- `display_name` - name shown in the `Modules` panel;
- `tooltip`, `icon_label`, `button_color`, `icon_path` - module button appearance; when `icon_path` is set, NetStitch renders the module-owned icon across the whole button;
- `display_name_key`, `tooltip_key`, `title_key`, `value_key`, `placeholder_key`, `label_key` - keys from module-owned `locales/*.ini`;
- `header_actions` - square action buttons in the module header;
- `ui_schema` - declarative UI entities;
- `transport = "native_library"`;
- `library_paths` - platform library paths inside the module folder.

```json
{
  "schema": "netstitch.integration.module.v1",
  "id": "my-module",
  "display_name": "My Module",
  "display_name_key": "my_module.module.display_name",
  "tooltip": "Example module",
  "tooltip_key": "my_module.module.tooltip",
  "icon_label": "M",
  "icon_path": "assets/icon.svg",
  "button_color": "#1287d8",
  "transport": "native_library",
  "library_paths": {
    "windows-x86_64": "bin/my_module.dll",
    "linux-x86_64": "bin/libmy_module.so",
    "macos-aarch64": "bin/libmy_module.dylib",
    "default": "bin/my_module.dll"
  }
}
```

`module.json` is plain JSON, so comments belong in README files or source comments.

## Localization

Use English fallback text directly in `module.json` and put localized strings into module-owned locale files:

```ini
[strings]
my_module.module.display_name=My Module
my_module.entity.note.title=Note
my_module.action.apply.label=Apply
```

Module localization must stay in the module folder. Do not add module strings to `resources/language/*` of the main application.

## Build

Build a native shared library into `bin/` and export the stable C/JSON ABI described in `REFERENCE_EN.md`. Windows and Linux use the same manifest and ABI; only the library file name changes.

If you change tracked SDK examples, rebuild ready-to-install archives:

```powershell
scripts\package_module_sdk_examples.ps1
```
