use std::ffi::{CString, c_char};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use libloading::Library;
use plugin_api::{FREE_SYMBOL, INVOKE_SYMBOL, MANIFEST_SYMBOL};
use plugin_manifest::PluginManifest;
use plugin_protocol::PluginRequest;
use plugin_runtime::render_response;

type ManifestFn = unsafe extern "C" fn() -> *mut c_char;
type InvokeFn = unsafe extern "C" fn(*const c_char) -> *mut c_char;
type FreeFn = unsafe extern "C" fn(*mut c_char);

pub struct LoadedPlugin {
    _library: Library,
    manifest: PluginManifest,
    invoke: InvokeFn,
    free: FreeFn,
    path: PathBuf,
}

impl LoadedPlugin {
    pub fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn invoke(&self, request: &PluginRequest) -> Result<plugin_protocol::PluginResponse> {
        let request_json = serde_json::to_string(request)?;
        let request_c_string = CString::new(request_json)
            .map_err(|error| anyhow!("request JSON contained NUL byte: {error}"))?;

        let response_ptr = unsafe { (self.invoke)(request_c_string.as_ptr()) };
        if response_ptr.is_null() {
            return Err(anyhow!(
                "plugin '{}' returned a null response pointer",
                self.manifest.id
            ));
        }

        let response_json = unsafe { plugin_api::copy_c_string(response_ptr.cast_const()) }
            .map_err(|error| anyhow!(error))?;
        unsafe { (self.free)(response_ptr) };

        let response = serde_json::from_str(&response_json).with_context(|| {
            format!(
                "failed to parse plugin response from '{}':\n{}",
                self.manifest.id, response_json
            )
        })?;

        Ok(response)
    }

    pub fn invoke_and_render(&self, request: &PluginRequest) -> Result<String> {
        self.invoke(request)
            .map(|response| render_response(&response))
    }

    unsafe fn load(path: &Path) -> Result<Self> {
        let library = unsafe { Library::new(path) }
            .with_context(|| format!("failed to open dynamic library '{}'", path.display()))?;
        let manifest_symbol = unsafe { library.get::<ManifestFn>(MANIFEST_SYMBOL) };
        let invoke_symbol = unsafe { library.get::<InvokeFn>(INVOKE_SYMBOL) };
        let free_symbol = unsafe { library.get::<FreeFn>(FREE_SYMBOL) };

        let (manifest_fn, invoke, free) = match (manifest_symbol, invoke_symbol, free_symbol) {
            (Ok(manifest_fn), Ok(invoke), Ok(free)) => (*manifest_fn, *invoke, *free),
            _ => {
                return Err(anyhow!(
                    "dynamic library '{}' is not a playground plugin",
                    path.display()
                ));
            }
        };

        let manifest_ptr = unsafe { manifest_fn() };
        if manifest_ptr.is_null() {
            return Err(anyhow!(
                "plugin at '{}' returned a null manifest pointer",
                path.display()
            ));
        }

        let manifest_json = unsafe { plugin_api::copy_c_string(manifest_ptr.cast_const()) }
            .map_err(|error| anyhow!(error))?;
        unsafe { free(manifest_ptr) };

        let manifest =
            serde_json::from_str::<PluginManifest>(&manifest_json).with_context(|| {
                format!(
                    "failed to deserialize plugin manifest from '{}':\n{}",
                    path.display(),
                    manifest_json
                )
            })?;

        Ok(Self {
            _library: library,
            manifest,
            invoke,
            free,
            path: path.to_path_buf(),
        })
    }
}

pub struct PluginCatalog {
    pub plugins: Vec<LoadedPlugin>,
    pub warnings: Vec<String>,
}

pub fn load_plugins_from_directory(directory: &Path) -> Result<PluginCatalog> {
    let mut candidates = fs::read_dir(directory)
        .with_context(|| format!("failed to read plugin directory '{}'", directory.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| is_dynamic_library(path))
        .collect::<Vec<_>>();
    candidates.sort();

    let mut plugins = Vec::new();
    let mut warnings = Vec::new();

    for candidate in candidates {
        match unsafe { LoadedPlugin::load(&candidate) } {
            Ok(plugin) => plugins.push(plugin),
            Err(error) => warnings.push(format!("{}: {error}", candidate.display())),
        }
    }

    Ok(PluginCatalog { plugins, warnings })
}

fn is_dynamic_library(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };

    let extension_matches = if cfg!(target_os = "macos") {
        path.extension().and_then(|value| value.to_str()) == Some("dylib")
    } else if cfg!(target_os = "windows") {
        path.extension().and_then(|value| value.to_str()) == Some("dll")
    } else {
        path.extension().and_then(|value| value.to_str()) == Some("so")
    };

    extension_matches && file_name.starts_with("lib")
}
