#!/usr/bin/env sh
set -eu

# Determine semver bump from Conventional Commits since the latest tag.
# Priority: major > minor > patch
#
# Rules:
# - major: commit header contains ! before ':' (e.g. feat!: ...)
#          or body contains BREAKING CHANGE:
# - minor: at least one feat: commit and no major signals
# - patch: anything else

last_tag=$(git describe --tags --abbrev=0 2>/dev/null || true)
if [ -n "$last_tag" ]; then
  range="$last_tag..HEAD"
else
  range="HEAD"
fi

log=$(git log --format='%s%n%b%n----END----' "$range")

if printf '%s\n' "$log" | grep -Eiq '^[a-z]+\([^)]+\)!:|^[a-z]+!:|BREAKING CHANGE:'; then
  echo major
  exit 0
fi

if printf '%s\n' "$log" | grep -Eiq '^feat(\([^)]+\))?:'; then
  echo minor
  exit 0
fi

echo patch
