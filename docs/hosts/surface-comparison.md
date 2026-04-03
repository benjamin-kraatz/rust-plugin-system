# Surface Comparison

Phase 3 turns the host layer into a real comparison exercise instead of a set of thin wrappers around the same runtime loader.

## What stays shared

All hosts build on the same host-side foundation in `crates/host-core`:

- `Playground` loads runtime plugins and exposes manifests plus invocation helpers
- `default_payload_text(action)` turns payload hints into UI-ready templates
- `supports_host(manifest, host)` lets hosts label plugins as a natural fit for their surface
- `render_response(response)` gives every host a consistent textual rendering fallback

That shared core makes it easier to compare the host surfaces honestly: the plugin model stays the same while the UX changes.

## How to run the comparison

Build once so every host can load plugins from `target/debug`:

```bash
cargo build --workspace
```

Then run the surfaces you want to compare:

```bash
cargo run -p host-tui
cargo run -p host-egui
cargo run -p host-iced
cargo run -p host-dioxus-desktop
cargo run -p host-web      # open http://127.0.0.1:4000
cargo run -p host-service  # call http://127.0.0.1:5000
```

A good comparison loop is to pick one plugin, keep the same action in mind, and compare how each host exposes payload templates, metadata, and results.

## Host-by-host comparison

| Host | Best for | Phase 3 behavior |
| --- | --- | --- |
| `host-cli` | baseline learning and scripting | direct list/inspect/run flow with no UI overhead |
| `host-tui` | terminal and SSH-friendly workflows | keyboard-first panes, action selection, inline payload editing, template loading, output review |
| `host-egui` | quick visual inspection | inspector/dashboard layout with manifest metadata, action templates, and output panels |
| `host-iced` | explicit state management | state/update-driven plugin, action, payload, and invoke flow |
| `host-dioxus-desktop` | reactive component-style desktop UI | reactive action studio with local invocation and output framing |
| `host-web` | browser-guided exploration | manifest catalog, payload composer, and rich result cards |
| `host-service` | automation and backend workflows | structured discovery, detail, example, and invoke JSON endpoints |

## Terminal and desktop hosts

### `host-tui`

This is the most obviously keyboard-first host in the repo.

- Tab cycles between plugin list, action list, and payload editor
- Up/Down change the current selection
- `Enter` loads the current action template
- `r` runs the selected action
- `t` reapplies the current template
- `c` clears the payload editor
- manifest metadata, payload hint previews, and output all stay visible in one terminal-driven workflow

Use it when you want to compare plugin metadata and invocation flow without leaving the terminal.

### `host-egui`

This host behaves like a plugin inspector.

- scrollable catalog of plugins
- manifest metadata panel with host-fit cues
- explicit action list
- editable payload box with template loading and clearing
- large output pane for quick visual iteration

Use it when you want the fastest visual feedback loop.

### `host-iced`

This host exists to show a more explicit state/update architecture.

- plugin selection, action selection, and payload editing all route through message handling
- payload state syncs from the selected action template
- metadata, status, and output are framed like a deliberate desktop application

Use it when you want to see how plugin workflows map into a more formal desktop state machine.

### `host-dioxus-desktop`

This host emphasizes reactive desktop composition.

- reactive plugin and action selection
- payload editing inside a desktop action studio
- local invocation with result and status rendering
- manifest details include supported hosts, tags, capabilities, notes, version, architecture, and skill level
- the crate uses an explicit Cargo `[[bin]]` target with `autobins = false`

Use it when you want to compare plugin systems against a modern Rust component model.

## Web and service hosts

### `host-web`

This host is for human-guided browsing in a browser.

- plugin catalog sidebar
- manifest metadata and capability display
- action browser with payload-hint previews
- payload composer with template loading and JSON formatting helpers
- richer result cards plus raw JSON expansion
- legacy GET invocation route for simple demos alongside the form-based workflow

Run it with:

```bash
cargo run -p host-web
```

Then open `http://127.0.0.1:4000`.

### `host-service`

This host is for automation-oriented plugin workflows.

Key endpoints:

- `GET /`
- `GET /health`
- `GET /catalog`
- `GET /examples`
- `GET /plugins`
- `GET /plugins/{plugin_id}`
- `GET /plugins/{plugin_id}/actions/{action_id}`
- `POST /plugins/{plugin_id}/actions/{action_id}/invoke`

The canonical invoke path is:

```text
POST /plugins/{plugin_id}/actions/{action_id}/invoke
```

Example:

```bash
curl -s http://127.0.0.1:5000/plugins/hello-world/actions/greet | jq
curl -s -X POST http://127.0.0.1:5000/plugins/hello-world/actions/greet/invoke \
  -H 'content-type: application/json' \
  -d '{"name":"Rustacean"}' | jq
```

Use it when you want machine-friendly discovery and invocation instead of an interactive UI.

## Framework trade-offs

| Host | Strength | Trade-off |
| --- | --- | --- |
| `host-egui` | quick inspector/dashboard iteration | less explicit state structure |
| `host-iced` | clear state/update model | more interaction boilerplate |
| `host-dioxus-desktop` | reactive component composition | larger framework/runtime surface |
| `host-tui` | strongest keyboard-first loop | terminal constraints on space and presentation |
| `host-web` | best human-friendly browsing and result framing | less suited to automation |
| `host-service` | best for scripts and backend integration | no interactive UI |

## Suggested comparison workflow

1. Start with `host-cli` to establish the plugin manifest and action baseline.
2. Move to `host-tui` to compare a terminal-native workflow against the same plugins.
3. Compare `host-egui`, `host-iced`, and `host-dioxus-desktop` using the same plugin and payload template.
4. Compare `host-web` and `host-service` to see the split between human-guided browser UX and automation-first API design.

## Important note about host support

The `supported_hosts` field is currently used as **host-fit metadata**, not as a hard execution gate. Hosts surface that metadata so readers can compare intent and UX, but they do not block invocation solely because a plugin is labeled for another host.
