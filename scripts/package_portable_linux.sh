#!/usr/bin/env bash
# Builds a Linux portable folder with NetStitch, native runtime libraries, and integrations.
set -euo pipefail

configuration="release"
output_dir="dist/NetStitch-linux64-portable"
skip_build=0
jobs="${NETSTITCH__CARGO_JOBS:-}"
package_modules="${NETSTITCH__PACKAGE_MODULES:-}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-build)
      skip_build=1
      shift
      ;;
    --output-dir)
      output_dir="${2:?missing --output-dir value}"
      shift 2
      ;;
    --jobs)
      jobs="${2:?missing --jobs value}"
      shift 2
      ;;
    --package-modules)
      package_modules="${2:?missing --package-modules value}"
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
linux_target_root="${NETSTITCH__LINUX_CARGO_TARGET_DIR:-$repo_root/target/linux-portable}"
target_dir="$linux_target_root/$configuration"
portable_dir="$repo_root/$output_dir"
platform_key="linux-x86_64"
cargo_jobs=()
if [[ -n "$jobs" ]]; then
  cargo_jobs=(-j "$jobs")
fi

require_file() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    echo "Required file is missing: $path" >&2
    exit 1
  fi
}

copy_dir_contents() {
  local source_dir="$1"
  local destination_dir="$2"
  local pattern="${3:-*}"
  if [[ ! -d "$source_dir" ]]; then
    return
  fi
  mkdir -p "$destination_dir"
  shopt -s nullglob
  local item
  for item in "$source_dir"/$pattern; do
    cp -f "$item" "$destination_dir/"
  done
  shopt -u nullglob
}

copy_native_module() {
  local module_id="$1"
  local display_name="$2"
  local library_name="$3"
  local source_library="$target_dir/$library_name"
  require_file "$source_library"

  local module_dir="$portable_dir/libs/$module_id"
  local bin_dir="$module_dir/bin/$platform_key"
  rm -rf "$module_dir"
  mkdir -p "$bin_dir"
  cp -f "$source_library" "$bin_dir/$library_name"

  python3 - "$module_dir/module.json" "$module_id" "$display_name" "$platform_key" "$library_name" <<'PY'
import json
import sys

path, module_id, display_name, platform_key, library_name = sys.argv[1:]
payload = {
    "schema_version": 1,
    "id": module_id,
    "display_name": display_name,
    "transport": "native_library",
    "library_paths": {
        platform_key: f"bin/{platform_key}/{library_name}",
    },
}

with open(path, "w", encoding="utf-8") as handle:
    json.dump(payload, handle, ensure_ascii=False, indent=2)
    handle.write("\n")
PY
}

module_is_included() {
  local module_name="$1"
  local normalized_module="${module_name,,}"
  local raw item
  raw="${package_modules//;/,}"
  if [[ -z "$raw" ]]; then
    return 1
  fi
  IFS=',' read -ra items <<< "$raw"
  for item in "${items[@]}"; do
    item="${item//[[:space:]]/}"
    item="${item,,}"
    if [[ "$item" == "*" || "$item" == "$normalized_module" ]]; then
      return 0
    fi
  done
  return 1
}

read_package_version() {
  python3 - "$repo_root/Cargo.toml" <<'PY'
import re
import sys

for line in open(sys.argv[1], "r", encoding="utf-8"):
    match = re.match(r'\s*version\s*=\s*"([^"]+)"\s*$', line)
    if match:
        print(match.group(1))
        break
else:
    raise SystemExit("version not found")
PY
}

next_package_revision() {
  local revision_file="$1"
  if [[ -f "$revision_file" ]]; then
    python3 - "$revision_file" <<'PY'
import sys
try:
    value = int(open(sys.argv[1], "r", encoding="utf-8").read().strip())
except Exception:
    value = 0
print(value + 1)
PY
  else
    printf '1\n'
  fi
}

manifest_max_revision() {
  local manifest_path="$1"
  if [[ ! -f "$manifest_path" ]]; then
    return
  fi
  python3 - "$manifest_path" <<'PY'
import json
import sys

try:
    manifest = json.load(open(sys.argv[1], "r", encoding="utf-8"))
except Exception:
    raise SystemExit(0)

revisions = []
for entry in (manifest.get("modules") or {}).values():
    try:
        revisions.append(int(entry.get("revision") or 0))
    except Exception:
        pass
if revisions:
    print(max(revisions))
PY
}

