# App Connectors

Applications can be added to NetStitch manually by executable path, through a Windows shortcut, or through an app connector.

An app connector is an editable description of a known application: its display name, possible process names, discovery rules for different operating systems, and icon. This lets NetStitch find the application automatically and show a readable name such as `Google Chrome`, `Telegram`, or `Discord` instead of only a file name.

## What Connectors Provide

- readable application names in the tracking list;
- process names for Windows, Linux, and macOS;
- discovery rules for installed applications;
- one product description across operating systems;
- separate profiles for the same executable when needed;
- portable application identity for CSV and cloud publishing.

## Where To Find It

- Interface: main window -> `Tracked apps` panel.
- Manual add without a connector: in `Tracked apps`, enter an executable path, choose a file through the add button, or choose a Windows `.lnk` shortcut.
- User connectors: portable `apps/*.app` folder next to `NetStitch.exe` or the Linux binary.
- Connector icons: portable `apps/icons/` folder.

An application can also be added without an app connector. This manual mode is useful for one-off checks or unknown programs: NetStitch will monitor the selected executable. On Windows, a `.lnk` shortcut can be selected and NetStitch will use the application it points to. Manual add does not provide the extended discovery rules or portable product identity provided by a connector.

## User Connectors

In portable builds, user-editable connectors live in the `apps/` folder as `.app` files. After editing a connector, restart NetStitch or refresh the application list.

Portable and installer packages ship a default set of `.app` files, but the files remain external runtime data. If the `apps/` folder or a specific `.app` file is missing, NetStitch does not recreate it on startup and does not enable a hidden built-in known-app list; the application can still be added manually by path.

Minimal example:

```toml
id = "my-app"
display_name = "My App"
process_names = ["my-app.exe"]
enabled = true
```

In most cases, a stable `id`, `display_name`, and executable names are enough. For more accurate discovery, add system discovery rules and platform-specific aliases.

It is best to describe a connector as cross-platform from the start: the same product may use different process names and discovery sources on Windows, Linux, and macOS, while remaining one application in NetStitch.

## `.app` Capabilities

The `.app` format is an external contract: NetStitch provides generic discovery strategies, while each connector file describes how to find a concrete application. The project code does not need a separate branch for each application.

Supported discovery rules:

- `app_paths` - Windows `App Paths` lookup by executable name.
- `known_path` - path below an environment variable such as `LOCALAPPDATA` or `ProgramFiles`; `*` is allowed in path segments.
- `fixed_path` - direct absolute path with `%ENV%` support.
- `root_paths` - search below named root sets such as `windows_drive_roots` or `windows_drive_games`, with multiple `relative_paths`. This is useful for games and portable apps installed on different drives.
- `registry_strings` - read Windows Registry string values (`REG_SZ`/`REG_EXPAND_SZ`) where the value may be either a direct executable path or an install directory.
- `store_msix` - Windows Store/MSIX package lookup.
- `macos_bundle` - macOS `.app` lookup.
- `linux_desktop` - Linux `.desktop` lookup plus `PATH` fallback.
- `linux_path` - direct command lookup through `PATH`.

If two connectors point to the same executable, NetStitch keeps them as separate applications by `connector id + path`. This supports separate profiles for one launcher and community connectors without core code changes.

The complete schema and examples live next to the shipped manifests: `resources/connectors/apps/README_RU.txt` and `resources/connectors/apps/README_EN.txt`. These files are also included in the portable `apps/` folder.
