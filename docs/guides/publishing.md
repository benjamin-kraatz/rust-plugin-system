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

## Alternate path: GitHub Packages

Cargo supports publishing to an alternate registry. Configure the registry index in `.cargo/config.toml`:

```toml
[registries]
github = { index = "sparse+https://<your-github-packages-registry-index>" }

[registry]
default = "github"
```

Then authenticate and publish:

```bash
cargo login --registry github
cargo publish --dry-run --registry github -p plugin-sdk
cargo publish --registry github -p plugin-sdk
```

If you want to lock a crate to a single registry, add:

```toml
[package]
publish = ["github"]
```

## Consuming packages from GitHub Packages

In downstream projects, declare the registry on the dependency:

```toml
[dependencies]
plugin-sdk = { version = "0.1", registry = "github" }
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
