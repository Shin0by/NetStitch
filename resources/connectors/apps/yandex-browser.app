version = 1
enabled = true
id = "yandex_browser"
display_name = "Yandex Browser"
icon_key = "yandex_browser"
process_names = ["browser.exe", "Yandex", "yandex-browser", "yandex-browser-stable"]
manual_only = false

[[process_aliases]]
os = "windows"
name = "browser.exe"

[[process_aliases]]
os = "macos"
name = "Yandex"

[[process_aliases]]
os = "linux"
name = "yandex-browser"

[[process_aliases]]
os = "linux"
name = "yandex-browser-stable"

[[discovery_sources]]
os = "windows"
kind = "known_path"
detail = "%LOCALAPPDATA%\Yandex\YandexBrowser\Application\browser.exe"

[[discovery_sources]]
os = "macos"
kind = "app_bundle"
detail = "/Applications/Yandex.app"

[[discovery_sources]]
os = "linux"
kind = "desktop_entry"
detail = "yandex-browser.desktop/yandex-browser-stable.desktop/ru.yandex.desktop.browser.desktop"

[[discovery]]
os = "windows"
kind = "known_path"
root_env = "LOCALAPPDATA"
relative_path = "Yandex\YandexBrowser\Application\browser.exe"

[[discovery]]
os = "windows"
kind = "known_path"
root_env = "ProgramFiles"
relative_path = "Yandex\YandexBrowser\Application\browser.exe"

[[discovery]]
os = "macos"
kind = "macos_bundle"
bundle_names = ["Yandex"]
executable_names = ["Yandex"]

[[discovery]]
os = "linux"
kind = "linux_desktop"
desktop_ids = ["yandex-browser.desktop", "yandex-browser-stable.desktop", "ru.yandex.Browser.desktop", "ru.yandex.desktop.browser.desktop"]
executable_names = ["yandex-browser", "yandex-browser-stable"]
