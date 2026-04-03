# Versioning and Releasing Crates

This guide explains how the shared crates in this workspace are versioned,
how changelogs are generated, and how to perform a release — both from your
local machine and through CI.

---

## Table of contents

- [Versioning and Releasing Crates](#versioning-and-releasing-crates)
  - [Table of contents](#table-of-contents)
  - [Strategy overview](#strategy-overview)
  - [Tools required](#tools-required)
  - [Commit message conventions](#commit-message-conventions)
    - [Breaking changes](#breaking-changes)
  - [Generating a changelog locally](#generating-a-changelog-locally)
  - [Running a release from your local machine](#running-a-release-from-your-local-machine)
    - [Prerequisites](#prerequisites)
    - [Step 1 — dry-run preview](#step-1--dry-run-preview)
    - [Step 2 — check semver compatibility (patch and minor only)](#step-2--check-semver-compatibility-patch-and-minor-only)
    - [Step 3 — execute](#step-3--execute)
    - [After the release](#after-the-release)
  - [Running a release through CI](#running-a-release-through-ci)
  - [Publishing to kellnr manually (ad-hoc)](#publishing-to-kellnr-manually-ad-hoc)
  - [Semver compatibility checks](#semver-compatibility-checks)
  - [Versioning rules reference](#versioning-rules-reference)
  - [Troubleshooting](#troubleshooting)
  - [Related docs](#related-docs)

---

## Strategy overview

All 11 shared crates (`plugin-capabilities`, `plugin-manifest`, `plugin-protocol`,
`plugin-api`, `plugin-abi`, `plugin-runtime`, `plugin-wasm`, `plugin-sdk`,
`plugin-loader`, `host-core`, `plugin-test-kit`) use **lockstep versioning** —
every release bumps all of them to the same version at the same time.

This simplifies dependency management: downstream projects always pin a single
version number for all crates from this workspace.

Releases follow [Semantic Versioning](https://semver.org/):

| Bump    | When                                                         |
| ------- | ------------------------------------------------------------ |
| `patch` | Bug fixes, documentation, internal refactoring — no API change |
| `minor` | New backwards-compatible features or public API additions    |
| `major` | Breaking public API changes                                  |

---

## Tools required

Install these once:

```bash
cargo install cargo-release git-cliff cargo-semver-checks
```

| Tool                  | Purpose                                                    |
| --------------------- | ---------------------------------------------------------- |
| `cargo-release`       | Bumps versions, runs changelog hook, commits, tags, publishes |
| `git-cliff`           | Generates `CHANGELOG.md` from Conventional Commit history  |
| `cargo-semver-checks` | Detects accidental public API breakage before a release    |

---

## Commit message conventions

This project uses [Conventional Commits](https://www.conventionalcommits.org/).
git-cliff parses these to group changelog entries automatically.

```
<type>[optional scope]: <description>

feat: add new capability negotiation API
fix: correct sparse index URL in kellnr config
perf: reduce allocations in plugin-runtime invoke path
refactor: split host-core discovery into smaller helpers
docs: update quickstart with registry instructions
test: add integration test for hot-reload scenario
chore: update dependencies
```

Commits that do **not** follow this format are silently skipped by git-cliff
(they still land in git history but won't appear in the changelog).

### Breaking changes

Append `!` after the type, or add a `BREAKING CHANGE:` footer:

```
feat!: rename PluginResponse::ok to PluginResponse::success

BREAKING CHANGE: PluginResponse::ok has been renamed to PluginResponse::success.
```

---

## Generating a changelog locally

Preview what the changelog would contain for unreleased commits:

```bash
git-cliff --unreleased --output -
```

Regenerate `CHANGELOG.md` from the full git history up to a specific tag (this
is what the release hook does automatically):

```bash
git-cliff --output CHANGELOG.md --tag v0.2.0
```

---

## Running a release from your local machine

### Prerequisites

1. You are on `main` with a clean working tree (`git status` shows nothing).
2. You have a kellnr API token with **publish** permission.
3. You have authenticated with the registry:

   ```bash
   cargo login --registry dzwei-registry <YOUR_TOKEN>
   ```

   Or export the token as an environment variable (preferred for scripts):

   ```bash
   export CARGO_REGISTRIES_DZWEI_REGISTRY_TOKEN=<YOUR_TOKEN>
   ```

### Step 1 — dry-run preview

Always preview before executing. This prints exactly what will change without
writing anything to disk, git, or the registry:

```bash
cargo release patch          # or: minor / major
```

Do not use `cargo-release release` for normal versioning. In this workspace,
that attempts a no-bump publish flow and can fail in dry-run mode while waiting
for dependencies that were intentionally not uploaded.

Review the output carefully:
- Each crate's version bump (`0.1.0 → 0.1.1`)
- Downstream dependency updates within the workspace
- The pre-release hook command (git-cliff invocation)
- The list of crates that will be published

### Step 2 — check semver compatibility (patch and minor only)

Before a `patch` or `minor` release, verify that no public API has changed
inadvertently:

```bash
# Check all publishable crates against their currently-published baseline
for pkg in plugin-capabilities plugin-manifest plugin-protocol plugin-api \
            plugin-abi plugin-runtime plugin-wasm plugin-sdk \
            plugin-loader host-core plugin-test-kit; do
  echo "=== $pkg ==="
  cargo semver-checks --package "$pkg"
done
```

If `cargo-semver-checks` reports a breaking change and you intended it, use
`major` instead of `patch`/`minor`.

### Step 3 — execute

```bash
cargo release patch --execute
```

You will be prompted to confirm each step (version bump, hook, tag, publish).
To skip all prompts:

```bash
cargo release patch --execute --no-confirm
```

What happens under the hood:

1. All 11 crates' `Cargo.toml` files are updated to the new version.
2. Workspace-internal dependency version requirements are updated to match.
3. `git-cliff` is called (pre-release hook) — it regenerates `CHANGELOG.md`
   and stages it.
4. A commit is made: `chore: release v0.1.1`.
5. A git tag is created: `v0.1.1`.
6. Each crate is published to `dzwei-registry` in dependency order.
7. The commit and tag are pushed to `origin`.

### After the release

Pull the updated `main` branch locally:

```bash
git pull --tags
```

---

## Running a release through CI

The repository ships `.github/workflows/release.yml`.  Trigger it from the
**Actions** tab in GitHub.

**Inputs:**

| Input      | Type    | Default  | Description                                                    |
| ---------- | ------- | -------- | -------------------------------------------------------------- |
| `level`    | choice  | `patch`  | Version bump level: `patch`, `minor`, or `major`               |
| `dry_run`  | boolean | `true`   | If `true`, previews the release without writing anything       |

**Recommended workflow:**

1. Trigger with `level=patch` and `dry_run=true`. Review the logs.
2. If everything looks right, trigger again with `dry_run=false`.

**What CI does additionally:**

- For `patch` and `minor`, it runs `cargo semver-checks` on all publishable
  crates before proceeding. The job is skipped for `major` (breaking changes
  are intentional).
- Uses `GITHUB_TOKEN` (built-in, no setup needed) for the git push and tag.
- Uses `DZWEI_CRATES_REG_TOKEN` (repository secret) for publishing.

**Required secrets (one-time setup in GitHub → Settings → Secrets):**

| Secret                    | Purpose                                 |
| ------------------------- | --------------------------------------- |
| `DZWEI_CRATES_REG_TOKEN`  | kellnr API token with publish permission |

`GITHUB_TOKEN` is injected automatically — no secret needed.

> **Note:** Commits pushed by `GITHUB_TOKEN` do not re-trigger CI workflows.
> This is intentional: the version-bump commit only modifies `Cargo.toml` files
> and `CHANGELOG.md`, which are safe without a full CI run.

---

## Publishing to kellnr manually (ad-hoc)

Use this when you need to re-publish a specific crate without a full version
bump (e.g. after a kellnr outage, or for a first-time publish).

The older `publish-crates.yml` workflow remains available for this purpose:
trigger it from **Actions → Publish crates to kellnr**.  It publishes each
crate in dependency order and skips versions that are already present.

For a one-off local publish of a single crate:

```bash
export CARGO_REGISTRIES_DZWEI_REGISTRY_TOKEN=<YOUR_TOKEN>

# Dry run
cargo publish --dry-run --registry dzwei-registry -p plugin-sdk

# Publish
cargo publish --registry dzwei-registry -p plugin-sdk
```

Dependency order (always publish in this sequence when doing it manually):

1. `plugin-capabilities`
2. `plugin-manifest`
3. `plugin-protocol`
4. `plugin-api`
5. `plugin-abi`
6. `plugin-runtime`
7. `plugin-wasm`
8. `plugin-sdk`
9. `plugin-loader`
10. `host-core`
11. `plugin-test-kit`

---

## Semver compatibility checks

`cargo-semver-checks` is integrated in two places:

| Where                        | When it runs                                    | Baseline       |
| ---------------------------- | ----------------------------------------------- | -------------- |
| `ci.yml` — `semver-checks` job | Every pull request targeting `main`           | `origin/main`  |
| `release.yml` — `semver-check` job | Before every `patch` or `minor` release | Published crate on `dzwei-registry` |

On pull requests, the job uses `--baseline-rev origin/main` so no registry
access is needed.  If the baseline doesn't exist yet (crate not yet published),
the check passes gracefully.

---

## Versioning rules reference

| Situation                              | Correct bump |
| -------------------------------------- | ------------ |
| Add a new `pub fn` or `pub struct`     | `minor`      |
| Remove or rename a `pub fn`            | `major`      |
| Change a function signature            | `major`      |
| Add a field to a `#[non_exhaustive]` struct | `minor` |
| Bug fix with no public API change      | `patch`      |
| Update documentation only             | `patch`      |
| Dependency version bump (non-breaking) | `patch`      |

---

## Troubleshooting

**`error: uncommitted changes detected`**  
Commit or stash all changes before running `cargo release`.

**`error: no token found for 'dzwei-registry'`**  
Export the token: `export CARGO_REGISTRIES_DZWEI_REGISTRY_TOKEN=<token>`,  
or run `cargo login --registry dzwei-registry <token>`.

**`cargo-semver-checks` warns about missing baseline**  
The crate has not been published yet. Run a first publish manually (see
[ad-hoc section](#publishing-to-kellnr-manually-ad-hoc)), then future releases
will have a baseline to compare against.

**git-cliff skips commits**  
Commits with non-conventional messages are silently skipped. Run
`git-cliff --unreleased --output - -vv` to see which commits were skipped and why.

**`cargo release patch` tries to publish non-publishable crates**  
All `hosts/`, `plugins/`, and `examples/` crates have `publish = false` in
their `Cargo.toml`.  If you add a new crate in those directories, add
`publish = false` under `[package]`.

---

## Related docs

- [`docs/guides/kellnr.md`](./kellnr.md) — registry setup and administration
- [`docs/guides/publishing.md`](./publishing.md) — consumer setup and ad-hoc publish reference
- [`docs/reference/version-compatibility.md`](../reference/version-compatibility.md)
- [`release.toml`](../../release.toml) — cargo-release workspace configuration
- [`cliff.toml`](../../cliff.toml) — git-cliff changelog configuration
