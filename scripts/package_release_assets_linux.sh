#!/usr/bin/env bash
# Builds a Linux portable package and creates a folder-first tar.gz release asset.
set -euo pipefail

version=""
portable_dir="dist/NetStitch-linux64-portable"
release_dir="dist/release-assets"
skip_build=0
jobs="${NETSTITCH__CARGO_JOBS:-}"

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

cd "$repo_root"
bash "$script_dir/package_portable_linux.sh" "${package_args[@]}"

portable_path="$(realpath "$portable_dir")"
release_path="$repo_root/$release_dir"
mkdir -p "$release_path"
find "$release_path" -mindepth 1 -maxdepth 1 -type f -name 'NetStitch-linux64-portable-*.tar.gz' -delete

if [[ -z "$version" ]]; then
  version="$(python3 - "$portable_path/config/version-manifest.json" <<'PY'
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
  version="dev"
fi
safe_version="$(printf '%s' "$version" | sed -E 's/[^A-Za-z0-9_.-]+/-/g')"
asset_path="$release_path/NetStitch-linux64-portable-$safe_version.tar.gz"

staging_root="$release_path/.release-staging"
rm -rf "$staging_root"
mkdir -p "$staging_root"
cp -a "$portable_path" "$staging_root/"
staged_portable_path="$staging_root/$(basename "$portable_path")"

mkdir -p "$staged_portable_path/apps/icons"
find "$staged_portable_path/apps" -maxdepth 1 \( -name '*.app' -o -name '*.toml' \) -type f -delete
find "$repo_root/resources/connectors/apps" -maxdepth 1 -name '*.app' -type f | while IFS= read -r connector_file; do
  cp -f "$connector_file" "$staged_portable_path/apps/"
done
for doc_name in README_RU.txt README_EN.txt; do
  if [[ -f "$repo_root/resources/connectors/apps/$doc_name" ]]; then
    cp -f "$repo_root/resources/connectors/apps/$doc_name" "$staged_portable_path/apps/"
  fi
done
rm -rf "$staged_portable_path/apps/icons"
mkdir -p "$staged_portable_path/apps/icons"
find "$repo_root/resources/connectors/icons" -maxdepth 1 -name '*.svg' -type f | while IFS= read -r icon_file; do
  cp -f "$icon_file" "$staged_portable_path/apps/icons/"
done

rm -rf "$staged_portable_path/storage"
mkdir -p "$staged_portable_path/storage/exports"

for required in \
  "libs/netstitch-tool/bin/linux-x86_64/libnetstitch_tool.so" \
  "libs/netstitch-watcher/bin/linux-x86_64/libnetstitch_watcher.so"
do
  if [[ ! -f "$staged_portable_path/$required" ]]; then
    echo "Linux portable release is missing required runtime file: $required" >&2
    exit 1
  fi
done

tar -C "$staging_root" -czf "$asset_path" "$(basename "$portable_path")"
rm -rf "$staging_root"
ls -l "$asset_path"
