# Error Model

How errors are represented, propagated, and handled in the plugin system.

---

## Error Types

### 1. Discovery Errors

Occur when the host scans the plugin directory.

- **Plugin directory not found** — `Playground::load()` returns
  `Err(anyhow::Error)` if the directory does not exist.
- **Manifest parse failure** — if a shared library exports
  `plugin_manifest_json` but the JSON is malformed, the plugin is skipped and
  a warning is added to `playground.warnings()`.

### 2. Load Errors

Occur when `libloading` tries to open a shared library or locate symbols.

- **Dynamic library load failure** — the `.dylib`/`.so`/`.dll` cannot be
  opened (missing file, incompatible architecture, etc.).
- **Symbol not found** — the library does not export one of the three required
  symbols (`plugin_manifest_json`, `plugin_invoke_json`,
  `plugin_free_c_string`).
- **ABI mismatch** — an ABI-stable plugin was built with an incompatible
  version of `abi_stable`. Reported as a load error by `plugin-abi`.

### 3. Invocation Errors

Occur during a call to `Playground::invoke()` or `invoke_text()`.

- **Plugin not found** — `"no loaded plugin named '{plugin_id}'"` returned as
  `Err(anyhow::Error)`.
- **Action not found** — the plugin itself should return
  `Err(format!("unknown action '{action_id}'"))` from `JsonPlugin::invoke()`.
  The `export_plugin!` macro wraps this into a `PluginResponse::error()`.
- **Payload parse failure** — `request_from_json_ptr` fails to deserialise the
  JSON request; the macro returns a `PluginResponse::error()` with title
  "Failed to decode request".
- **Plugin panic** — unhandled panics inside plugin code will abort the host
  process. Plugins must not panic.

### 4. Response Errors

A plugin can signal a business-level error by returning a
`PluginResponse` with `success: false`.

- `success: false` — indicates the action did not succeed.
- `title` and `summary` — describe what went wrong in human-readable form.
- `outputs` — may contain diagnostic `OutputBlock`s (kind: `Text`, `Json`,
  `Code`, or `Markdown`).
- `suggested_next_steps` — recovery hints the host can present to the user.
- `warnings` — non-fatal issues encountered during execution.

## PluginResponse Error Format

```rust
PluginResponse::error(
    "my-plugin",
    "my-action",
    "Title describing the failure",
    "Detailed summary of what went wrong",
)
.with_output(OutputKind::Text, "Diagnostics", "extra detail here")
.with_next_step("Try re-running with a valid payload")
.with_warning("Config file was missing; used defaults")
```

Key fields on `PluginResponse`:

- `success: bool` — `false` for errors
- `title: String` — short error title
- `summary: String` — longer explanation
- `outputs: Vec<OutputBlock>` — diagnostic blocks (Text, Json, Code, Markdown)
- `suggested_next_steps: Vec<String>` — recovery hints
- `warnings: Vec<String>` — non-fatal issues
- `request_id: Option<String>` — correlation ID echoed from request

---

## Host-Side Error Handling

`Playground::invoke_text()` returns `Result<PluginResponse, anyhow::Error>`.

- **Transport errors** (plugin not found, load failure) surface as `Err(...)`.
  These mean the invocation never reached the plugin.
- **Business errors** (bad input, unsupported action) surface as
  `Ok(PluginResponse { success: false, .. })`. The plugin ran but could not
  fulfil the request.

Hosts should handle both:

```rust
match playground.invoke_text(plugin_id, action_id, payload, host) {
    Err(e) => eprintln!("transport error: {e}"),
    Ok(response) if !response.success => {
        eprintln!("plugin error: {}", response.summary);
        for step in &response.suggested_next_steps {
            eprintln!("  hint: {step}");
        }
    }
    Ok(response) => println!("{}", host_core::render_response(&response)),
}
```

---

## Common Failure Modes

| Symptom | Likely cause | Fix |
|---|---|---|
| Plugin missing from `list` | Not built | `cargo build -p plugin-name` |
| "no loaded plugin named …" | Binary not in plugin dir | Check `--plugin-dir` or `RUST_PLUGIN_SYSTEM_PLUGIN_DIR` |
| ABI version error | ABI-stable plugin built with different `abi_stable` version | Rebuild both host and plugin together |
| WASM module not found | `wasm-plugin.json` path is wrong | Verify the manifest path in the workspace root |
| Payload parse failure | JSON shape does not match expectations | Check `payload_hint` in the plugin manifest |
| "unknown action" error | Typo or unsupported action ID | Run `host-cli inspect <plugin-id>` to list valid actions |

---

## Debugging Strategies

1. **Inspect the manifest** — `cargo run -p host-cli -- inspect <plugin-id>`
   shows actions and payload hints.
2. **Check discovery warnings** — `playground.warnings()` returns issues found
   during plugin directory scanning.
3. **Enable verbose logging** — run with `RUST_LOG=debug` for detailed output.
4. **Unit-test with `plugin-test-kit`** — build `PluginRequest` and
   `InvocationContext` to test `invoke()` without loading a shared library.
5. **Check response metadata** — `ExecutionMetadata` and `NegotiationOutcome`
   on `PluginResponse` help diagnose runtime issues.
