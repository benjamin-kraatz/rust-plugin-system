# Publishing to crates.io

This guide walks you through publishing the Rust Plugin System crates to
[crates.io](https://crates.io), the official Rust package registry.  Once
published, anyone can add these crates as dependencies by version without
cloning this repository.

---

## Prerequisites

| Requirement | Notes |
|---|---|
| Rust toolchain | Install via [rustup.rs](https://rustup.rs) |
| crates.io account | Register at [crates.io](https://crates.io) |
| API token | `cargo login` (see below) |
| Clean working tree | All changes committed |

### Authenticate with crates.io

1. Log in to [crates.io](https://crates.io) with your GitHub account.
2. Go to **Account Settings → API Tokens → New Token**.
3. Copy the token and run:

   ```bash
   cargo login <YOUR_TOKEN>
   ```

   The token is stored in `~/.cargo/credentials.toml` and is used automatically
   by every subsequent `cargo publish` call.

---

## Crate dependency order

The crates form a dependency chain.  You **must** publish them in the order
shown below so that each crate's dependencies are already available on
crates.io when you publish it.

```
plugin-capabilities
  └── plugin-manifest
        └── plugin-protocol
              └── plugin-api
                    ├── plugin-sdk
                    ├── plugin-loader
                    │     └── plugin-runtime
                    │           └── host-core
                    ├── plugin-abi
                    └── plugin-wasm

plugin-test-kit  (depends on plugin-capabilities, plugin-manifest, plugin-protocol)
```

---

## Step-by-step publishing

### 1. Verify the workspace builds cleanly

```bash
cargo build --workspace --locked
cargo test --workspace --locked \
  --exclude host-egui \
  --exclude host-iced \
  --exclude host-dioxus-desktop
```

### 2. Dry-run each crate (recommended)

`cargo publish --dry-run` validates packaging without actually uploading:

```bash
cargo publish --dry-run -p plugin-capabilities
cargo publish --dry-run -p plugin-manifest
cargo publish --dry-run -p plugin-protocol
cargo publish --dry-run -p plugin-api
cargo publish --dry-run -p plugin-runtime
cargo publish --dry-run -p plugin-loader
cargo publish --dry-run -p plugin-sdk
cargo publish --dry-run -p plugin-abi
cargo publish --dry-run -p plugin-wasm
cargo publish --dry-run -p plugin-test-kit
cargo publish --dry-run -p host-core
```

Fix any errors or warnings before continuing.

### 3. Publish in dependency order

```bash
cargo publish -p plugin-capabilities
cargo publish -p plugin-manifest
cargo publish -p plugin-protocol
cargo publish -p plugin-api
cargo publish -p plugin-runtime
cargo publish -p plugin-loader
cargo publish -p plugin-sdk
cargo publish -p plugin-abi
cargo publish -p plugin-wasm
cargo publish -p plugin-test-kit
cargo publish -p host-core
```

> **Tip:** crates.io has a short propagation delay (~30 seconds) after
> publishing each crate.  If the next `cargo publish` fails because a
> dependency is not yet visible, wait a moment and retry.

### 4. Verify on crates.io

Each crate will be visible at `https://crates.io/crates/<crate-name>` and its
API documentation will be built automatically at `https://docs.rs/<crate-name>`.

---

## Bumping versions

All crates share the same version via `[workspace.package]` in the root
`Cargo.toml`.  To release a new version:

1. Update `version` in `Cargo.toml` (workspace root).
2. Update the pinned version in every crate's path dependency if you want to
   use exact versions (optional — the workspace resolver handles this
   automatically within the workspace).
3. Commit and tag:

   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

4. Publish in dependency order as shown above.

---

## Automating with GitHub Actions

See [`docs/publishing/github-packages.md`](./github-packages.md) for a
GitHub Actions workflow that publishes automatically on a new version tag.

---

## Troubleshooting

| Error | Fix |
|---|---|
| `error: crate name already exists on crates.io` | Choose a unique crate name in `Cargo.toml` |
| `error: missing required field 'description'` | Add `description` to the crate's `[package]` |
| `error: aborting upload due to previous errors` | Run `--dry-run` first to catch all issues |
| Dependency not yet visible | Wait ~30 s after publishing the dependency, then retry |
| `cargo login` token invalid | Generate a new token on crates.io and run `cargo login` again |
