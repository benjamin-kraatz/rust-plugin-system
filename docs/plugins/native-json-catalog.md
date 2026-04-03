# Native JSON Plugin Catalog

This page is the course-style catalog for the runtime-loaded **native JSON** plugins used throughout the playground.

Use it in this order:

1. list the plugins with `cargo run -p host-cli -- list`
2. inspect a manifest with `cargo run -p host-cli -- inspect <plugin-id>`
3. run one action with `cargo run -p host-cli -- run <plugin-id> <action-id> '<json>'`

## Foundation pack

These are the first plugins to try when you want to understand the native JSON model before moving into the more practical Phase 2 set.

| Plugin | What it demonstrates | Try this action |
| --- | --- | --- |
| `hello-world` | the smallest runtime-loaded plugin contract | `greet` |
| `logger` | structured operational output from a plugin | `emit-demo-log` |
| `formatter` | JSON formatting and host-friendly output rendering | `pretty-json` |
| `transformer` | deterministic text transformation | `slugify` |
| `command-pack` | curated command suggestions for host workflows | `suggest-commands` |
| `ui-panel` | dashboard-style content for visual hosts | `dashboard-snapshot` |

## Phase 2 practical pack

Phase 2 expands the native catalog from toy examples into host-shaped utility plugins.

| Plugin | What it demonstrates | Actions |
| --- | --- | --- |
| `config-provider` | layered config resolution, dotted-path lookup, flattening | `merge-layers`, `get-value`, `flatten-config` |
| `filesystem-tools` | workspace-safe file and directory inspection | `list-directory`, `read-text-file`, `describe-path` |
| `data-pipeline` | record projection, filtering, and summarization | `project-records`, `filter-records`, `summarize-field` |
| `metrics-observer` | deterministic metrics rollups and SLO budget checks | `summarize-signals`, `evaluate-slo` |
| `service-hooks` | service-hook rollout planning and delivery previews | `plan-hook-set`, `preview-delivery` |
| `tui-tools` | terminal layout planning and status-line composition | `plan-layout`, `draft-status-line` |

## Running from `host-cli`

Build once so the host can find the dynamic libraries:

```bash
cargo build --workspace
```

Then use the CLI host as your reference runner:

```bash
cargo run -p host-cli -- list
cargo run -p host-cli -- inspect config-provider
cargo run -p host-cli -- run config-provider merge-layers '{"defaults":{"service":{"port":8080}},"environment":{"service":{"port":8081}},"overrides":{"service":{"host":"127.0.0.1"}}}'
```

## `config-provider`

**What it demonstrates**

- config layering with the precedence `defaults < environment < overrides`
- dotted-path lookup, including array indexes
- flattening nested JSON into a dotted-path map

**Actions**

- `merge-layers`
- `get-value`
- `flatten-config`

**What the actions return**

- `merge-layers` returns both the merged config and a flattened view
- `get-value` returns the resolved value for one dotted path
- `flatten-config` returns the flattened map plus a text list of discovered paths

**Run it**

```bash
cargo run -p host-cli -- run config-provider merge-layers '{"defaults":{"service":{"port":8080}},"environment":{"service":{"port":8081}},"overrides":{"service":{"host":"127.0.0.1"}}}'
cargo run -p host-cli -- run config-provider get-value '{"config":{"service":{"port":8081,"host":"127.0.0.1"}},"path":"service.port"}'
cargo run -p host-cli -- run config-provider flatten-config '{"config":{"service":{"host":"127.0.0.1","port":8081}}}'
```

## `filesystem-tools`

**What it demonstrates**

- workspace-relative directory discovery
- bounded text-file previews
- path metadata inspection

**Important boundary**

This plugin is read-only. Paths are resolved against the invocation workspace root and rejected when they escape it.

**Actions**

- `list-directory`
- `read-text-file`
- `describe-path`

**Run it**

