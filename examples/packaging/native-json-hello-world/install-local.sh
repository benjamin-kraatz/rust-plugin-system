#!/usr/bin/env bash
set -euo pipefail

bundle_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
target_root="${1:-$HOME/.local/share/rust-plugin-system/plugins/hello-world}"
plugin_root="$target_root"

mkdir -p "$plugin_root"
cp "$bundle_root/layout/plugins/hello-world/plugin-manifest.json" "$plugin_root/plugin-manifest.json"

if [[ -f "$bundle_root/layout/plugins/hello-world/libhello_world.dylib" ]]; then
  cp "$bundle_root/layout/plugins/hello-world/libhello_world.dylib" "$plugin_root/libhello_world.dylib"
else
  printf 'warning: expected %s/layout/plugins/hello-world/libhello_world.dylib\n' "$bundle_root" >&2
  printf 'build the plugin and copy the dynamic library into the bundle before installing\n' >&2
fi

cp "$bundle_root/release-metadata.json" "$plugin_root/release-metadata.json"
printf 'installed hello-world bundle skeleton to %s\n' "$plugin_root"
