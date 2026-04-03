//! ABI-stable plugin loader for the **Rust Plugin System**.
//!
//! This crate loads plugins built with [`abi_stable`](https://docs.rs/abi_stable)
//! rather than the plain C FFI used by `plugin-loader`.  Because `abi_stable`
//! enforces layout compatibility across Rust compiler updates you can ship new
//! plugin versions without recompiling the host, making it ideal for
//! long-lived applications.
//!
//! # How it works
//!
//! 1. [`load_plugins_from_directory`] scans a directory for shared libraries
//!    whose filename contains `"abi_stable"`.
//! 2. Each library is loaded via `abi_stable`'s root-module infrastructure which
//!    validates the ABI layout at load time.
//! 3. Successfully loaded plugins are wrapped in a [`LoadedAbiPlugin`] that
//!    exposes typed `invoke` methods through the stable vtable.
//!
//! # Example
//!
//! ```rust,no_run
//! use plugin_abi::load_plugins_from_directory;
//!
//! let catalog = load_plugins_from_directory("target/debug").unwrap();
//! for plugin in &catalog.plugins {
//!     println!("{} — {}", plugin.manifest().id, plugin.manifest().name);
//! }
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use abi_stable::{
    StableAbi, declare_root_module_statics,
    library::{RootModule, lib_header_from_path},
    sabi_types::VersionStrings,
    std_types::RString,
};
use anyhow::{Context, Result};
use plugin_manifest::PluginManifest;
use plugin_protocol::{PluginRequest, PluginResponse};

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = AbiPluginModuleRef)))]
#[sabi(missing_field(panic))]
pub struct AbiPluginModule {
    #[sabi(last_prefix_field)]
    pub manifest_json: extern "C" fn() -> RString,
    pub invoke_json: extern "C" fn(RString) -> RString,
}

impl RootModule for AbiPluginModuleRef {
    declare_root_module_statics! {AbiPluginModuleRef}
    const BASE_NAME: &'static str = "abi_plugin_module";
    const NAME: &'static str = "ABI Plugin Module";
    const VERSION_STRINGS: VersionStrings = abi_stable::package_version_strings!();
}

#[derive(Clone)]
pub struct LoadedAbiPlugin {
    manifest: PluginManifest,
    path: PathBuf,
    module: AbiPluginModuleRef,
}

impl LoadedAbiPlugin {
    pub fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn invoke(&self, request: &PluginRequest) -> Result<PluginResponse> {
        let request_json = serde_json::to_string(request)?;
        let response_json = (self.module.invoke_json())(RString::from(request_json));
        serde_json::from_str(response_json.as_str()).with_context(|| {
            format!(
                "failed to parse ABI plugin response from '{}'",
                self.manifest.id
            )
        })
    }
}

pub struct AbiPluginCatalog {
    pub plugins: Vec<LoadedAbiPlugin>,
    pub warnings: Vec<String>,
}

pub fn load_plugins_from_directory(directory: &Path) -> Result<AbiPluginCatalog> {
    let mut candidates = fs::read_dir(directory)
        .with_context(|| {
            format!(
                "failed to read ABI plugin directory '{}'",
                directory.display()
            )
        })?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| is_abi_plugin_candidate(path))
        .collect::<Vec<_>>();
    candidates.sort();

    let mut plugins = Vec::new();
    let mut warnings = Vec::new();

    for candidate in candidates {
        match load_plugin(&candidate) {
            Ok(plugin) => plugins.push(plugin),
            Err(error) => warnings.push(format!("{}: {error}", candidate.display())),
        }
    }

    Ok(AbiPluginCatalog { plugins, warnings })
}

pub fn load_plugin(path: &Path) -> Result<LoadedAbiPlugin> {
    let header = lib_header_from_path(path)
        .with_context(|| format!("failed to open ABI plugin '{}'", path.display()))?;
    let module = header
        .init_root_module::<AbiPluginModuleRef>()
        .with_context(|| format!("failed to initialize ABI plugin '{}'", path.display()))?;

    let manifest_json = (module.manifest_json())();
    let manifest =
        serde_json::from_str::<PluginManifest>(manifest_json.as_str()).with_context(|| {
            format!(
                "failed to deserialize ABI plugin manifest from '{}'",
                path.display()
            )
        })?;

    Ok(LoadedAbiPlugin {
        manifest,
        path: path.to_path_buf(),
        module,
    })
}

fn is_abi_plugin_candidate(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };

    if !file_name.contains("abi_stable") {
        return false;
    }

    is_dynamic_library(path)
}

fn is_dynamic_library(path: &Path) -> bool {
    let extension = path.extension().and_then(|value| value.to_str());

    if cfg!(target_os = "macos") {
        extension == Some("dylib")
    } else if cfg!(target_os = "windows") {
        extension == Some("dll")
    } else {
        extension == Some("so")
    }
}
