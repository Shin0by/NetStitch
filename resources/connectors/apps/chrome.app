version = 1
enabled = true
id = "chrome"
display_name = "Google Chrome"
icon_key = "chrome"
process_names = ["chrome.exe", "Google Chrome", "google-chrome", "google-chrome-stable", "chrome"]
manual_only = false

[[process_aliases]]
os = "windows"
name = "chrome.exe"

[[process_aliases]]
os = "macos"
name = "Google Chrome"

[[process_aliases]]
os = "linux"
name = "google-chrome"

[[process_aliases]]
os = "linux"
name = "google-chrome-stable"

[[discovery_sources]]
os = "windows"
kind = "app_paths"
detail = "chrome.exe"

[[discovery_sources]]
os = "windows"
kind = "known_path"
detail = "ProgramFiles/LOCALAPPDATA Google Chrome"

[[discovery_sources]]
os = "macos"
kind = "app_bundle"
detail = "/Applications/Google Chrome.app"

[[discovery_sources]]
os = "linux"
kind = "desktop_entry"
detail = "google-chrome*.desktop"

[[discovery]]
os = "windows"
kind = "app_paths"
value = "chrome.exe"

[[discovery]]
os = "windows"
kind = "known_path"
root_env = "ProgramFiles"
relative_path = "Google\Chrome\Application\chrome.exe"

[[discovery]]
os = "windows"
kind = "known_path"
root_env = "ProgramFiles(x86)"
relative_path = "Google\Chrome\Application\chrome.exe"

[[discovery]]
os = "windows"
kind = "known_path"
root_env = "LOCALAPPDATA"
relative_path = "Google\Chrome\Application\chrome.exe"

[[discovery]]
os = "macos"
kind = "macos_bundle"
bundle_names = ["Google Chrome"]
executable_names = ["Google Chrome"]

[[discovery]]
os = "linux"
kind = "linux_desktop"
desktop_ids = ["google-chrome.desktop", "google-chrome-stable.desktop", "com.google.Chrome.desktop"]
executable_names = ["google-chrome", "google-chrome-stable", "chrome"]
