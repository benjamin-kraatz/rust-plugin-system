# CLI Recipes

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
