//! Minimal plugin loader example.
//!
//! Demonstrates the simplest possible host: load one native-JSON plugin
//! and call its FFI surface (`plugin_manifest_json`, `plugin_invoke_json`,
//! `plugin_free_c_string`).

use std::env;
use std::ffi::{CStr, CString, c_char};

use anyhow::{Context, Result, bail};

fn main() -> Result<()> {
    let path = env::args()
        .nth(1)
        .context("Usage: minimal-loader <path/to/plugin.dylib|.so>")?;

    println!("Loading plugin from: {path}\n");

    // SAFETY: We trust the plugin library to expose the expected C ABI.
    // In production you would sandbox or checksum the library first.
    unsafe {
        // Step 1 – Open the shared library.
        let lib = libloading::Library::new(&path)
            .with_context(|| format!("Failed to load library: {path}"))?;

        // Step 2 – Look up the manifest symbol.
        // `plugin_manifest_json` returns a `*mut c_char` (heap-allocated JSON).
        let manifest_fn: libloading::Symbol<unsafe extern "C" fn() -> *mut c_char> = lib
            .get(b"plugin_manifest_json")
            .context("Symbol `plugin_manifest_json` not found")?;

        let manifest_ptr = manifest_fn();
        if manifest_ptr.is_null() {
            bail!("plugin_manifest_json returned null");
        }

        let manifest_cstr = CStr::from_ptr(manifest_ptr);
        let manifest_json: serde_json::Value = serde_json::from_str(manifest_cstr.to_str()?)?;

        println!("=== Plugin Manifest ===");
        println!("{}\n", serde_json::to_string_pretty(&manifest_json)?);

        // Step 3 – Free the manifest string (plugin owns that memory).
        let free_fn: libloading::Symbol<unsafe extern "C" fn(*mut c_char)> = lib
            .get(b"plugin_free_c_string")
            .context("Symbol `plugin_free_c_string` not found")?;

        free_fn(manifest_ptr);

        // Step 4 – Invoke the first action with an empty payload.
        let actions = manifest_json["actions"]
            .as_array()
            .context("manifest has no actions array")?;
        let first_action = actions
            .first()
            .and_then(|a| a["id"].as_str())
            .context("no action id found")?;

        let plugin_id = manifest_json["id"].as_str().context("no plugin id found")?;

        // Build a minimal PluginRequest JSON.
        let request = serde_json::json!({
            "plugin_id": plugin_id,
            "action_id": first_action,
            "payload": {},
            "context": { "host": "Cli" }
        });

        let request_cstr = CString::new(request.to_string())?;

        let invoke_fn: libloading::Symbol<unsafe extern "C" fn(*const c_char) -> *mut c_char> = lib
            .get(b"plugin_invoke_json")
            .context("Symbol `plugin_invoke_json` not found")?;

        let response_ptr = invoke_fn(request_cstr.as_ptr());
        if response_ptr.is_null() {
            bail!("plugin_invoke_json returned null");
        }

        let response_cstr = CStr::from_ptr(response_ptr);
        let response_json: serde_json::Value = serde_json::from_str(response_cstr.to_str()?)?;

        println!("=== Invocation Response (action: {first_action}) ===");
        println!("{}", serde_json::to_string_pretty(&response_json)?);

        // Step 5 – Free the response string.
        free_fn(response_ptr as *mut c_char);
    }

    Ok(())
}
