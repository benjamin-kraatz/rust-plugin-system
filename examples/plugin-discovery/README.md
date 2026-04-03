# Plugin Discovery Example

Scans the default plugin directory, groups discovered plugins by
architecture and skill level, and prints a summary table.

## What it demonstrates

- Using `Playground::load_default()` to auto-discover plugins.
- Grouping plugins by `PluginArchitecture` (Native, ABI-Stable, WASM).
- Grouping plugins by `SkillLevel`.
- Checking host support per plugin.
- Filtering: finding Advanced+ plugins that support a given host.

## How to run

```bash
# Build several plugins first
cargo build -p hello-world -p logger -p formatter -p command-pack

# Run the discovery scanner
cargo run -p example-plugin-discovery
```
