version = 1
enabled = true
id = "discord"
display_name = "Discord"
icon_key = "discord"
process_names = ["Discord.exe", "Discord", "discord"]
manual_only = false

[[process_aliases]]
os = "windows"
name = "Discord.exe"

[[process_aliases]]
os = "macos"
name = "Discord"

[[process_aliases]]
os = "linux"
name = "Discord"

[[process_aliases]]
os = "linux"
name = "discord"

[[discovery_sources]]
os = "windows"
kind = "app_paths"
detail = "Discord.exe"

[[discovery_sources]]
os = "windows"
kind = "known_path"
detail = "%LOCALAPPDATA%\Discord\app-*\Discord.exe"

[[discovery_sources]]
os = "macos"
kind = "app_bundle"
detail = "/Applications/Discord.app"

[[discovery_sources]]
os = "linux"
kind = "desktop_entry"
detail = "discord.desktop/com.discordapp.Discord.desktop"

[[discovery]]
os = "windows"
kind = "app_paths"
value = "Discord.exe"

[[discovery]]
os = "windows"
kind = "known_path"
root_env = "LOCALAPPDATA"
relative_path = "Discord\app-*\Discord.exe"

[[discovery]]
os = "macos"
kind = "macos_bundle"
bundle_names = ["Discord"]
executable_names = ["Discord"]

[[discovery]]
os = "linux"
kind = "linux_desktop"
desktop_ids = ["discord.desktop", "com.discordapp.Discord.desktop"]
executable_names = ["discord", "Discord"]
