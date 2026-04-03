# CLI Recipes

## Build first

```bash
cargo build --workspace
```

## List runtime-loaded plugins

```bash
cargo run -p host-cli -- list
```

## Inspect a manifest

```bash
cargo run -p host-cli -- inspect transformer
```

## Pretty-print JSON through a plugin

```bash
cargo run -p host-cli -- run formatter pretty-json '{"hello":"world","n":42}'
```

## Transform text into a slug

```bash
cargo run -p host-cli -- run transformer slugify '{"text":"Rust Plugin Systems Course Module"}'
```

## Ask the command pack for useful commands

```bash
cargo run -p host-cli -- run command-pack suggest-commands '{}'
```

## Merge layered configuration

```bash
cargo run -p host-cli -- run config-provider merge-layers '{"defaults":{"service":{"port":8080}},"environment":{"service":{"port":8081}},"overrides":{"service":{"host":"127.0.0.1"}}}'
```

## Read one resolved config value

```bash
cargo run -p host-cli -- run config-provider get-value '{"config":{"service":{"host":"127.0.0.1","port":8081}},"path":"service.port"}'
```

## Flatten a config tree into dotted paths

```bash
cargo run -p host-cli -- run config-provider flatten-config '{"config":{"service":{"host":"127.0.0.1","port":8081}}}'
```

## List plugin directories safely from the workspace root

```bash
cargo run -p host-cli -- run filesystem-tools list-directory '{"path":"plugins","max_depth":1,"max_entries":8}'
```

## Preview a text file without editing it

```bash
cargo run -p host-cli -- run filesystem-tools read-text-file '{"path":"README.md","max_bytes":512}'
```

## Inspect one workspace path

```bash
cargo run -p host-cli -- run filesystem-tools describe-path '{"path":"plugins/config-provider"}'
```

## Project a record set down to stable fields

```bash
cargo run -p host-cli -- run data-pipeline project-records '{"records":[{"id":1,"name":"Ada","team":"platform"}],"fields":["id","name"]}'
```

## Filter a dataset before passing it downstream

```bash
cargo run -p host-cli -- run data-pipeline filter-records '{"records":[{"status":"active","duration":12.5},{"status":"draft","duration":7.5}],"predicate":{"field":"status","op":"eq","value":"active"}}'
```

## Summarize one numeric field across records

```bash
cargo run -p host-cli -- run data-pipeline summarize-field '{"records":[{"duration":12.5},{"duration":7.5}],"field":"duration"}'
```

## Produce a deterministic metrics rollup

```bash
cargo run -p host-cli -- run metrics-observer summarize-signals '{"service":"checkout","window_minutes":15,"samples":{"requests":1800,"errors":18,"p95_ms":245,"saturation_pct":61}}'
```

## Evaluate a simple SLO budget

```bash
cargo run -p host-cli -- run metrics-observer evaluate-slo '{"service":"checkout","objective_pct":99.5,"window_requests":1800,"window_errors":18}'
```

## Plan a service-hook rollout

```bash
cargo run -p host-cli -- run service-hooks plan-hook-set '{"service":"billing","environment":"staging","events":["deploy.succeeded","incident.opened"],"target_base_url":"https://hooks.internal.example"}'
```

## Preview one hook delivery attempt

```bash
cargo run -p host-cli -- run service-hooks preview-delivery '{"service":"billing","event":"deploy.succeeded","attempt":2,"target_base_url":"https://hooks.internal.example"}'
```

## Plan a terminal workspace layout

```bash
cargo run -p host-cli -- run tui-tools plan-layout '{"width":120,"height":36,"panes":["nav","editor","logs"],"focus":"editor","include_help":true}'
```

## Draft a TUI status line

```bash
cargo run -p host-cli -- run tui-tools draft-status-line '{"mode":"NORMAL","workspace":"rust-plugin-system","branch":"main","dirty":false,"pending_tasks":2,"focus":"editor"}'
```
