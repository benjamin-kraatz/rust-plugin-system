# Host Matrix

This repo intentionally compares the same plugin ideas across very different host surfaces.

| Host | Purpose | Current role |
| --- | --- | --- |
| `host-cli` | Fastest way to inspect and invoke plugins | Primary baseline host |
| `host-tui` | Terminal-native plugin browsing | Keyboard-first comparison host |
| `host-egui` | Rapid Rust-native GUI panels and actions | Visual host |
| `host-iced` | Explicit state/update GUI architecture | Visual comparison host |
| `host-dioxus-desktop` | Component-oriented reactive desktop UI | Visual comparison host |
| `host-web` | Browser-facing plugin catalog and invocation | Web-facing host |
| `host-service` | Backend/API plugin workflows | Service/backend host |

## Why so many hosts?

Because plugin systems are not just a loader problem. They are also a **host integration problem**:

- how users discover plugins
- how actions are surfaced
- how results are rendered
- how runtime boundaries differ across environments

This repo is meant to make those differences visible.

