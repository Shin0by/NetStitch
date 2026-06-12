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

Minimal example:

```toml
id = "my-app"
display_name = "My App"
process_names = ["my-app.exe"]
enabled = true
```

In most cases, a stable `id`, `display_name`, and executable names are enough. For more accurate discovery, add system discovery rules and platform-specific aliases.

It is best to describe a connector as cross-platform from the start: the same product may use different process names and discovery sources on Windows, Linux, and macOS, while remaining one application in NetStitch.
