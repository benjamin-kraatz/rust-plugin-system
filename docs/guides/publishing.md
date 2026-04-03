# Publishing the Shared Crates

This workspace supports two publishing paths:

1. the standard crates.io path
2. an alternate Cargo registry backed by GitHub Packages

## Before you publish

Make sure the package you are publishing has:

- a version
- a license
- a repository URL
- a README
- a `publish` policy that allows the intended registry

The shared crates in this workspace already include the metadata they need.

## Common path: crates.io

This is the default and the most common release flow.

```bash
cargo login
cargo publish --dry-run -p plugin-sdk
cargo publish -p plugin-sdk
```

Use the same sequence for the other reusable crates:

- `plugin-capabilities`
- `plugin-manifest`
- `plugin-protocol`
- `plugin-api`
- `plugin-sdk`
- `plugin-abi`
- `plugin-loader`
- `plugin-runtime`
- `plugin-wasm`
- `host-core`
- `plugin-test-kit`

## Alternate path: self-hosted registry on GitHub

This repository ships a GitHub Actions workflow that maintains a **sparse Cargo
registry** backed by two built-in GitHub features:

| Concern | GitHub feature | URL |
|---------|---------------|-----|
| Index   | GitHub Pages  | `https://<owner>.github.io/<repo>/cargo/` |
| Crates  | Releases      | one release per crate version, tagged `<crate>-<version>` |

No external hosting or third-party services are required.

### One-time repository setup

1. In **Settings → Pages**, set *Source* to **Deploy from a branch** and select
   the `gh-pages` branch (root folder).
2. That is all – the first workflow run initialises the branch and the index
   automatically.

### Running the publish workflow

Trigger `.github/workflows/publish-github-packages.yml` manually from the
**Actions** tab.  The `dry_run` toggle (default `true`) lets you preview what
would be published without uploading anything.

The workflow also runs automatically whenever you publish a **GitHub Release**.

### Consuming packages from this registry

Add the registry to `.cargo/config.toml` in the consuming project (replace
`<owner>` and `<repo>` with the actual values):

```toml
[registries.plugin-system]
index = "sparse+https://<owner>.github.io/<repo>/cargo/"
```

Then add dependencies normally, naming the registry:

```toml
[dependencies]
plugin-sdk    = { version = "0.1", registry = "plugin-system" }
host-core     = { version = "0.1", registry = "plugin-system" }
```

Authenticate with a GitHub personal access token that has `read:packages`
scope (or the default `GITHUB_TOKEN` in Actions workflows):

```bash
cargo login --registry plugin-system <YOUR_GITHUB_TOKEN>
```

### Restricting a crate to this registry only

If you want to prevent accidental publication to crates.io, add a `publish`
allowlist in the crate's `Cargo.toml`:

```toml
[package]
publish = ["plugin-system"]
```

## Recommended release order

Publish the shared contract crates first:

1. `plugin-capabilities`
2. `plugin-manifest`
3. `plugin-protocol`
4. `plugin-api`
5. `plugin-sdk`
6. `plugin-abi`
7. `plugin-loader`
8. `plugin-runtime`
9. `plugin-wasm`
10. `host-core`
11. `plugin-test-kit`

That keeps downstream host and plugin crates on the same version line.

## Notes

- Cargo’s alternate-registry flow is documented in the Cargo book.
- If you dual-publish, keep dependency versions aligned across registries.
- Avoid adding workspace-only test fixtures to publishable crates.

## Related docs

- [`docs/guides/external-host-plugin.md`](./external-host-plugin.md)
- [`docs/reference/testing-packaging.md`](../reference/testing-packaging.md)
- [`docs/reference/version-compatibility.md`](../reference/version-compatibility.md)
