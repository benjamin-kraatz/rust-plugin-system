# First Runtime Plugin

The easiest plugin to study first is `plugins/hello-world`.

## What to look at

1. The manifest definition
2. The supported host list
3. The action list
4. The `invoke` implementation
5. The `export_plugin!` macro use

## What it teaches

- how runtime plugin metadata is exposed
- how the request/response JSON model works
- how hosts can stay generic while plugins stay useful

## Suggested walkthrough

1. Run `cargo run -p host-cli -- inspect hello-world`
2. Run `cargo run -p host-cli -- run hello-world greet '{"name":"Rustacean"}'`
3. Open `plugins/hello-world/src/lib.rs`
4. Compare the output from `host-cli` and a GUI host

