//! Hot-reload example.
//!
//! Watches a plugin library on disk and reloads it whenever the file changes.
//! This lets you rebuild a plugin in one terminal and see the new behaviour
//! immediately in another.
//!
//! Caveats:
//! - `libloading::Library::close` does *not* guarantee the OS will actually
//!   unload the shared object (especially on macOS with `dlopen`).
//! - Reloading while the old code is still executing is undefined behaviour.
//! - In production you would double-buffer: load the new library, swap
//!   pointers atomically, *then* drop the old handle after a grace period.

use std::env;
use std::ffi::{CStr, CString, c_char};
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use notify::{EventKind, RecursiveMode, Watcher};

/// Load a plugin, print its manifest, and invoke its first action.
fn load_and_invoke(path: &str) -> Result<()> {
    // SAFETY: We trust the plugin to expose the expected C ABI.
    unsafe {
        let lib = libloading::Library::new(path)
            .with_context(|| format!("Failed to load library: {path}"))?;

        // --- manifest ---
        let manifest_fn: libloading::Symbol<unsafe extern "C" fn() -> *mut c_char> = lib
            .get(b"plugin_manifest_json")
            .context("symbol `plugin_manifest_json` not found")?;

        let free_fn: libloading::Symbol<unsafe extern "C" fn(*mut c_char)> = lib
            .get(b"plugin_free_c_string")
            .context("symbol `plugin_free_c_string` not found")?;

        let manifest_ptr = manifest_fn();
        if manifest_ptr.is_null() {
            bail!("plugin_manifest_json returned null");
        }
        let manifest: serde_json::Value =
            serde_json::from_str(CStr::from_ptr(manifest_ptr).to_str()?)?;
        free_fn(manifest_ptr);

        let name = manifest["name"].as_str().unwrap_or("?");
        let version = manifest["version"].as_str().unwrap_or("?");
        println!("  plugin : {name} v{version}");

        // --- invoke first action ---
        if let Some(action_id) = manifest["actions"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|a| a["id"].as_str())
        {
            let plugin_id = manifest["id"].as_str().unwrap_or("?");
            let request = serde_json::json!({
                "plugin_id": plugin_id,
                "action_id": action_id,
                "payload": {},
                "context": { "host": "Cli" }
            });
            let req_cstr = CString::new(request.to_string())?;

            let invoke_fn: libloading::Symbol<unsafe extern "C" fn(*const c_char) -> *mut c_char> =
                lib.get(b"plugin_invoke_json")?;

            let resp_ptr = invoke_fn(req_cstr.as_ptr());
            if !resp_ptr.is_null() {
                let resp: serde_json::Value =
                    serde_json::from_str(CStr::from_ptr(resp_ptr).to_str()?)?;
                free_fn(resp_ptr as *mut c_char);

                let ok = resp["success"].as_bool().unwrap_or(false);
                let summary = resp["summary"].as_str().unwrap_or("");
                println!("  action : {action_id} → success={ok}  {summary}");
            }
        }

        // `lib` is dropped here which *requests* the OS to unload the library.
    }

    Ok(())
}

fn main() -> Result<()> {
    let path = env::args()
        .nth(1)
        .context("Usage: hot-reload <path/to/plugin.dylib|.so>")?;

    println!("=== Hot-reload watcher ===");
    println!("Watching: {path}");
    println!("Press Ctrl+C to stop.\n");

    // Initial load.
    println!("[initial load]");
    load_and_invoke(&path)?;
    println!();

    // Set up file watcher.
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            // We care about writes / creates (a rebuild replaces the file).
            if matches!(
                event.kind,
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
            ) {
                let _ = tx.send(());
            }
        }
    })?;

    // Watch the parent directory (some OSes replace the file atomically via rename).
    let watch_dir = Path::new(&path)
        .parent()
        .context("cannot determine parent directory")?;
    watcher.watch(watch_dir, RecursiveMode::NonRecursive)?;

    // Event loop – deduplicate rapid-fire events with a small delay.
    loop {
        rx.recv()?; // block until a change arrives

        // Drain any additional events that arrived in quick succession.
        thread::sleep(Duration::from_millis(200));
        while rx.try_recv().is_ok() {}

        println!("[reload]");
        match load_and_invoke(&path) {
            Ok(()) => {}
            Err(e) => eprintln!("  error: {e:#}"),
        }
        println!();
    }
}
