# kellnr — Self-Hosted Cargo Registry

[kellnr](https://kellnr.io) is an open-source, self-hosted Cargo registry for
Rust crates.  It provides a web UI, a crates.io-compatible API, sparse index
support, user management, public and private crates, and optional transparent
caching of crates.io packages.

This project's shared crates are published to a kellnr instance running at
`https://crates.d-zwei.de`.

---

## Table of contents

1. [Running kellnr with Docker](#running-kellnr-with-docker)
2. [Docker Compose](#docker-compose)
3. [Deploying on Coolify](#deploying-on-coolify)
4. [First-time setup](#first-time-setup)
5. [Configuration reference](#configuration-reference)
6. [User and token management](#user-and-token-management)
7. [Crate visibility (public vs private)](#crate-visibility-public-vs-private)
8. [Consuming packages](#consuming-packages)
9. [Publishing packages](#publishing-packages)
10. [Web UI overview](#web-ui-overview)
11. [crates.io caching](#cratesio-caching)
12. [Yanking and deleting versions](#yanking-and-deleting-versions)
13. [Edge cases and troubleshooting](#edge-cases-and-troubleshooting)
14. [Advanced topics](#advanced-topics)

---

## Running kellnr with Docker

The fastest way to get kellnr running is the official Docker image:

```bash
docker run -d \
  --name kellnr \
  -p 8000:8000 \
  -v kellnr-data:/var/lib/kellnr \
  -e KELLNR_ORIGIN__HOSTNAME=localhost \
  -e KELLNR_ORIGIN__PORT=8000 \
  -e KELLNR_ORIGIN__PROTOCOL=http \
  ghcr.io/kellnr/kellnr:6
```

Open `http://localhost:8000` and log in with the default admin credentials
(`admin` / `admin`) — change the password immediately after first login.

> **Tip:** For a production deployment, replace `localhost` with your actual
> domain, set `PROTOCOL=https`, and run kellnr behind a reverse proxy (see
> [Advanced topics](#advanced-topics)).

---

## Docker Compose

A minimal `docker-compose.yml` for a self-contained deployment:

```yaml
services:
  kellnr:
    image: ghcr.io/kellnr/kellnr:6
    restart: unless-stopped
    ports:
      - "8000:8000"
    volumes:
      - kellnr-data:/var/lib/kellnr
    environment:
      KELLNR_ORIGIN__HOSTNAME: crates.d-zwei.de
      KELLNR_ORIGIN__PORT: 443
      KELLNR_ORIGIN__PROTOCOL: https
      # Optional: enable crates.io caching (recommended)
      KELLNR_REGISTRY__CACHE_NUM_THREADS: 4

volumes:
  kellnr-data:
```

```bash
docker compose up -d
```

---

## Deploying on Coolify

[Coolify](https://coolify.io) is a self-hosted PaaS that makes container
deployments simple.  The following steps deploy kellnr as a Docker image
resource.

### Step 1 — Create a new resource

1. Open your Coolify dashboard and navigate to your project.
2. Click **Add Resource → Docker Image**.
3. Enter the image: `ghcr.io/kellnr/kellnr:6`.

### Step 2 — Ports

In the **Network** tab, expose port `8000`.  If you are putting a Traefik
reverse proxy in front (Coolify's default), you do not need to expose this
port publicly — Coolify handles routing.

### Step 3 — Persistent storage

kellnr stores its database, crate files, and index in `/var/lib/kellnr`.
Add a persistent volume:

| Mount path | Volume name |
|---|---|
| `/var/lib/kellnr` | `kellnr-data` (or any name) |

Without this, all data is lost on container restart.

### Step 4 — Environment variables

Add the following environment variables in the **Environment** tab:

| Variable | Value | Notes |
|---|---|---|
| `KELLNR_ORIGIN__HOSTNAME` | `crates.d-zwei.de` | Your domain |
| `KELLNR_ORIGIN__PORT` | `443` | 443 for HTTPS |
| `KELLNR_ORIGIN__PROTOCOL` | `https` | Use `http` for local-only |
| `KELLNR_REGISTRY__CACHE_NUM_THREADS` | `4` | Enables crates.io caching |

### Step 5 — Custom domain

In the **Domains** tab, add your domain (`crates.d-zwei.de`).  Coolify will
provision a Let's Encrypt certificate automatically if you have a wildcard DNS
record pointing to your server.

### Step 6 — Deploy

Click **Deploy**.  After the container starts, open `https://crates.d-zwei.de`
and complete [first-time setup](#first-time-setup).

---

## First-time setup

### Change the admin password

Log in at `https://crates.d-zwei.de` with `admin` / `admin`.

1. Click the user icon (top right) → **Settings**.
2. Enter a new password and save.

### Create additional users

1. Go to **Admin** → **Users** → **Add user**.
2. Assign the **admin** role for users who can manage the registry, or leave
   as a regular user for publish-only or read-only access.

### Create an API token

Tokens are per-user and are used by `cargo login` and CI pipelines.

1. Click the user icon → **Settings** → **Tokens** → **Add token**.
2. Give the token a name (e.g., `ci-publish`) and click **Create**.
3. **Copy the token immediately** — it is shown only once.

---

## Configuration reference

kellnr is configured entirely through environment variables.  Key options:

| Variable | Default | Description |
|---|---|---|
| `KELLNR_ORIGIN__HOSTNAME` | `localhost` | Public hostname of the registry |
| `KELLNR_ORIGIN__PORT` | `8000` | Public port (use 443 for HTTPS behind a proxy) |
| `KELLNR_ORIGIN__PROTOCOL` | `http` | `http` or `https` |
| `KELLNR_REGISTRY__DATA_DIR` | `/var/lib/kellnr` | Where crate files and the index are stored |
| `KELLNR_REGISTRY__CACHE_NUM_THREADS` | `0` | Set to ≥ 1 to enable crates.io caching |
| `KELLNR_REGISTRY__MAX_CRATE_SIZE` | `10` | Max upload size in MB |
| `KELLNR_REGISTRY__AUTH_REQUIRED` | `false` | Require auth for all read operations |
| `KELLNR_LOG__LEVEL` | `info` | Log level: `trace`, `debug`, `info`, `warn`, `error` |

A full list of variables is in the
[kellnr documentation](https://kellnr.io/documentation).

---

## User and token management

### Roles

| Role | Capabilities |
|---|---|
| **admin** | Everything: manage users, delete crates, change settings |
| **user** | Publish crates, create tokens, read all public crates |

### Tokens

Each user can have multiple named tokens.  Tokens do not have fine-grained
scopes in the current kellnr version — any valid token for a user grants full
publish access for that user.

To revoke a token, go to **Settings → Tokens** and click the trash icon next
to the token name.

---

## Crate visibility (public vs private)

After publishing, each crate defaults to **private** — visible only to
authenticated users.

To make a crate public:

1. Open the crate's page in the kellnr web UI.
2. Click **Settings** (visible to admins and the crate owner).
3. Toggle **Visibility** to **Public**.

Public crates can be downloaded without a token.  This is the correct setting
for the plugin system's shared crates, which are intended for external
consumers.

> **Note:** Visibility is per-crate, not per-version.  Making a crate public
> exposes all its versions.

---

## Consuming packages

### Add the registry to your project

Create or edit `.cargo/config.toml` at the project root (or in `~/.cargo/`
for user-wide config):

```toml
[registries.dzwei-registry]
index = "sparse+https://crates.d-zwei.de/api/v1/crates/"
```

### Declare dependencies

```toml
[dependencies]
plugin-sdk  = { version = "0.1", registry = "dzwei-registry" }
host-core   = { version = "0.1", registry = "dzwei-registry" }
```

### Authenticate (private crates only)

If `AUTH_REQUIRED` is enabled or the crate is private, run:

```bash
cargo login --registry dzwei-registry <YOUR_TOKEN>
```

This stores the token in `~/.cargo/credentials.toml`.  In CI, set the
environment variable instead:

```bash
export CARGO_REGISTRIES_DZWEI_REGISTRY_TOKEN=<YOUR_TOKEN>
```

Or in GitHub Actions:

```yaml
env:
  CARGO_REGISTRIES_DZWEI_REGISTRY_TOKEN: ${{ secrets.DZWEI_CRATES_REG_TOKEN }}
```

---

## Publishing packages

This project publishes both crate archives and rustdoc documentation to
Kellnr. After the crate versions are visible in the registry, rustdoc is built
from the workspace root, zipped from `target/doc`, and uploaded to the Kellnr
docs API.

The raw upload shape is:

```bash
curl -H "Authorization: <TOKEN>" \
  https://crates.d-zwei.de/api/v1/docs/<crate>/<version> \
  --upload-file target/doc.zip
```

The same token used for crate publishing is used for docs uploads. In this
repository, `scripts/upload-kellnr-docs.sh` performs that upload, and
`scripts/publish-shared-crates.sh` calls it automatically after the crates have
been published.

### Manual publish

```bash
# Authenticate once
cargo login --registry dzwei-registry <YOUR_TOKEN>

# Dry run (packages but does not upload)
cargo publish --dry-run --registry dzwei-registry -p plugin-sdk

# Publish
cargo publish --registry dzwei-registry -p plugin-sdk
```

### Publish via CI

This repository ships `.github/workflows/publish-crates.yml`.

Trigger it from the **Actions** tab or by creating a GitHub Release.  See
[`docs/guides/publishing.md`](./publishing.md) for the full workflow reference.

### Dependency resolution during publish

`cargo publish` converts workspace path-deps to version-reqs in the packaged
`Cargo.toml`, then checks that those versions are available in a registry.
For this to work with workspace-internal deps (e.g., `plugin-manifest` depends
on `plugin-capabilities`):

1. **Publish in the correct order** (leaf crates first — see
   [publishing.md](./publishing.md#recommended-release-order)).
2. **Enable crates.io caching** on your kellnr instance so external deps
   (serde, anyhow, etc.) resolve without requiring crates.io access.
3. The CI workflow sets `dzwei-registry` as the default registry, so the
   version-req check queries kellnr (where the crate is already published)
   rather than crates.io.

---

## Web UI overview

The kellnr web UI is available at `https://crates.d-zwei.de`.

| Section | What you can do |
|---|---|
| **Home / Search** | Browse all crates, search by name or keyword |
| **Crate page** | View versions, README, dependencies, download stats |
| **Crate → Settings** | Toggle public/private, transfer ownership |
| **Crate → Versions** | Yank or un-yank individual versions |
| **Admin → Users** | Add/remove users, change roles |
| **Admin → Settings** | Runtime configuration (some fields are read-only) |
| **User → Settings** | Change password, manage API tokens |

---

## crates.io caching

kellnr can act as a transparent cache for crates.io.  When enabled:

- Requests for unknown crates are forwarded to crates.io and cached locally.
- Consumers can set kellnr as their **only** registry source — no need to
  configure crates.io separately.
- The CI workflow relies on this feature to resolve external deps without
  touching crates.io directly.

### Enable caching

Set the environment variable before starting kellnr:

```bash
KELLNR_REGISTRY__CACHE_NUM_THREADS=4
```

Higher values allow more parallel upstream fetches.  For a single-server
deployment, `4` is a good starting point.

### Verify caching is working

After enabling, install a crates.io crate through kellnr:

```toml
# .cargo/config.toml
[registry]
default = "dzwei-registry"
```

```bash
cargo add serde  # should download via kellnr, not crates.io directly
```

The crate will appear in the kellnr web UI under the **Cached** section.

---

## Yanking and deleting versions

### Yank a version

Yanking marks a version as "do not use for new resolutions" without removing
it.  Existing projects that already have the yanked version in their
`Cargo.lock` continue to work.

```bash
cargo yank --registry dzwei-registry --version 0.1.0 plugin-sdk
```

Or use the web UI: crate page → **Versions** → yank icon.

To un-yank:

```bash
cargo yank --registry dzwei-registry --version 0.1.0 --undo plugin-sdk
```

**Prefer yanking over deletion** for broken releases — deletion breaks
existing lock files.

### Delete a version

Deletion is permanent and breaks any project that has the deleted version in
its `Cargo.lock`.  Only admins can delete via the web UI.

Use deletion only when a version contains sensitive data that was accidentally
published.

---

## Edge cases and troubleshooting

### "no matching package named X found"

This happens when `cargo publish` tries to resolve a workspace dep against
the wrong registry.  Fix: publish in the correct order (leaf crates first)
and ensure kellnr's crates.io caching is enabled.

### "already uploaded"

kellnr rejects re-publishing an existing version.  Bump the version in
`Cargo.toml` or yank the old version first if you need to replace it.

### Token not accepted

- Verify the token is correct (`cargo login --registry dzwei-registry <token>`).
- Check that the token has not been revoked in the kellnr web UI.
- In CI, confirm the `DZWEI_CRATES_REG_TOKEN` secret is set and the env var
  `CARGO_REGISTRIES_DZWEI_REGISTRY_TOKEN` is wired to it.

### Sparse index not updating

Cargo caches the registry index locally at
`~/.cargo/registry/index/`.  Force a refresh:

```bash
cargo update --registry dzwei-registry
```

Or delete the cached index:

```bash
rm -rf ~/.cargo/registry/index/*crates.d-zwei.de*
```

### Private crate downloaded without auth

Check `KELLNR_REGISTRY__AUTH_REQUIRED`.  If it is `false` (the default), the
index and public crates are readable without a token.  Only private crates
enforce authentication.

---

## Advanced topics

### Running behind a reverse proxy (Nginx)

```nginx
server {
    listen 443 ssl;
    server_name crates.d-zwei.de;

    ssl_certificate     /etc/letsencrypt/live/crates.d-zwei.de/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/crates.d-zwei.de/privkey.pem;

    client_max_body_size 50M;  # Allow large .crate uploads

    location / {
        proxy_pass         http://127.0.0.1:8000;
        proxy_set_header   Host              $host;
        proxy_set_header   X-Real-IP         $remote_addr;
        proxy_set_header   X-Forwarded-For   $proxy_add_x_forwarded_for;
        proxy_set_header   X-Forwarded-Proto $scheme;
    }
}
```

Set `KELLNR_ORIGIN__PORT=443` and `KELLNR_ORIGIN__PROTOCOL=https` so kellnr
generates correct download URLs.

### Coolify + Traefik (no separate Nginx needed)

Coolify's built-in Traefik proxy handles TLS termination automatically.
Set the environment variables as shown in
[Deploying on Coolify](#deploying-on-coolify) and add the custom domain in
Coolify's **Domains** tab — no Nginx config required.

### Backup

All persistent state is in the volume mounted at `/var/lib/kellnr`.  Back up
this directory:

```bash
# Stop kellnr first for a consistent snapshot
docker stop kellnr
tar -czf kellnr-backup-$(date +%Y%m%d).tar.gz /path/to/kellnr-data
docker start kellnr
```

For a live backup (data may be slightly inconsistent), copy the directory
while kellnr is running — kellnr uses SQLite, which is safe to copy with
the WAL journal present.

### Upgrading kellnr

```bash
docker pull ghcr.io/kellnr/kellnr:6
docker stop kellnr
docker rm kellnr
docker run -d --name kellnr \
  -p 8000:8000 \
  -v kellnr-data:/var/lib/kellnr \
  -e KELLNR_ORIGIN__HOSTNAME=crates.d-zwei.de \
  -e KELLNR_ORIGIN__PORT=443 \
  -e KELLNR_ORIGIN__PROTOCOL=https \
  ghcr.io/kellnr/kellnr:6
```

kellnr runs database migrations automatically on startup.  Always back up
before upgrading.

On Coolify, click **Redeploy** after pulling the latest image tag — Coolify
handles the container lifecycle.

### Pinning to a specific kellnr version

Replace `latest` with a version tag, e.g., `ghcr.io/kellnr/kellnr:5.0.0`.
Check available tags at
[github.com/kellnr/kellnr/releases](https://github.com/kellnr/kellnr/releases).

---

## Related docs

- [`docs/guides/publishing.md`](./publishing.md) — publish workflow for this project's crates
- [`docs/guides/external-host-plugin.md`](./external-host-plugin.md) — external project setup
- [kellnr official documentation](https://kellnr.io/documentation)
- [kellnr GitHub repository](https://github.com/kellnr/kellnr)
