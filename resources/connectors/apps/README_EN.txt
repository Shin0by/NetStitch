NetStitch Runtime Connector Apps

The *.app files in this folder describe known-app connectors that the portable build places next to NetStitch.exe under apps\.

If an apps\ folder exists next to the exe, NetStitch uses it as the user-managed connector set. To remove a connector, delete its *.app file or set enabled = false. To add a new one, create a new app file following the schema below.

Minimal interface:

version = 1
enabled = true
id = "my_app"
display_name = "My App"
icon_key = "my_app"
process_names = ["MyApp.exe", "my-app"]
manual_only = false

[[process_aliases]]
os = "windows"
name = "MyApp.exe"

[[discovery_sources]]
os = "windows"
kind = "known_path"
detail = "%LOCALAPPDATA%\MyApp\MyApp.exe"

[[discovery]]
os = "windows"
kind = "known_path"
root_env = "LOCALAPPDATA"
relative_path = "MyApp\MyApp.exe"

Supported fields:
- version: currently always 1; reserved for future format migrations.
- enabled: true or false; a disabled file stays on disk but is ignored by discovery.
- id: stable system connector identifier using letters, digits, _ or -.
- display_name: app name shown in UI.
- icon_key: stable icon key. NetStitch looks for `apps/icons/<icon_key>.svg` or `apps/icons/<icon_key>.png`; if no file exists, known keys (discord, telegram, whatsapp, chrome, firefox, yandex_browser) use the shipped connector icon.
- process_names: known process names across platforms.
- manual_only: if true, the connector only documents a manual flow and does not perform discovery.

Discovery rules:
- kind = "app_paths": Windows App Paths registry lookup; uses value = "App.exe".
- kind = "known_path": path relative to an environment variable; uses root_env and relative_path; * is allowed in path segments. root_env reads a normal environment variable by name, so standard Windows/Linux/macOS variables and custom user variables are supported.
- kind = "fixed_path": absolute path; uses value or path and supports %ENV%.
- kind = "root_paths": searches below a named root set; uses root_aliases and relative_paths. Supported root_aliases: windows_drive_roots, windows_drive_games, drive_roots, drive_games, games_dirs, home, linux_mount_roots. * is allowed in relative_paths.
- kind = "registry_strings": Windows registry lookup for string values; uses root_key, subkey, value_name or value_names, relative_path/relative_paths, and executable_names. Only string REG_SZ/REG_EXPAND_SZ values are read. If a value is a file, it is used directly; if it is a directory, NetStitch applies relative_paths or executable_names.
- kind = "store_msix": Windows Store/MSIX; uses package_prefixes and exe_names.
- kind = "macos_bundle": macOS .app; uses bundle_names and executable_names.
- kind = "linux_desktop": Linux .desktop plus PATH fallback; uses desktop_ids and executable_names.
- kind = "linux_path": Linux PATH lookup; uses executable_names.

Example: find a game under drive roots and Games folders:

[[discovery]]
os = "windows"
kind = "root_paths"
root_aliases = ["windows_drive_roots", "windows_drive_games"]
relative_paths = ["MyGame\\MyGame.exe", "SteamLibrary\\steamapps\\common\\MyGame\\MyGame.exe"]

Supported root_aliases:
- windows_drive_roots: C:\, D:\, E:\ ...
- windows_drive_games: C:\Games, D:\Games, E:\Games ...
- drive_roots: alias for windows_drive_roots.
- drive_games: alias for windows_drive_games.
- games_dirs: alias for windows_drive_games.
- home: HOME or USERPROFILE.
- linux_mount_roots: /mnt and /media.

Common Windows environment variables for known_path:
- ProgramFiles: C:\Program Files
- ProgramFiles(x86): C:\Program Files (x86)
- LOCALAPPDATA: C:\Users\<user>\AppData\Local
- APPDATA: C:\Users\<user>\AppData\Roaming
- USERPROFILE: C:\Users\<user>
- PUBLIC: C:\Users\Public

known_path examples:

[[discovery]]
os = "windows"
kind = "known_path"
root_env = "ProgramFiles"
relative_path = "My Studio\\My Game\\MyGame.exe"

[[discovery]]
os = "windows"
kind = "known_path"
root_env = "USERPROFILE"
relative_path = "Desktop\\My Game\\MyGame.exe"

[[discovery]]
os = "windows"
kind = "known_path"
root_env = "MY_CUSTOM_GAME_ROOT"
relative_path = "My Game\\MyGame.exe"

Example: find an app through registry string values:

[[discovery]]
os = "windows"
kind = "registry_strings"
root_key = "HKCU"
subkey = "Software\\Vendor\\MyGame"
value_names = ["InstallLocation", "Path"]
relative_paths = ["MyGame.exe", "Bin\\MyGame.exe"]

os may be windows, macos, or linux. If os is omitted on a discovery rule, the rule is treated as shared.
If no discovery rule is provided, NetStitch tries to derive one from discovery_sources for app_paths, fixed_path, and known_path entries with direct paths.
For direct absolute paths, kind = "fixed_path" or kind = "known_path" with value/path are both accepted.
Linux and macOS descriptions are optional; missing platforms are simply skipped during discovery.
If multiple connectors point to the same executable, NetStitch keeps them as separate applications by connector id + path. This supports separate profiles for the same launcher without project code changes.