```bash
cargo run -p host-cli -- run filesystem-tools list-directory '{"path":"plugins","max_depth":1,"max_entries":8}'
cargo run -p host-cli -- run filesystem-tools read-text-file '{"path":"README.md","max_bytes":512}'
cargo run -p host-cli -- run filesystem-tools describe-path '{"path":"plugins/config-provider"}'
```

## `data-pipeline`

**What it demonstrates**

- selecting stable subsets of fields from record sets
- filtering records with simple deterministic predicates
- summarizing one field across a dataset

**Supported filter operators**

- `eq`
- `neq`
- `contains`
- `gt`
- `lt`

**Actions**

- `project-records`
- `filter-records`
- `summarize-field`

**Run it**

```bash
cargo run -p host-cli -- run data-pipeline project-records '{"records":[{"id":1,"name":"Ada","team":"platform"}],"fields":["id","name"]}'
cargo run -p host-cli -- run data-pipeline filter-records '{"records":[{"status":"active","duration":12.5},{"status":"draft","duration":7.5}],"predicate":{"field":"status","op":"eq","value":"active"}}'
cargo run -p host-cli -- run data-pipeline summarize-field '{"records":[{"duration":12.5},{"duration":7.5}],"field":"duration"}'
```

## `metrics-observer`

**What it demonstrates**

- deterministic signal rollups for request volume, latency, and saturation
- simple SLO error-budget evaluation for host demos and smoke tests

**Actions**

- `summarize-signals`
- `evaluate-slo`

**What the actions return**

- `summarize-signals` returns a JSON snapshot plus a markdown operator summary
- `evaluate-slo` returns budget math and a compact text verdict such as `status=breached`

**Run it**

```bash
cargo run -p host-cli -- run metrics-observer summarize-signals '{"service":"checkout","window_minutes":15,"samples":{"requests":1800,"errors":18,"p95_ms":245,"saturation_pct":61}}'
cargo run -p host-cli -- run metrics-observer evaluate-slo '{"service":"checkout","objective_pct":99.5,"window_requests":1800,"window_errors":18}'
```

## `service-hooks`

**What it demonstrates**

- planning a deterministic set of hook endpoints from service and event inputs
- previewing a webhook delivery envelope without making a network call

**Actions**

- `plan-hook-set`
- `preview-delivery`

**Implemented details worth noticing**

- endpoints use the pattern `{base}/v1/{service}/{environment}/{event-slug}`
- retries are planned, but the plugin only returns preview data
- delivery previews include headers, request body, and computed backoff timing

**Run it**

```bash
cargo run -p host-cli -- run service-hooks plan-hook-set '{"service":"billing","environment":"staging","events":["deploy.succeeded","incident.opened"],"target_base_url":"https://hooks.internal.example"}'
cargo run -p host-cli -- run service-hooks preview-delivery '{"service":"billing","event":"deploy.succeeded","attempt":2,"target_base_url":"https://hooks.internal.example"}'
```

## `tui-tools`

**What it demonstrates**

- deterministic pane geometry for terminal workspaces
- compact status-line generation from structured state

**Actions**

- `plan-layout`
- `draft-status-line`

**What the actions return**

- `plan-layout` returns both a text layout plan and a JSON pane description
- `draft-status-line` returns a terminal-safe line such as `[NORMAL] | rust-plugin-system | branch:main | focus:editor | pending:2 | state:clean`

**Run it**

```bash
cargo run -p host-cli -- run tui-tools plan-layout '{"width":120,"height":36,"panes":["nav","editor","logs"],"focus":"editor","include_help":true}'
cargo run -p host-cli -- run tui-tools draft-status-line '{"mode":"NORMAL","workspace":"rust-plugin-system","branch":"main","dirty":false,"pending_tasks":2,"focus":"editor"}'
```

## Where to go next

- Use `docs/getting-started/quickstart.md` for a shorter first pass.
- Use `docs/snippets/cli-recipes.md` when you want copy-paste command sequences.
- Use `docs/advanced/roadmap.md` after the native catalog feels familiar.
