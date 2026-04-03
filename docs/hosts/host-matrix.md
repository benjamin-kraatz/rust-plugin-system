# Host Matrix

This repo intentionally compares the same plugin ideas across very different host surfaces.

| Host | Best for | UX shape | What stands out now |
| --- | --- | --- | --- |
| `host-cli` | Fast inspection, scripting, smoke tests | Command-driven | Baseline host for raw plugin discovery and invocation |
| `host-tui` | Staying in the terminal | Keyboard-first multi-pane UI | Separate plugin list, action list, payload editor, and output view |
| `host-egui` | Inspecting metadata visually | Dashboard / inspector | Manifest metadata, host-fit cues, template loading, and output panel |
| `host-iced` | Studying explicit application state | State/update desktop app | More deliberate selection and invocation flow than the dashboard-style hosts |
| `host-dioxus-desktop` | Trying a reactive desktop surface | Component-style desktop UI | Reactive plugin/action selection with payload editing and local output framing |
| `host-web` | Human-guided browser exploration | Catalog + composer + result cards | Manifest badges, action browser, payload composer, and formatted response cards |
| `host-service` | Automation and integration | JSON API | Catalog, examples, per-plugin/action detail, and canonical invoke endpoint |

## What the Phase 3 hosts have in common

The richer hosts now share a common baseline instead of exposing only a bare invoke button:

- plugin selection
- action selection
- payload hints/templates loaded into editors or composers
- manifest metadata beyond name/description
- explicit output/result framing
- host support checks powered by shared host-core helpers

## How the surfaces differ

### `host-tui`

Use this when you want the richest comparison surface without leaving the terminal. It is the most keyboard-centric host in the repo and makes pane-to-pane movement part of the learning experience.

### `host-egui`

Use this when you want a quick inspector. It is the most dashboard-like host: easy to scan, easy to click through, and good for browsing manifest metadata and payload hints.

### `host-iced`

Use this when you want the host implementation model itself to be part of the lesson. The explicit state/update structure is a good contrast with the more immediate style of `egui` and the reactive style of Dioxus.

### `host-dioxus-desktop`

Use this when you want a component-oriented desktop surface. It lands between the inspector feel of `egui` and the browser-like composition patterns that Dioxus users may expect.

### `host-web`

Use this when a human should browse plugins in the browser, load example payloads, and compare richer response presentation without needing an API client.

### `host-service`

Use this when another tool or service needs stable JSON responses. It mirrors many of the same comparison concepts as `host-web`, but through machine-friendly endpoints instead of panels.

## Recommended follow-up

For the deeper walkthrough, continue with `docs/hosts/rich-host-surfaces.md`.
