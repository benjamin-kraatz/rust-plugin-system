# Capability Negotiation Example

Shows the host/plugin capability-matching flow for every discovered plugin
across multiple host kinds (CLI, Egui, Web, Service).

## What it demonstrates

- Loading all plugins via `Playground::load_default()`.
- Inspecting each plugin's declared capabilities and constraints.
- Running `assess_host_fit()` to see whether a plugin is Ready, Degraded,
  or Rejected for a given host.
- Viewing negotiation details: missing required capabilities, degradation
  rules, and fallback behaviour.

## How to run

```bash
# Build some plugins first
cargo build -p hello-world -p logger -p formatter -p command-pack

# Run the example
cargo run -p example-capability-negotiation
```
