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
- kind = "known_path": путь относительно env-переменной; поля root_env, relative_path; в сегментах разрешён *.
- kind = "fixed_path": абсолютный путь; поле value или path, поддерживает %ENV%.
- kind = "store_msix": Windows Store/MSIX; поля package_prefixes, exe_names.
- kind = "macos_bundle": macOS .app; поля bundle_names, executable_names.
- kind = "linux_desktop": Linux .desktop + PATH fallback; поля desktop_ids, executable_names.
- kind = "linux_path": Linux PATH lookup; поле executable_names.

os может быть windows, macos или linux. Если os не указан у discovery rule, правило считается общим.
Если discovery rule не указан, NetStitch попробует использовать discovery_sources как fallback для app_paths, fixed_path и known_path с прямым путём.
Для прямого абсолютного пути можно писать kind = "fixed_path" или kind = "known_path" с value/path.
Описание Linux и macOS не обязательно: отсутствующие платформы просто не участвуют в поиске.
