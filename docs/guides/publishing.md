# Publishing the Shared Crates

The shared crates in this workspace are published to the project's self-hosted
[kellnr](https://kellnr.io) Cargo registry at `https://crates.your-domain.com`.

See [`docs/guides/kellnr.md`](./kellnr.md) for full information on setting up
and administrating the registry.

## Consumer setup

Add the registry to `.cargo/config.toml` in any project that depends on these
crates:

```toml
[registries.dzwei-registry]
index = "sparse+https://crates.your-domain.com/api/v1/crates/"
```

Then declare dependencies as usual, naming the registry:

```toml
[dependencies]
plugin-sdk    = { version = "0.1", registry = "dzwei-registry" }
host-core     = { version = "0.1", registry = "dzwei-registry" }
```

If the registry requires authentication to read (private crates), log in once
with a read-scoped token:

```bash
cargo login --registry dzwei-registry <YOUR_TOKEN>
```

## Publishing manually

You need a kellnr API token with **publish** permission.

```bash
# Authenticate (stored in ~/.cargo/credentials.toml)
cargo login --registry dzwei-registry <YOUR_TOKEN>

# Dry run — packages the crate but does not upload
cargo publish --dry-run --registry dzwei-registry -p plugin-sdk

# Publish for real
cargo publish --registry dzwei-registry -p plugin-sdk
```

## Recommended release order

Publish leaf crates first so that each subsequent crate can resolve its
workspace-internal dependencies from the registry:

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

## Publishing via CI

The repository ships `.github/workflows/publish-crates.yml`.  Trigger it
manually from the **Actions** tab, or it runs automatically when you publish a
GitHub Release.

The `dry_run` toggle (default `true`) packages each crate and reports what
would be uploaded without actually pushing anything.  Set it to `false` for a
real publish.

**Required GitHub secret:** `DZWEI_CRATES_REG_TOKEN` — a kellnr API token with
publish permission.  Add it in *Settings → Secrets and variables → Actions*.

The workflow:
1. Configures `dzwei-registry` as the default Cargo registry so that
   workspace-internal path-dep version checks resolve against kellnr rather
   than crates.io (kellnr's crates.io caching handles external deps like serde).
2. Publishes each crate in dependency order, skipping versions already present
   in the registry.

## Notes

- Avoid adding workspace-only test fixtures to publishable crates.
- If you need to pull back a broken release, yank the version from the kellnr
  web UI rather than deleting it — yanked versions satisfy existing lock files
  but are not chosen for new resolutions.

## Related docs

- [`docs/guides/kellnr.md`](./kellnr.md) — registry setup, admin, consumer config
- [`docs/guides/versioning.md`](./versioning.md) — versioning strategy, cargo-release, git-cliff, local and CI release workflows
- [`docs/guides/external-host-plugin.md`](./external-host-plugin.md) — external project setup
- [`docs/reference/testing-packaging.md`](../reference/testing-packaging.md)
- [`docs/reference/version-compatibility.md`](../reference/version-compatibility.md)
