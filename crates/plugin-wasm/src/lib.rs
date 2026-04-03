//! Wasm plugin loader backed by `wasmtime`.
//!
//! A Wasm plugin directory is expected to contain:
//!
//! - `wasm-plugin.json` with a serialized `PluginManifest`
//! - `module.wasm` or `module.wat` with exported `memory`, `alloc`, and
//!   `invoke_json`

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use plugin_manifest::PluginManifest;
use plugin_protocol::{PluginRequest, PluginResponse};
use wasmtime::{Engine, Instance, Module, Store};

/// A loaded Wasm plugin descriptor.
#[derive(Debug, Clone)]
pub struct LoadedWasmPlugin {
    manifest: PluginManifest,
    module_path: PathBuf,
}

impl LoadedWasmPlugin {
    /// Returns the parsed plugin manifest.
    pub fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    /// Returns the path to the Wasm or WAT module file.
    pub fn module_path(&self) -> &Path {
        &self.module_path
    }

    /// Instantiates the module and invokes `invoke_json` with a request payload.
    ///
    /// # Errors
    ///
    /// Returns an error when module exports are missing, memory operations fail,
    /// or response JSON is invalid.
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

/// Result of scanning and loading Wasm plugin directories.
pub struct WasmPluginCatalog {
    /// Successfully loaded Wasm plugins.
    pub plugins: Vec<LoadedWasmPlugin>,
    /// Non-fatal loading failures keyed by plugin directory.
    pub warnings: Vec<String>,
}

/// Loads all Wasm plugins from `root/plugins`.
///
/// Subdirectories without a `wasm-plugin.json` file are ignored.
///
/// # Errors
///
/// Returns an error if the workspace plugin directory cannot be read.
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

/// Loads one Wasm plugin from a directory.
///
/// The directory must include `wasm-plugin.json` and either `module.wasm` or
/// `module.wat`.
///
/// # Errors
///
/// Returns an error when manifest or module files are missing, unreadable, or
/// malformed.
///
/// # Examples
///
/// ```no_run
/// use plugin_wasm::load_plugin_from_dir;
/// use std::path::Path;
///
/// let plugin = load_plugin_from_dir(Path::new("plugins/my-wasm-plugin"))?;
/// println!("loaded {}", plugin.manifest().id);
/// # Ok::<(), anyhow::Error>(())
/// ```
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