resolve_package_revision() {
  local linux_revision_file="$1"
  local shared_manifests=(
    "$repo_root/dist/NetStitch-win64-portable/config/version-manifest.json"
    "$repo_root/dist/NetStitch-portable-windows-x86_64/config/version-manifest.json"
    "$repo_root/dist/NetStitch-portable/config/version-manifest.json"
  )
  local explicit_revision="${NETSTITCH__PACKAGE_REVISION:-}"
  local shared_revision candidate_revision

  if [[ -n "$explicit_revision" ]]; then
    printf '%s\n' "$explicit_revision"
    return
  fi

  shared_revision=""
  for shared_manifest in "${shared_manifests[@]}"; do
    candidate_revision="$(manifest_max_revision "$shared_manifest")"
    if [[ -n "$candidate_revision" && ( -z "$shared_revision" || "$candidate_revision" -gt "$shared_revision" ) ]]; then
      shared_revision="$candidate_revision"
    fi
  done
  if [[ -n "$shared_revision" ]]; then
    printf '%s\n' "$shared_revision"
    return
  fi

  next_package_revision "$linux_revision_file"
}

build_integration_workspaces() {
  local module_root cargo_manifest
  shopt -s nullglob
  for module_root in "$repo_root"/integrations/*; do
    if ! module_is_included "$(basename "$module_root")"; then
      continue
    fi
    cargo_manifest="$module_root/Cargo.toml"
    if [[ -f "$module_root/module.json" && -f "$cargo_manifest" ]]; then
      cargo build --manifest-path "$cargo_manifest" --target-dir "$module_root/target/linux-portable" --release "${cargo_jobs[@]}"
    fi
  done
  shopt -u nullglob
}

sync_integrations() {
  local integrations_dir="$portable_dir/integrations"
  mkdir -p "$integrations_dir"

  local module_root manifest module_name module_dir library_path library_name source_library
  shopt -s nullglob
  for module_root in "$repo_root"/integrations/*; do
    manifest="$module_root/module.json"
    if [[ ! -f "$manifest" ]]; then
      continue
    fi
    module_name="$(basename "$module_root")"
    if [[ -d "$integrations_dir/$module_name" ]] && ! module_is_included "$module_name"; then
      rm -rf "$integrations_dir/$module_name"
    fi
  done
  for module_root in "$repo_root"/integrations/*; do
    manifest="$module_root/module.json"
    if [[ ! -f "$manifest" ]]; then
      continue
    fi

    module_name="$(basename "$module_root")"
    if ! module_is_included "$module_name"; then
      continue
    fi
    module_dir="$integrations_dir/$module_name"
    mkdir -p "$module_dir/data"
    cp -f "$manifest" "$module_dir/module.json"
    if [[ -d "$module_root/assets" ]]; then
      rm -rf "$module_dir/assets"
      cp -a "$module_root/assets" "$module_dir/assets"
    fi

    library_path="$(python3 - "$manifest" "$platform_key" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    manifest = json.load(handle)
paths = manifest.get("library_paths") or {}
for key in (sys.argv[2], "linux", "default"):
    value = paths.get(key)
    if value:
        print(value)
        break
PY
)"
    if [[ -z "$library_path" ]]; then
      continue
    fi

    library_name="$(basename "$library_path")"
    mkdir -p "$module_dir/$(dirname "$library_path")"
    source_library=""
    if [[ -f "$module_root/target/linux-portable/$configuration/$library_name" ]]; then
      source_library="$module_root/target/linux-portable/$configuration/$library_name"
    elif [[ -f "$target_dir/$library_name" ]]; then
      source_library="$target_dir/$library_name"
    elif [[ -f "$module_root/$library_path" ]]; then
      source_library="$module_root/$library_path"
    fi

    if [[ -n "$source_library" ]]; then
      cp -f "$source_library" "$module_dir/$library_path"
    else
      echo "Warning: integration module '$module_name' declares '$library_path', but the native library was not found." >&2
    fi
  done
  shopt -u nullglob
}

write_version_manifest() {
  local package_version="$1"
  local revision="$2"
  local config_dir="$portable_dir/config"
  local revision_file="$config_dir/package-revision.txt"
  local manifest_path="$config_dir/version-manifest.json"

  mkdir -p "$config_dir"
  printf '%s\n' "$revision" > "$revision_file"

  python3 - "$manifest_path" "$package_version" "$revision" \
    "$portable_dir/NetStitch" \
    "$portable_dir/libs/netstitch-watcher/bin/$platform_key/libnetstitch_watcher.so" \
    "$portable_dir/libs/netstitch-tool/bin/$platform_key/libnetstitch_tool.so" <<'PY'
import datetime as dt
import hashlib
import json
import os
import sys

manifest_path, package_version, revision, app_path, watcher_path, tool_path = sys.argv[1:]
revision = int(revision)

def entry(name, path):
    with open(path, "rb") as handle:
        digest = hashlib.sha256(handle.read()).hexdigest()
    return {
        "artifact": name,
        "revision": revision,
        "full_version": f"{package_version}.{revision}",
        "artifact_hash": digest,
    }

payload = {
    "package_version": package_version,
    "generated_at": dt.datetime.now(dt.timezone.utc).isoformat().replace("+00:00", "Z"),
    "modules": {
        "app": entry("NetStitch", app_path),
        "watcher": entry("libnetstitch_watcher.so", watcher_path),
        "tool": entry("libnetstitch_tool.so", tool_path),
    },
}
os.makedirs(os.path.dirname(manifest_path), exist_ok=True)
with open(manifest_path, "w", encoding="utf-8") as handle:
    json.dump(payload, handle, ensure_ascii=False, indent=2)
    handle.write("\n")
PY
}

write_desktop_launcher() {
  local icon_dir="$portable_dir/resources"
  local app_icon_path="$icon_dir/shin0by.png"
  local branding_path="$icon_dir/netstitch.png"
  local app_icon_source="$repo_root/src/netstitch-ui/assets/shin0by.png"
  local branding_source="$repo_root/resources/branding/images/NetStitch.png"
  local desktop_path="$portable_dir/NetStitch.desktop"

  if [[ ! -f "$app_icon_source" ]]; then
    echo "Missing Linux desktop icon: $app_icon_source" >&2
    exit 1
  fi
  if [[ ! -f "$branding_source" ]]; then
    echo "Missing Linux branding image: $branding_source" >&2
    exit 1
  fi
  mkdir -p "$icon_dir"
  cp -f "$app_icon_source" "$app_icon_path"
  cp -f "$branding_source" "$branding_path"
  cat > "$desktop_path" <<EOF
[Desktop Entry]
Type=Application
Name=NetStitch
Comment=Local network endpoint observer
Exec=$portable_dir/NetStitch
Icon=$app_icon_path
Terminal=false
Categories=Network;Utility;
StartupWMClass=NetStitch
EOF
  chmod +x "$desktop_path"

  if [[ -f "$repo_root/resources/linux/install-desktop-launcher.sh" ]]; then
    cp -f "$repo_root/resources/linux/install-desktop-launcher.sh" "$portable_dir/install-desktop-launcher.sh"
    chmod +x "$portable_dir/install-desktop-launcher.sh"
  fi
}

cd "$repo_root"

mkdir -p "$portable_dir/config"
package_version="$(read_package_version)"
package_revision="$(resolve_package_revision "$portable_dir/config/package-revision.txt")"
export NETSTITCH__BUILD_FULL_VERSION="$package_version.$package_revision"

if [[ "$skip_build" -eq 0 ]]; then
  cargo build --workspace --target-dir "$linux_target_root" --release "${cargo_jobs[@]}"
  build_integration_workspaces
fi

require_file "$target_dir/NetStitch"
require_file "$target_dir/libnetstitch_watcher.so"
require_file "$target_dir/libnetstitch_tool.so"

mkdir -p "$portable_dir"
find "$portable_dir" -mindepth 1 -maxdepth 1 \
  ! -name config \
  ! -name integrations \
  ! -name language \
  ! -name libs \
  ! -name apps \
  ! -name storage \
  -exec rm -rf {} +

cp -f "$target_dir/NetStitch" "$portable_dir/NetStitch"
chmod +x "$portable_dir/NetStitch"

copy_native_module "netstitch-watcher" "NetStitch Watcher" "libnetstitch_watcher.so"
copy_native_module "netstitch-tool" "NetStitch Tool" "libnetstitch_tool.so"

mkdir -p "$portable_dir/config"
if [[ -f "$repo_root/config/endpoint_probe_targets.txt" ]]; then
  cp -f "$repo_root/config/endpoint_probe_targets.txt" "$portable_dir/config/"
fi

rm -rf "$portable_dir/language"
copy_dir_contents "$repo_root/resources/language" "$portable_dir/language" "*.ini"

mkdir -p "$portable_dir/apps"
find "$portable_dir/apps" -maxdepth 1 -name '*.toml' -type f -delete
find "$repo_root/resources/connectors/apps" -maxdepth 1 -name '*.app' -type f | while IFS= read -r connector_file; do
  connector_name="$(basename "$connector_file")"
  connector_base="${connector_name%.app}"
  rm -f "$portable_dir/apps/$connector_base.app"
  cp -f "$connector_file" "$portable_dir/apps/$connector_name"
done
copy_dir_contents "$repo_root/resources/connectors/icons" "$portable_dir/apps/icons" "*.svg"
for doc_name in README_RU.txt README_EN.txt; do
  if [[ -f "$repo_root/resources/connectors/apps/$doc_name" ]]; then
    cp -f "$repo_root/resources/connectors/apps/$doc_name" "$portable_dir/apps/"
  fi
done

sync_integrations

mkdir -p "$portable_dir/storage/exports"
printf 'clear system_events on next NetStitch startup\n' > "$portable_dir/storage/clear-system-events-on-next-start"
rm -rf "$portable_dir/storage/icons"

write_desktop_launcher
write_version_manifest "$package_version" "$package_revision"
find "$portable_dir" -maxdepth 2 -type f | sort
