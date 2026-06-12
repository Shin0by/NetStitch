version = 1
enabled = true
id = "whatsapp"
display_name = "WhatsApp"
icon_key = "whatsapp"
process_names = ["WhatsApp.exe", "WhatsApp", "WhatsApp.Root.exe", "whatsapp", "whatsapp-for-linux"]
manual_only = false

[[process_aliases]]
os = "windows"
name = "WhatsApp.exe"

[[process_aliases]]
os = "windows"
name = "WhatsApp.Root.exe"

[[process_aliases]]
os = "macos"
name = "WhatsApp"

[[process_aliases]]
os = "linux"
name = "whatsapp"

[[process_aliases]]
os = "linux"
name = "whatsapp-for-linux"

[[discovery_sources]]
os = "windows"
kind = "store_msix"
detail = "5319275A.WhatsAppDesktop_*"

[[discovery_sources]]
os = "windows"
kind = "known_path"
detail = "%LOCALAPPDATA%\WhatsApp\WhatsApp.exe"

[[discovery_sources]]
os = "macos"
kind = "app_bundle"
detail = "/Applications/WhatsApp.app"

[[discovery_sources]]
os = "linux"
kind = "desktop_entry"
detail = "whatsapp.desktop/whatsapp-for-linux.desktop/com.whatsapp.WhatsApp.desktop/com.github.eneshecan.WhatsAppForLinux.desktop"

[[discovery]]
os = "windows"
kind = "store_msix"
package_prefixes = ["5319275A.WhatsAppDesktop_", "WhatsApp.WhatsApp_"]
exe_names = ["WhatsApp.Root.exe", "WhatsApp.exe"]

[[discovery]]
os = "windows"
kind = "known_path"
root_env = "LOCALAPPDATA"
relative_path = "WhatsApp\WhatsApp.exe"

[[discovery]]
os = "macos"
kind = "macos_bundle"
bundle_names = ["WhatsApp"]
executable_names = ["WhatsApp"]

[[discovery]]
os = "linux"
kind = "linux_desktop"
desktop_ids = ["whatsapp.desktop", "whatsapp-for-linux.desktop", "com.whatsapp.WhatsApp.desktop", "com.github.eneshecan.WhatsAppForLinux.desktop"]
executable_names = ["whatsapp", "whatsapp-for-linux"]
