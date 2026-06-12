version = 1
enabled = true
id = "telegram"
display_name = "Telegram"
icon_key = "telegram"
process_names = ["Telegram.exe", "Telegram", "telegram-desktop"]
manual_only = false

[[process_aliases]]
os = "windows"
name = "Telegram.exe"

[[process_aliases]]
os = "macos"
name = "Telegram"

[[process_aliases]]
os = "linux"
name = "telegram-desktop"

[[process_aliases]]
os = "linux"
name = "Telegram"

[[discovery_sources]]
os = "windows"
kind = "app_paths"
detail = "Telegram.exe"

[[discovery_sources]]
os = "windows"
kind = "known_path"
detail = "%APPDATA%\Telegram Desktop\Telegram.exe"

[[discovery_sources]]
os = "macos"
kind = "app_bundle"
detail = "/Applications/Telegram.app"

[[discovery_sources]]
os = "linux"
kind = "desktop_entry"
detail = "org.telegram.desktop.desktop/telegramdesktop.desktop/telegram-desktop.desktop"

[[discovery]]
os = "windows"
kind = "app_paths"
value = "Telegram.exe"

[[discovery]]
os = "windows"
kind = "known_path"
root_env = "APPDATA"
relative_path = "Telegram Desktop\Telegram.exe"

[[discovery]]
os = "windows"
kind = "store_msix"
package_prefixes = ["TelegramMessengerLLP.TelegramDesktop_"]
exe_names = ["Telegram.exe"]

[[discovery]]
os = "macos"
kind = "macos_bundle"
bundle_names = ["Telegram"]
executable_names = ["Telegram"]

[[discovery]]
os = "linux"
kind = "linux_desktop"
desktop_ids = ["org.telegram.desktop.desktop", "telegramdesktop.desktop", "telegram-desktop.desktop"]
executable_names = ["telegram-desktop", "Telegram"]
