#!/usr/bin/env bash
set -euo pipefail

# Publish shared crates to dzwei-registry in strict dependency order.
#
# Usage:
#   scripts/publish-shared-crates.sh                       # publish
#   scripts/publish-shared-crates.sh --dry-run             # package only
#   scripts/publish-shared-crates.sh --allow-dirty         # allow local edits
#   scripts/publish-shared-crates.sh --dry-run --allow-dirty

DRY_RUN=false
ALLOW_DIRTY=false

for arg in "$@"; do
  case "$arg" in
    --dry-run)
      DRY_RUN=true
      ;;
    --allow-dirty)
      ALLOW_DIRTY=true
      ;;
    *)
      echo "error: unknown argument: $arg" >&2
      exit 1
      ;;
  esac
done

if [[ -z "${CARGO_REGISTRIES_DZWEI_REGISTRY_TOKEN:-}" ]]; then
  echo "error: CARGO_REGISTRIES_DZWEI_REGISTRY_TOKEN is not set" >&2
  exit 1
fi

REGISTRY="dzwei-registry"
MAX_RETRIES=12

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

for pkg in "${packages[@]}"; do
  echo
  echo "=== ${pkg} ==="

  publish_args=(--registry "$REGISTRY" -p "$pkg")
  if [[ "$ALLOW_DIRTY" == "true" ]]; then
    publish_args=(--allow-dirty "${publish_args[@]}")
  fi

  if [[ "$DRY_RUN" == "true" ]]; then
    cargo publish --dry-run "${publish_args[@]}"
    continue
  fi

  attempt=1
  while true; do
    out=$(cargo publish "${publish_args[@]}" 2>&1) && {
      echo "$out"
      echo "published ${pkg}"
      break
    } || {
      if echo "$out" | grep -q "already exists"; then
        echo "already published: ${pkg}"
        break
      fi

      # Retry while the registry index catches up after publishing dependencies.
      if echo "$out" | grep -Eq "candidate versions found which didn't match|due to a timeout while waiting for published dependencies"; then
        if (( attempt >= MAX_RETRIES )); then
          echo "$out" >&2
          exit 1
        fi
        echo "dependency not visible yet for ${pkg}; retry ${attempt}/${MAX_RETRIES} ..."
        attempt=$((attempt + 1))
        sleep 5
        continue
      fi

      echo "$out" >&2
      exit 1
    }
  done
done
