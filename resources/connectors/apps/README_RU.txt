NetStitch Runtime Connector Apps

Файлы *.app в этой папке описывают known-app коннекторы, которые portable-сборка кладёт рядом с NetStitch.exe в apps\.

Если папка apps\ существует рядом с exe, NetStitch использует её как пользовательский набор коннекторов. Чтобы удалить коннектор, удалите его *.app или поставьте enabled = false. Чтобы добавить новый, создайте новый app-файл по этой схеме.

Минимальный интерфейс:

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

Поддерживаемые поля:
- version: сейчас всегда 1; нужно для будущих миграций формата.
- enabled: true или false; выключенный файл остаётся на месте, но не участвует в discovery.
- id: стабильный системный идентификатор коннектора, латиница/цифры/_/-.
- display_name: имя приложения в UI.
- icon_key: стабильный ключ иконки. NetStitch ищет `apps/icons/<icon_key>.svg` или `apps/icons/<icon_key>.png`; если файла нет, для известных ключей (discord, telegram, whatsapp, chrome, firefox, yandex_browser) используется shipped connector-иконка.
- process_names: известные имена процессов для разных платформ.
- manual_only: если true, коннектор только документирует manual-flow и не делает discovery.

Discovery rules:
- kind = "app_paths": Windows App Paths registry lookup; поле value = "App.exe".
- kind = "known_path": путь относительно env-переменной; поля root_env, relative_path; в сегментах разрешён *. root_env читает обычную переменную окружения по имени, поэтому можно использовать стандартные переменные Windows/Linux/macOS или свою пользовательскую переменную.
- kind = "fixed_path": абсолютный путь; поле value или path, поддерживает %ENV%.
- kind = "root_paths": поиск относительно набора корней; поля root_aliases и relative_paths. Поддерживаемые root_aliases: windows_drive_roots, windows_drive_games, drive_roots, drive_games, games_dirs, home, linux_mount_roots. В relative_paths разрешён *.
- kind = "registry_strings": Windows registry lookup по строковым значениям; поля root_key, subkey, value_name или value_names, relative_path/relative_paths и executable_names. Читаются только строковые REG_SZ/REG_EXPAND_SZ значения. Если значение является файлом, оно используется напрямую; если это папка, NetStitch применяет relative_paths или executable_names.
- kind = "store_msix": Windows Store/MSIX; поля package_prefixes, exe_names.
- kind = "macos_bundle": macOS .app; поля bundle_names, executable_names.
- kind = "linux_desktop": Linux .desktop + PATH fallback; поля desktop_ids, executable_names.
- kind = "linux_path": Linux PATH lookup; поле executable_names.

Пример поиска игры по корням дисков и папкам Games:

[[discovery]]
os = "windows"
kind = "root_paths"
root_aliases = ["windows_drive_roots", "windows_drive_games"]
relative_paths = ["MyGame\\MyGame.exe", "SteamLibrary\\steamapps\\common\\MyGame\\MyGame.exe"]

Поддерживаемые root_aliases:
- windows_drive_roots: C:\, D:\, E:\ ...
- windows_drive_games: C:\Games, D:\Games, E:\Games ...
- drive_roots: alias для windows_drive_roots.
- drive_games: alias для windows_drive_games.
- games_dirs: alias для windows_drive_games.
- home: HOME или USERPROFILE.
- linux_mount_roots: /mnt и /media.

Частые Windows env-переменные для known_path:
- ProgramFiles: C:\Program Files
- ProgramFiles(x86): C:\Program Files (x86)
- LOCALAPPDATA: C:\Users\<user>\AppData\Local
- APPDATA: C:\Users\<user>\AppData\Roaming
- USERPROFILE: C:\Users\<user>
- PUBLIC: C:\Users\Public

Примеры known_path:

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

Пример поиска через строковые значения реестра:

[[discovery]]
os = "windows"
kind = "registry_strings"
root_key = "HKCU"
subkey = "Software\\Vendor\\MyGame"
value_names = ["InstallLocation", "Path"]
relative_paths = ["MyGame.exe", "Bin\\MyGame.exe"]

os может быть windows, macos или linux. Если os не указан у discovery rule, правило считается общим.
Если discovery rule не указан, NetStitch попробует использовать discovery_sources как fallback для app_paths, fixed_path и known_path с прямым путём.
Для прямого абсолютного пути можно писать kind = "fixed_path" или kind = "known_path" с value/path.
Описание Linux и macOS не обязательно: отсутствующие платформы просто не участвуют в поиске.
Если несколько коннекторов указывают на один executable, NetStitch сохраняет их как разные приложения по паре connector id + path. Это позволяет делать разные профили одного launcher-а без правок в коде проекта.
