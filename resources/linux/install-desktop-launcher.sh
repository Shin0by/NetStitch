#!/usr/bin/env bash
# Installs a per-user NetStitch desktop launcher and icon from the portable folder.
set -euo pipefail

portable_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
app_path="$portable_dir/NetStitch"
icon_source="$portable_dir/resources/shin0by.png"

if [[ ! -x "$app_path" ]]; then
  echo "NetStitch executable is missing or not executable: $app_path" >&2
  exit 1
fi
if [[ ! -f "$icon_source" ]]; then
  echo "NetStitch icon is missing: $icon_source" >&2
  exit 1
fi

desktop_dir="${HOME}/Desktop"
if command -v xdg-user-dir >/dev/null 2>&1; then
  desktop_dir="$(xdg-user-dir DESKTOP 2>/dev/null || printf '%s\n' "$desktop_dir")"
fi

applications_dir="${HOME}/.local/share/applications"
icons_dir="${HOME}/.local/share/icons/hicolor/256x256/apps"
mkdir -p "$applications_dir" "$icons_dir"
if [[ -n "$desktop_dir" ]]; then
  mkdir -p "$desktop_dir"
fi

cp -f "$icon_source" "$icons_dir/netstitch.png"

escape_exec_path() {
  local value="$1"
  value="${value//\\/\\\\}"
  value="${value// /\\ }"
  printf '%s' "$value"
}

write_launcher() {
  local target="$1"
  cat > "$target" <<EOF
[Desktop Entry]
Type=Application
Name=NetStitch
Comment=Local network endpoint observer
Exec=$(escape_exec_path "$app_path")
Icon=netstitch
Terminal=false
Categories=Network;Utility;
StartupWMClass=NetStitch
EOF
  chmod +x "$target"
}

write_launcher "$applications_dir/netstitch.desktop"

if [[ -n "$desktop_dir" ]]; then
  desktop_file="$desktop_dir/NetStitch.desktop"
  write_launcher "$desktop_file"
  if command -v gio >/dev/null 2>&1; then
    gio set "$desktop_file" metadata::trusted true >/dev/null 2>&1 || true
  fi
fi

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$applications_dir" >/dev/null 2>&1 || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f -t "${HOME}/.local/share/icons/hicolor" >/dev/null 2>&1 || true
fi

echo "Installed NetStitch launcher for current user."
