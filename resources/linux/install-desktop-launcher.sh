#!/usr/bin/env bash
# Installs a per-user NetStitch launcher from the portable folder and reports missing runtime libraries.
set -euo pipefail

portable_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
copy_to=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --copy-to)
      copy_to="${2:?missing --copy-to value}"
      shift 2
      ;;
    --help|-h)
      cat <<'EOF'
Usage: ./install-desktop-launcher.sh [--copy-to ~/.local/opt/netstitch]

Without --copy-to, installs launchers that point to this portable folder.
With --copy-to, copies this portable folder to the target directory first and
installs launchers that point to the copied app.
EOF
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

if [[ -n "$copy_to" ]]; then
  target_dir="${copy_to/#\~/$HOME}"
  mkdir -p "$(dirname "$target_dir")"
  rm -rf "$target_dir"
  mkdir -p "$target_dir"
  cp -a "$portable_dir/." "$target_dir/"
  portable_dir="$(cd "$target_dir" && pwd)"
fi

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

report_missing_runtime_libraries() {
  if ! command -v ldd >/dev/null 2>&1; then
    return
  fi
  local missing
  missing="$(ldd "$app_path" 2>/dev/null | awk '/not found/ { print $1 }' | sort -u || true)"
  if [[ -z "$missing" ]]; then
    return
  fi

  cat >&2 <<EOF
NetStitch was installed, but some Linux runtime libraries are missing:
$missing

Install the matching packages for your distribution. Common package names:
- Debian/Ubuntu/Mint: libwebkit2gtk-4.1-0 libgtk-3-0 libayatana-appindicator3-1 libxdo3 libssl3 libsqlite3-0
- Debian 13: libwebkit2gtk-4.1-0 libgtk-3-0t64 libayatana-appindicator3-1 libxdo3 libssl3t64 libsqlite3-0
- Fedora: webkit2gtk4.1 gtk3 libxdo libappindicator-gtk3 openssl-libs sqlite-libs
- Arch: webkit2gtk-4.1 gtk3 libxdo libappindicator-gtk3 openssl sqlite
- openSUSE: webkit2gtk4 libgtk-3-0 libxdo3 libopenssl3 sqlite3
EOF
}

report_missing_network_diagnostic_tools() {
  local missing=()
  for command_name in ping traceroute nslookup; do
    if ! command -v "$command_name" >/dev/null 2>&1; then
      missing+=("$command_name")
    fi
  done
  if [[ ${#missing[@]} -eq 0 ]]; then
    return
  fi

  cat >&2 <<EOF
NetStitch network diagnostics need additional commands: ${missing[*]}

Common package names:
- Debian/Ubuntu/Mint: iputils-ping traceroute dnsutils
- Fedora: iputils traceroute bind-utils
- Arch: iputils traceroute bind
- openSUSE: iputils traceroute bind-utils
EOF
}

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

report_missing_runtime_libraries
report_missing_network_diagnostic_tools
echo "Installed NetStitch launcher for current user."
