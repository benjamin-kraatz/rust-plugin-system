# Publishing via GitHub

This guide covers two GitHub-based ways to distribute the Rust Plugin System
crates, complementing the crates.io workflow described in
[`crates-io.md`](./crates-io.md).

---

## Option A — Automated publishing to crates.io via GitHub Actions

The easiest way to automate publishing is a GitHub Actions workflow that runs
`cargo publish` whenever you push a version tag.

### 1. Store your crates.io token as a secret

1. Go to **Settings → Secrets and variables → Actions → New repository secret**.
2. Name: `CRATES_IO_TOKEN`
3. Value: your crates.io API token (generate one at
   [crates.io → Account Settings → API Tokens](https://crates.io/settings/tokens)).

### 2. Add the workflow

The repository ships a ready-to-use workflow at
`.github/workflows/publish.yml`.  It is triggered by any tag that matches
`v*` (e.g. `v0.1.0`, `v0.2.0-rc.1`).

```yaml
# .github/workflows/publish.yml
name: Publish to crates.io

on:
  push:
    tags:
      - "v*"

jobs:
  publish:
    name: Publish crates
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Publish in dependency order
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CRATES_IO_TOKEN }}
        run: |
          cargo publish -p plugin-capabilities
          sleep 30
          cargo publish -p plugin-manifest
          sleep 30
          cargo publish -p plugin-protocol
          sleep 30
          cargo publish -p plugin-api
          sleep 30
          cargo publish -p plugin-runtime
          sleep 30
          cargo publish -p plugin-loader
          sleep 30
          cargo publish -p plugin-sdk
          sleep 30
          cargo publish -p plugin-abi
          sleep 30
          cargo publish -p plugin-wasm
          sleep 30
          cargo publish -p plugin-test-kit
          sleep 30
          cargo publish -p host-core
```

### 3. Trigger a release

```bash
git tag v0.1.0
git push origin v0.1.0
```

The workflow runs automatically and publishes each crate in dependency order.

---

## Option B — Git-based dependency via GitHub (no registry required)

If you want to share the crates without going through any registry, consumers
can reference them directly from the GitHub repository URL.  This is especially
useful for:

* private forks or enterprise repositories,
* development builds of unreleased versions,
* trying changes before they are published.

### Referencing a branch

```toml
# In the consumer's Cargo.toml
[dependencies]
plugin-sdk = { git = "https://github.com/benjamin-kraatz/rust-plugin-system", branch = "main" }
host-core  = { git = "https://github.com/benjamin-kraatz/rust-plugin-system", branch = "main" }
```

### Referencing a specific tag

```toml
[dependencies]
plugin-sdk = { git = "https://github.com/benjamin-kraatz/rust-plugin-system", tag = "v0.1.0" }
host-core  = { git = "https://github.com/benjamin-kraatz/rust-plugin-system", tag = "v0.1.0" }
```

### Referencing a specific commit

```toml
[dependencies]
plugin-sdk = { git = "https://github.com/benjamin-kraatz/rust-plugin-system", rev = "a1b2c3d" }
```

> **Important:** When using git dependencies Cargo resolves the entire
> workspace.  The workspace contains GUI crates (`host-egui`, `host-iced`,
> `host-dioxus-desktop`) that require platform-specific system libraries.
> On Linux CI you can exclude them via:
>
> ```bash
> cargo build --locked \
>   --exclude host-egui \
>   --exclude host-iced \
>   --exclude host-dioxus-desktop
> ```

### Private repositories

For private GitHub repositories, authenticate via SSH or a personal access
token:

```toml
[dependencies]
plugin-sdk = { git = "ssh://git@github.com/your-org/rust-plugin-system.git", tag = "v0.1.0" }
```

Ensure the machine running `cargo build` has the appropriate SSH key or
`CARGO_NET_GIT_FETCH_WITH_CLI=true` set to use the system Git.

---

## Option C — Custom sparse Cargo registry on GitHub Pages

For organisations that want a fully private registry with version resolution,
you can host a [sparse Cargo registry](https://doc.rust-lang.org/cargo/reference/registries.html)
on GitHub Pages.

### Set up the registry

1. Create a repository (e.g. `my-org/cargo-registry`) and enable GitHub Pages
   on the `main` branch.
2. Publish the index using a tool such as
   [cargo-http-registry](https://crates.io/crates/cargo-http-registry) or
   follow the
   [Cargo registry index format specification](https://doc.rust-lang.org/cargo/reference/registry-index.html).

### Configure consumers

Add the registry to the consuming project's `.cargo/config.toml`:

```toml
[registries]
my-org = { index = "sparse+https://my-org.github.io/cargo-registry/" }
```

Then reference crates by registry name:

```toml
[dependencies]
plugin-sdk = { version = "0.1", registry = "my-org" }
host-core  = { version = "0.1", registry = "my-org" }
```

### Publish to the custom registry

```bash
cargo publish -p plugin-capabilities --registry my-org
cargo publish -p plugin-manifest     --registry my-org
# … continue in dependency order
```

---

## Choosing the right option

| Scenario | Recommended option |
|---|---|
| Open-source release for everyone | Option A (automated crates.io) |
| Quick sharing / dev builds | Option B (git dependency) |
| Enterprise / private distribution | Option C (sparse registry on GitHub Pages) |
