#!/usr/bin/env bash
# Builds an installable Debian package from the Linux portable staging folder.
set -euo pipefail

version=""
portable_dir="dist/NetStitch-linux64-portable"
release_dir="dist/release-assets"
skip_build=0
jobs="${NETSTITCH__CARGO_JOBS:-}"
package_modules="${NETSTITCH__PACKAGE_MODULES:-}"
architecture="${NETSTITCH__DEB_ARCH:-amd64}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version)
      version="${2:?missing --version value}"
      shift 2
      ;;
    --portable-dir)
      portable_dir="${2:?missing --portable-dir value}"
      shift 2
      ;;
    --release-dir)
      release_dir="${2:?missing --release-dir value}"
      shift 2
      ;;
    --skip-build)
      skip_build=1
      shift
      ;;
    --jobs)
      jobs="${2:?missing --jobs value}"
      shift 2
      ;;
    --package-modules)
      package_modules="${2:?missing --package-modules value}"
      shift 2
      ;;
    --architecture)
      architecture="${2:?missing --architecture value}"
      shift 2
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"

package_args=(--output-dir "$portable_dir")
if [[ "$skip_build" -eq 1 ]]; then
  package_args+=(--skip-build)
fi
if [[ -n "$jobs" ]]; then
  package_args+=(--jobs "$jobs")
fi
if [[ -n "$package_modules" ]]; then
  package_args+=(--package-modules "$package_modules")
fi

cd "$repo_root"
bash "$script_dir/package_portable_linux.sh" "${package_args[@]}"

portable_path="$(realpath "$portable_dir")"
manifest_path="$portable_path/config/version-manifest.json"
if [[ -z "$version" ]]; then
  version="$(python3 - "$manifest_path" <<'PY'
import json
import sys

try:
    manifest = json.load(open(sys.argv[1], "r", encoding="utf-8"))
    print((manifest.get("modules") or {}).get("app", {}).get("full_version") or "")
except Exception:
    print("")
PY
)"
fi
if [[ -z "$version" ]]; then
  version="0.0.0+dev"
fi
deb_version="$(printf '%s' "$version" | sed -E 's|[^A-Za-z0-9.+:~/-]+|-|g')"
if [[ ! "$deb_version" =~ ^[0-9] ]]; then
  deb_version="0.0.0+${deb_version}"
fi
file_version="$(printf '%s' "$deb_version" | sed -E 's/[^A-Za-z0-9_.+-]+/-/g')"

release_path="$repo_root/$release_dir"
staging_root="$(mktemp -d "${TMPDIR:-/tmp}/netstitch-deb.XXXXXX")"
package_root="$staging_root/netstitch"
deb_path="$release_path/netstitch_${file_version}_${architecture}.deb"

cleanup() {
  rm -rf "$staging_root"
}
trap cleanup EXIT

mkdir -p \
  "$package_root/DEBIAN" \
  "$package_root/opt/netstitch" \
  "$package_root/usr/bin" \
  "$package_root/usr/share/applications" \
  "$package_root/usr/share/pixmaps"

cp -a "$portable_path/." "$package_root/opt/netstitch/"
rm -rf "$package_root/opt/netstitch/storage"
mkdir -p "$package_root/opt/netstitch/storage/exports"
if [[ -d "$package_root/opt/netstitch/integrations" ]]; then
  find "$package_root/opt/netstitch/integrations" -mindepth 2 -maxdepth 2 -type d -name data -exec rm -rf {} +
fi

required_runtime_files=(
  "opt/netstitch/NetStitch"
  "opt/netstitch/libs/netstitch-tool/bin/linux-x86_64/libnetstitch_tool.so"
  "opt/netstitch/libs/netstitch-watcher/bin/linux-x86_64/libnetstitch_watcher.so"
  "opt/netstitch/resources/shin0by.png"
)
for required in "${required_runtime_files[@]}"; do
  if [[ ! -f "$package_root/$required" ]]; then
    echo "Linux installer package is missing required runtime file: $required" >&2
    exit 1
  fi
