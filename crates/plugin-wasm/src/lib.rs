//! Scaffolding for the WASM plugin track.
//!
//! This crate is the future home of the sandboxed runtime integration layer.
//! Keeping the crate in the workspace now makes the repository structure stable
//! while the working native JSON track grows.

use plugin_manifest::PluginManifest;
use plugin_protocol::PluginRequest;

#[derive(Debug, Clone)]
pub struct WasmPluginDescriptor {
    pub manifest: PluginManifest,
    pub module_path: String,
}

impl WasmPluginDescriptor {
    pub fn new(manifest: PluginManifest, module_path: impl Into<String>) -> Self {
        Self {
            manifest,
            module_path: module_path.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WasmInvocationEnvelope {
    pub request: PluginRequest,
    pub entrypoint: String,
}

impl WasmInvocationEnvelope {
    pub fn new(request: PluginRequest, entrypoint: impl Into<String>) -> Self {
        Self {
            request,
            entrypoint: entrypoint.into(),
        }
    }
}
