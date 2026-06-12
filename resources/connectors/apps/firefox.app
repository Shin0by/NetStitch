version = 1
enabled = true
id = "firefox"
display_name = "Mozilla Firefox"
icon_key = "firefox"
process_names = ["firefox.exe", "Firefox", "firefox", "firefox-esr"]
manual_only = false

[[process_aliases]]
os = "windows"
name = "firefox.exe"

[[process_aliases]]
os = "macos"
name = "Firefox"

[[process_aliases]]
os = "linux"
name = "firefox"

[[process_aliases]]
os = "linux"
name = "firefox-esr"

[[discovery_sources]]
os = "windows"
kind = "app_paths"
detail = "firefox.exe"

[[discovery_sources]]
os = "windows"
kind = "known_path"
detail = "ProgramFiles/LOCALAPPDATA Mozilla Firefox"

[[discovery_sources]]
os = "macos"
kind = "app_bundle"
detail = "/Applications/Firefox.app"

[[discovery_sources]]
os = "linux"
kind = "desktop_entry"
detail = "firefox.desktop/firefox-esr.desktop/org.mozilla.firefox.desktop"

[[discovery]]
os = "windows"
kind = "app_paths"
value = "firefox.exe"

[[discovery]]
os = "windows"
kind = "known_path"
root_env = "ProgramFiles"
relative_path = "Mozilla Firefox\firefox.exe"

[[discovery]]
os = "windows"
kind = "known_path"
root_env = "ProgramFiles(x86)"
relative_path = "Mozilla Firefox\firefox.exe"

[[discovery]]
os = "windows"
kind = "store_msix"
package_prefixes = ["Mozilla.Firefox_"]
exe_names = ["firefox.exe"]

[[discovery]]
os = "macos"
kind = "macos_bundle"
bundle_names = ["Firefox"]
executable_names = ["firefox"]

[[discovery]]
os = "linux"
kind = "linux_desktop"
desktop_ids = ["firefox.desktop", "firefox-esr.desktop", "org.mozilla.firefox.desktop"]
executable_names = ["firefox", "firefox-esr"]