done

chmod +x "$package_root/opt/netstitch/NetStitch"
cp -f "$package_root/opt/netstitch/resources/shin0by.png" "$package_root/usr/share/pixmaps/netstitch.png"

cat > "$package_root/usr/bin/netstitch" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

app_root="/opt/netstitch"
data_root="${XDG_DATA_HOME:-$HOME/.local/share}/netstitch"
mkdir -p "$data_root" "$data_root/apps" "$data_root/integrations"

if [[ -d "$app_root/apps" ]]; then
  cp -an "$app_root/apps/." "$data_root/apps/" 2>/dev/null || true
fi
if [[ -d "$app_root/integrations" ]]; then
  find "$app_root/integrations" -mindepth 1 -maxdepth 1 -type d | while IFS= read -r module_dir; do
    module_id="$(basename "$module_dir")"
    target_module_dir="$data_root/integrations/$module_id"
    mkdir -p "$target_module_dir"
    find "$module_dir" -mindepth 1 -maxdepth 1 ! -name data -exec cp -a {} "$target_module_dir/" \; 2>/dev/null || true
  done
fi

export NETSTITCH__DATA_DIR="$data_root"
export NETSTITCH__CONNECTORS_DIR="$data_root/apps"
export NETSTITCH__INTEGRATIONS_DIR="$data_root/integrations"

cd /opt/netstitch
exec /opt/netstitch/NetStitch "$@"
EOF
chmod +x "$package_root/usr/bin/netstitch"

cat > "$package_root/usr/share/applications/netstitch.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=NetStitch
Comment=Local network endpoint observer
Exec=/usr/bin/netstitch
Icon=netstitch
Terminal=false
Categories=Network;Utility;
StartupNotify=true
StartupWMClass=NetStitch
EOF
chmod 0644 "$package_root/usr/share/applications/netstitch.desktop"

cat > "$package_root/DEBIAN/control" <<EOF
Package: netstitch
Version: $deb_version
Section: net
Priority: optional
Architecture: $architecture
Maintainer: NetStitch <local@netstitch>
Depends: ca-certificates, xdg-utils, iputils-ping, traceroute, dnsutils, libwebkit2gtk-4.1-0, libgtk-3-0 | libgtk-3-0t64, libayatana-appindicator3-1, libxdo3, libssl3 | libssl3t64, libsqlite3-0
Description: NetStitch local network endpoint observer
 NetStitch shows which network addresses and domains are contacted by selected
 applications and includes the native runtime libraries required by the app.
EOF

cat > "$package_root/DEBIAN/postinst" <<'EOF'
#!/usr/bin/env bash
set -e

chmod +x /opt/netstitch/NetStitch /usr/bin/netstitch 2>/dev/null || true
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database /usr/share/applications >/dev/null 2>&1 || true
fi
EOF
chmod 0755 "$package_root/DEBIAN/postinst"

find "$package_root" -type d -exec chmod 0755 {} +
find "$package_root" -type f -exec chmod 0644 {} +
chmod 0755 \
  "$package_root/opt/netstitch/NetStitch" \
  "$package_root/opt/netstitch/install-desktop-launcher.sh" \
  "$package_root/usr/bin/netstitch" \
  "$package_root/DEBIAN/postinst"
find "$package_root/opt/netstitch/libs" -type f -name '*.so' -exec chmod 0755 {} +
find "$package_root/opt/netstitch/integrations" -type f -name '*.so' -exec chmod 0755 {} + 2>/dev/null || true
chmod 0755 "$package_root/DEBIAN"
chmod 0644 "$package_root/DEBIAN/control"

mkdir -p "$release_path"
find "$release_path" -mindepth 1 -maxdepth 1 -type f -name 'netstitch_*_*.deb' -delete
dpkg-deb --root-owner-group --build "$package_root" "$deb_path"
ls -l "$deb_path"
