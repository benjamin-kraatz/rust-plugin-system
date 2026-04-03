#!/usr/bin/env bash
set -euo pipefail

# Build and upload rustdoc for the shared crates to Kellnr.
#
# Usage:
#   scripts/upload-kellnr-docs.sh
#   scripts/upload-kellnr-docs.sh --dry-run
#
# Environment:
#   CARGO_REGISTRIES_DZWEI_REGISTRY_TOKEN  required for upload
#   KELLNR_BASE_URL                        optional, default https://crates.d-zwei.de

DRY_RUN=false
for arg in "$@"; do
  case "$arg" in
    --dry-run)
      DRY_RUN=true
      ;;
    *)
      echo "error: unknown argument: $arg" >&2
      exit 1
      ;;
  esac
done

if ! command -v zip >/dev/null 2>&1; then
  echo "error: zip is required but not installed" >&2
  exit 1
fi

if ! command -v curl >/dev/null 2>&1; then
  echo "error: curl is required but not installed" >&2
  exit 1
fi

BASE_URL="${KELLNR_BASE_URL:-https://crates.d-zwei.de}"
DOC_ZIP="target/doc.zip"
MAX_RETRIES=12
REGISTRY_TOKEN="${CARGO_REGISTRIES_DZWEI_REGISTRY_TOKEN:-}"

if [[ -z "$REGISTRY_TOKEN" && -f .env ]]; then
  REGISTRY_TOKEN=$(python3 - <<'PY'
import pathlib, re
text = pathlib.Path('.env').read_text(encoding='utf-8')
token = ''
for key in ('CARGO_REGISTRIES_DZWEI_REGISTRY_TOKEN', 'DZWEI_CRATES_REG_TOKEN'):
    m = re.search(rf'^{key}="([^"]+)"', text, re.M)
    if m:
        value = m.group(1)
        if value.startswith('$'):
            ref = value[1:]
            ref_match = re.search(rf'^{ref}="([^"]+)"', text, re.M)
            value = ref_match.group(1) if ref_match else ''
        token = value
        break
print(token, end='')
PY
)
fi

if [[ "$DRY_RUN" != "true" && -z "$REGISTRY_TOKEN" ]]; then
  credentials_file="${CARGO_HOME:-$HOME/.cargo}/credentials.toml"
  if [[ ! -f "$credentials_file" ]]; then
    credentials_file="${CARGO_HOME:-$HOME/.cargo}/credentials"
  fi

  if [[ -f "$credentials_file" ]]; then
    REGISTRY_TOKEN=$(CREDENTIALS_FILE="$credentials_file" python3 - <<'PY'
import os
import pathlib
import sys

credentials_path = pathlib.Path(os.environ['CREDENTIALS_FILE'])
try:
    import tomllib
except ModuleNotFoundError:
    sys.exit(0)

with credentials_path.open('rb') as handle:
    data = tomllib.load(handle)

token = (
    data.get('registries', {})
        .get('dzwei-registry', {})
        .get('token', '')
)
print(token, end='')
PY
)
  fi
fi

if [[ "$DRY_RUN" != "true" && -z "$REGISTRY_TOKEN" ]]; then
  echo "error: no dzwei-registry token found; run cargo login --registry dzwei-registry <TOKEN> or export CARGO_REGISTRIES_DZWEI_REGISTRY_TOKEN" >&2
  exit 1
fi

packages=(
  plugin-capabilities
  plugin-manifest
  plugin-protocol
  plugin-api
  plugin-abi
  plugin-runtime
  plugin-wasm
  plugin-sdk
  plugin-loader
  host-core
  plugin-test-kit
)

metadata_file=$(mktemp)
trap 'rm -f "$metadata_file"' EXIT
cargo metadata --no-deps --format-version 1 > "$metadata_file"

# Build docs once into the shared workspace target/doc directory.
doc_args=(--workspace --no-deps)
for pkg in "${packages[@]}"; do
  doc_args+=( -p "$pkg" )
done

echo "building rustdoc for shared crates..."
cargo doc "${doc_args[@]}"

rm -f "$DOC_ZIP"
(
  cd target
  zip -qr doc.zip doc
)

echo "prepared $DOC_ZIP"

for pkg in "${packages[@]}"; do
  version=$(PKG_NAME="$pkg" METADATA_FILE="$metadata_file" python3 - <<'PY'
import json, os
with open(os.environ['METADATA_FILE'], 'r', encoding='utf-8') as handle:
    metadata = json.load(handle)
pkg_name = os.environ['PKG_NAME']
pkg = next(p for p in metadata['packages'] if p['name'] == pkg_name)
print(pkg['version'])
PY
)

  url="${BASE_URL}/api/v1/docs/${pkg}/${version}"
  echo
  echo "=== ${pkg} ${version} ==="
  echo "url: ${url}"

  if [[ "$DRY_RUN" == "true" ]]; then
    continue
  fi

  attempt=1
  while true; do
    out=$(curl --silent --show-error --fail-with-body \
      -H "Authorization: ${REGISTRY_TOKEN}" \
      --upload-file "$DOC_ZIP" \
      "$url" 2>&1) && {
      if [[ -n "$out" ]]; then
        echo "$out"
      fi
      echo "uploaded docs for ${pkg} ${version}"
      break
    } || {
      if echo "$out" | grep -Eq '404|not found|crate with the correct version'; then
        if (( attempt >= MAX_RETRIES )); then
          echo "$out" >&2
          exit 1
        fi
        echo "crate version not visible yet for docs upload; retry ${attempt}/${MAX_RETRIES} ..."
        attempt=$((attempt + 1))
        sleep 5
        continue
      fi

      echo "$out" >&2
      exit 1
    }
  done
done
