# Rich Host Surfaces

Phase 3 turns the host apps into a deliberate comparison set. Each host still loads the same plugin manifests and actions, but each one now teaches a different lesson about discovery, payload authoring, and result presentation.

## Shared baseline

The Phase 3 hosts now converge on a common workflow:

1. choose a plugin
2. choose an action
3. load or edit a payload template
4. invoke the action
5. compare the result in a host-specific output surface

Shared host-core helpers make that consistency visible:

- `default_payload_text()` turns action payload hints into editor-friendly starter JSON
- `supports_host()` lets hosts explain whether a manifest is a good fit for that surface

## How to run the comparison set

Build once so the hosts can load plugins from `target/debug`:

```bash
cargo build --workspace
```

Then compare the surfaces with the same plugin/action in mind:

```bash
cargo run -p host-tui
cargo run -p host-egui
cargo run -p host-iced
cargo run -p host-dioxus-desktop
cargo run -p host-web      # open http://127.0.0.1:4000
cargo run -p host-service  # call http://127.0.0.1:5000
```

A good comparison loop is:

1. inspect the same plugin in `host-tui`, `host-egui`, and `host-iced`
2. compare the same action payload in `host-dioxus-desktop`
3. compare `host-web` and `host-service` to see browser-first vs automation-first design

## Surface-by-surface comparison

### `host-tui`

**Use it for:** terminal-native exploration, keyboard workflows, and quick comparison without switching to a browser or desktop shell.

**What it feels like:** a multi-pane terminal workbench. You move between plugin selection, action selection, payload editing, and output review with the keyboard.

**Current behaviors:**

- plugin list and action list are separate panes
- `Enter` loads the selected action template
- payload editing happens inline in the terminal
- `r` runs the selected action, `t` reapplies the template, `c` clears the payload
- manifest details include architecture, skill level, supported hosts, tags, capabilities, notes, and a payload hint preview

**Tradeoff:** fastest keyboard loop, but the terminal is still the terminal; large payloads and rich output are more cramped than on the web or desktop hosts.

### `host-egui`

**Use it for:** fast visual inspection and metadata-heavy browsing.

**What it feels like:** an inspector/dashboard. The UI is optimized for scanning cards, metadata, and actions quickly.

**Current behaviors:**

- manifest list with host-fit cues and summary badges
- selected plugin metadata shown in a grid
- action selection beside a multiline payload editor
- one-click template loading, clearing, and invocation
- dedicated output panel for comparing results

**Tradeoff:** easiest desktop host to browse casually, but its immediacy is less explicit about state transitions than `host-iced`.

### `host-iced`

**Use it for:** comparing plugin workflows in a more explicit application-state architecture.

**What it feels like:** a desktop app where state changes are part of the point. Selection, template sync, and invocation are spelled out in the update flow.

**Current behaviors:**

- plugin selection and action selection both feed explicit state
- payload editor is synchronized from the selected action template
- metadata includes architecture, skill, supported hosts, tags, capabilities, and notes
- status/output framing makes action execution feel deliberate and app-like

**Tradeoff:** less dashboard-like than `egui`, but better when you want the framework's state model to be visible in the host design.

### `host-dioxus-desktop`

**Use it for:** trying the same plugin workflow in a reactive component-style desktop surface.

**What it feels like:** a reactive desktop UI with browser-like composition patterns.

**Current behaviors:**

- reactive plugin and action selection
- payload template loading and inline editing
- manifest details include supported hosts, tags, capabilities, notes, architecture, version, and skill level
- local invocation output and status messaging
- explicit Cargo `[[bin]]` target with `autobins = false` in `hosts/host-dioxus-desktop/Cargo.toml`

**Tradeoff:** good for comparing reactive composition with Rust-native desktop approaches, but it is still a local desktop host rather than a multi-user or remote surface.

## Web vs service

The biggest Phase 3 learning jump is the split between browser-first and API-first hosts.

### `host-web`

**Use it for:** human-guided browsing in the browser.

**What it feels like:** a catalog plus composer. You browse plugins, inspect manifest badges and notes, choose an action, load an example payload, edit JSON, and then review output cards.

**Current behaviors:**

- plugin catalog sidebar
- manifest badges for id, version, architecture, skill level, and supported hosts
- action browser with payload-hint previews
- browser payload composer with example-loading and JSON formatting helpers
- result cards that render mixed plugin outputs more readably than plain text
- legacy GET invocation path remains available for simple URL-driven demos

**Tradeoff:** best for demos, teaching, and manual exploration; not the host you would choose for automation.

### `host-service`

**Use it for:** scripts, service orchestration, and machine-friendly discovery/invocation.

**What it feels like:** a small comparison API, not a UI.

**Current behaviors:**

- `GET /` for overview and endpoint framing
- `GET /health` for loader status
- `GET /catalog` for a high-level plugin catalog
- `GET /examples` for curated payload examples
- `GET /plugins` for summaries
- `GET /plugins/{plugin_id}` for manifest details
- `GET /plugins/{plugin_id}/actions/{action_id}` for per-action detail plus example payloads
- `POST /plugins/{plugin_id}/actions/{action_id}/invoke` as the canonical invoke endpoint

Example invoke:

```bash
curl -s http://127.0.0.1:5000/plugins/hello-world/actions/greet | jq
curl -s -X POST http://127.0.0.1:5000/plugins/hello-world/actions/greet/invoke \
  -H 'content-type: application/json' \
  -d '{"name":"Rustacean"}' | jq
```

**Tradeoff:** ideal for integration and repeatability, but intentionally less friendly than `host-web` for browsing and explanation.

## Which host should you start with?

- Start with `host-tui` if you live in the terminal.
- Start with `host-egui` if you want the quickest visual overview.
- Start with `host-iced` if framework architecture is part of what you are studying.
- Start with `host-dioxus-desktop` if you want a reactive desktop comparison point.
- Start with `host-web` if the audience is a human exploring plugins in the browser.
- Start with `host-service` if the audience is another program.

## Suggested learning path

1. Use `host-cli` to learn the raw plugin model.
2. Compare `host-tui`, `host-egui`, and `host-iced` to feel terminal vs desktop tradeoffs.
3. Open `host-dioxus-desktop` to compare reactive composition against the other desktop hosts.
4. Compare `host-web` and `host-service` to see the difference between human-facing and automation-facing surfaces.
