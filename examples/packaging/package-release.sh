#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
output_root="${1:-$repo_root/target/example-bundles}"

case "$(uname -s)" in
  Darwin) dylib_ext="dylib" ;;
  Linux) dylib_ext="so" ;;
  MINGW*|MSYS*|CYGWIN*|Windows_NT) dylib_ext="dll" ;;
  *) echo "Unsupported platform" >&2; exit 1 ;;
esac

cargo build --release -p hello-world -p service-hooks -p abi-stable-greeter --quiet

native_bundle="$output_root/native-json/hello-world-bundle"
service_bundle="$output_root/native-json/service-hooks-bundle"
abi_bundle="$output_root/abi-stable/abi-stable-greeter-bundle"
wasm_bundle="$output_root/wasm/wasm-sandboxed-bundle"
web_bundle="$output_root/wasm/web-widget-bundle"

rm -rf "$native_bundle" "$service_bundle" "$abi_bundle" "$wasm_bundle" "$web_bundle"
mkdir -p "$native_bundle/lib" "$service_bundle/lib" "$abi_bundle/lib" "$wasm_bundle" "$web_bundle"

cp "$repo_root/examples/packaging/native-json/hello-world-bundle/package.json" "$native_bundle/package.json"
cp "$repo_root/examples/packaging/native-json/hello-world-bundle/plugin-manifest.json" "$native_bundle/plugin-manifest.json"
cp "$repo_root/examples/packaging/native-json/hello-world-bundle/release.json" "$native_bundle/release.json"
cp "$repo_root/target/release/libhello_world.$dylib_ext" "$native_bundle/lib/"

cp "$repo_root/examples/packaging/native-json/service-hooks-bundle/package.json" "$service_bundle/package.json"
cp "$repo_root/examples/packaging/native-json/service-hooks-bundle/plugin-manifest.json" "$service_bundle/plugin-manifest.json"
cp "$repo_root/examples/packaging/native-json/service-hooks-bundle/release.json" "$service_bundle/release.json"
cp "$repo_root/target/release/libservice_hooks.$dylib_ext" "$service_bundle/lib/"

cp "$repo_root/examples/packaging/abi-stable/abi-stable-greeter-bundle/package.json" "$abi_bundle/package.json"
cp "$repo_root/examples/packaging/abi-stable/abi-stable-greeter-bundle/plugin-manifest.json" "$abi_bundle/plugin-manifest.json"
cp "$repo_root/examples/packaging/abi-stable/abi-stable-greeter-bundle/release.json" "$abi_bundle/release.json"
cp "$repo_root/target/release/libabi_stable_greeter.$dylib_ext" "$abi_bundle/lib/"

cp "$repo_root/examples/packaging/wasm/wasm-sandboxed-bundle/package.json" "$wasm_bundle/package.json"
cp "$repo_root/examples/packaging/wasm/wasm-sandboxed-bundle/release.json" "$wasm_bundle/release.json"
cp "$repo_root/examples/packaging/wasm/wasm-sandboxed-bundle/wasm-plugin.json" "$wasm_bundle/wasm-plugin.json"
cp "$repo_root/examples/packaging/wasm/wasm-sandboxed-bundle/module.wat" "$wasm_bundle/module.wat"

cp "$repo_root/examples/packaging/wasm/web-widget-bundle/package.json" "$web_bundle/package.json"
cp "$repo_root/examples/packaging/wasm/web-widget-bundle/release.json" "$web_bundle/release.json"
cp "$repo_root/examples/packaging/wasm/web-widget-bundle/wasm-plugin.json" "$web_bundle/wasm-plugin.json"
cp "$repo_root/examples/packaging/wasm/web-widget-bundle/module.wat" "$web_bundle/module.wat"

echo "Wrote example bundles to $output_root"
