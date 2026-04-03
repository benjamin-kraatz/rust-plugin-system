//! Scaffolding for the ABI-stable native plugin track.
//!
//! The working repository slice currently uses the native JSON-over-FFI path.
//! This crate reserves the shared types and terminology for the next layer:
//! version-tolerant native plugin contracts.

use plugin_manifest::PluginManifest;
use plugin_protocol::PluginResponse;

#[derive(Debug, Clone)]
pub struct AbiPluginDescriptor {
    pub manifest: PluginManifest,
    pub compatibility_notes: Vec<String>,
}

impl AbiPluginDescriptor {
    pub fn new(manifest: PluginManifest) -> Self {
        Self {
            manifest,
            compatibility_notes: Vec::new(),
        }
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.compatibility_notes.push(note.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct AbiInvocationResult {
    pub response: PluginResponse,
    pub contract_version: String,
}

impl AbiInvocationResult {
    pub fn new(response: PluginResponse, contract_version: impl Into<String>) -> Self {
        Self {
            response,
            contract_version: contract_version.into(),
        }
    }
}
