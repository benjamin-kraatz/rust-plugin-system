//! WebAssembly plugin sandbox for the **Rust Plugin System**.
//!
//! This crate loads and executes plugins compiled to WebAssembly (`.wasm`) or
//! WebAssembly Text format (`.wat`) using [Wasmtime](https://wasmtime.dev/).
//! Each invocation runs inside a fresh Wasmtime [`Store`](wasmtime::Store),
//! providing strong memory isolation between the host and the plugin.
//!
//! # Plugin layout
//!
//! A WASM plugin lives in a directory that contains:
//!
//! * `wasm-plugin.json` — the plugin's [`PluginManifest`](plugin_manifest::PluginManifest)
//!   serialised as JSON.
//! * A `.wasm` or `.wat` module file exporting `alloc`, `invoke_json`, and
//!   `free` symbols.
//!
//! # How it works
//!
//! 1. [`load_plugins_from_workspace`] scans subdirectories for `wasm-plugin.json`
//!    files.
//! 2. Each manifest is paired with its compiled module; the module is compiled
//!    once and cached in a [`LoadedWasmPlugin`].
//! 3. On invocation a fresh Wasmtime `Store` + `Instance` is created, memory is
//!    linearised, and the JSON-encoded request is written into WASM linear memory
//!    before calling the `invoke_json` export.
//!
//! # Example
//!
//! ```rust,no_run
//! use plugin_wasm::load_plugins_from_workspace;
//!
//! let catalog = load_plugins_from_workspace("plugins").unwrap();
//! for plugin in &catalog.plugins {
//!     println!("{} — {}", plugin.manifest().id, plugin.manifest().name);
//! }
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use plugin_manifest::PluginManifest;
use plugin_protocol::{PluginRequest, PluginResponse};
use wasmtime::{Engine, Instance, Module, Store};

#[derive(Debug, Clone)]
pub struct LoadedWasmPlugin {
    manifest: PluginManifest,
    module_path: PathBuf,
}

impl LoadedWasmPlugin {
    pub fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    pub fn module_path(&self) -> &Path {
        &self.module_path
    }

    pub fn invoke(&self, request: &PluginRequest) -> Result<PluginResponse> {
        let engine = Engine::default();
        let module = load_module(&engine, &self.module_path)?;
        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[])?;
        let memory = instance.get_memory(&mut store, "memory").ok_or_else(|| {
            anyhow!(
                "module '{}' does not export memory",
                self.module_path.display()
            )
        })?;
        let alloc = instance
            .get_typed_func::<i32, i32>(&mut store, "alloc")
            .map_err(|error| anyhow!("module is missing alloc(len) -> ptr: {error}"))?;
        let invoke = instance
            .get_typed_func::<(i32, i32), (i32, i32)>(&mut store, "invoke_json")
            .map_err(|error| {
                anyhow!("module is missing invoke_json(ptr, len) -> (ptr, len): {error}")
            })?;

        let request_json = serde_json::to_string(request)?;
        let request_bytes = request_json.as_bytes();
        let request_len = i32::try_from(request_bytes.len())
            .map_err(|_| anyhow!("request too large for wasm demo"))?;
        let request_ptr = alloc.call(&mut store, request_len)?;
        memory.write(&mut store, request_ptr as usize, request_bytes)?;

        let (response_ptr, response_len) = invoke.call(&mut store, (request_ptr, request_len))?;
        let mut response_bytes = vec![0; response_len as usize];
        memory.read(&mut store, response_ptr as usize, &mut response_bytes)?;
        let response_json =
            String::from_utf8(response_bytes).context("wasm plugin returned invalid UTF-8")?;

        serde_json::from_str(&response_json).with_context(|| {
            format!(
                "failed to parse wasm plugin response from '{}'",
                self.manifest.id
            )
        })
    }
}

pub struct WasmPluginCatalog {
    pub plugins: Vec<LoadedWasmPlugin>,
    pub warnings: Vec<String>,
}

pub fn load_plugins_from_workspace(root: &Path) -> Result<WasmPluginCatalog> {
    let plugins_dir = root.join("plugins");
    let mut plugins = Vec::new();
    let mut warnings = Vec::new();

    for entry in fs::read_dir(&plugins_dir).with_context(|| {
        format!(
            "failed to read wasm plugin workspace '{}'",
            plugins_dir.display()
        )
    })? {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join("wasm-plugin.json");
        if !manifest_path.exists() {
            continue;
        }

        match load_plugin_from_dir(&path) {
            Ok(plugin) => plugins.push(plugin),
            Err(error) => warnings.push(format!("{}: {error}", path.display())),
        }
    }

    Ok(WasmPluginCatalog { plugins, warnings })
}

pub fn load_plugin_from_dir(directory: &Path) -> Result<LoadedWasmPlugin> {
    let manifest_path = directory.join("wasm-plugin.json");
    let manifest_json = fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read '{}'", manifest_path.display()))?;
    let manifest = serde_json::from_str::<PluginManifest>(&manifest_json)
        .with_context(|| format!("failed to parse '{}'", manifest_path.display()))?;

    let module_path = if directory.join("module.wasm").exists() {
        directory.join("module.wasm")
    } else {
        directory.join("module.wat")
    };

    if !module_path.exists() {
        return Err(anyhow!(
            "expected module.wat or module.wasm next to '{}'",
            manifest_path.display()
        ));
    }

    Ok(LoadedWasmPlugin {
        manifest,
        module_path,
    })
}

fn load_module(engine: &Engine, module_path: &Path) -> Result<Module> {
    match module_path.extension().and_then(|value| value.to_str()) {
        Some("wat") => {
            let source = fs::read_to_string(module_path)
                .with_context(|| format!("failed to read '{}'", module_path.display()))?;
            let bytes = wat::parse_str(&source)
                .with_context(|| format!("failed to compile WAT '{}'", module_path.display()))?;
            Module::new(engine, bytes).map_err(|error| {
                anyhow!(
                    "failed to compile wasm module '{}': {error}",
                    module_path.display()
                )
            })
        }
        _ => Module::from_file(engine, module_path).map_err(|error| {
            anyhow!(
                "failed to load wasm module '{}': {error}",
                module_path.display()
            )
        }),
    }
}
