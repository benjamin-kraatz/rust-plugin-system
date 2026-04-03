use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use plugin_loader::{LoadedPlugin, PluginCatalog, load_plugins_from_directory};
use plugin_manifest::PluginManifest;
use plugin_protocol::{HostKind, InvocationContext, PluginRequest, PluginResponse};
use plugin_runtime::PluginSummary;
use serde_json::Value;

pub struct Playground {
    plugins: Vec<LoadedPlugin>,
    warnings: Vec<String>,
    plugin_dir: PathBuf,
}

impl Playground {
    pub fn load_default() -> Result<Self> {
        Self::load(default_plugin_dir())
    }

    pub fn load(plugin_dir: impl AsRef<Path>) -> Result<Self> {
        let plugin_dir = plugin_dir.as_ref().to_path_buf();
        let PluginCatalog { plugins, warnings } = load_plugins_from_directory(&plugin_dir)?;

        Ok(Self {
            plugins,
            warnings,
            plugin_dir,
        })
    }

    pub fn plugin_dir(&self) -> &Path {
        &self.plugin_dir
    }

    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    pub fn manifests(&self) -> Vec<PluginManifest> {
        self.plugins
            .iter()
            .map(|plugin| plugin.manifest().clone())
            .collect()
    }

    pub fn summaries(&self) -> Vec<PluginSummary> {
        self.plugins
            .iter()
            .map(|plugin| PluginSummary::from(plugin.manifest()))
            .collect()
    }

    pub fn invoke_text(
        &self,
        plugin_id: &str,
        action_id: &str,
        payload_text: &str,
        host: HostKind,
    ) -> Result<PluginResponse> {
        let payload = parse_payload(payload_text);
        self.invoke(plugin_id, action_id, payload, host)
    }

    pub fn invoke(
        &self,
        plugin_id: &str,
        action_id: &str,
        payload: Value,
        host: HostKind,
    ) -> Result<PluginResponse> {
        let plugin = self
            .plugins
            .iter()
            .find(|plugin| plugin.manifest().id == plugin_id)
            .ok_or_else(|| anyhow!("no loaded plugin named '{plugin_id}'"))?;

        let request = PluginRequest {
            plugin_id: plugin_id.to_owned(),
            action_id: action_id.to_owned(),
            payload,
            context: InvocationContext {
                host,
                workspace_root: std::env::current_dir()
                    .ok()
                    .and_then(|path| path.to_str().map(str::to_owned)),
                plugin_dir: self.plugin_dir.to_str().map(str::to_owned),
                mode: Some("interactive".to_owned()),
            },
        };

        plugin.invoke(&request)
    }
}

pub fn default_plugin_dir() -> PathBuf {
    std::env::var_os("RUST_PLUGIN_SYSTEM_PLUGIN_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/debug"))
}

fn parse_payload(payload_text: &str) -> Value {
    let trimmed = payload_text.trim();
    if trimmed.is_empty() {
        Value::Null
    } else {
        serde_json::from_str(trimmed).unwrap_or_else(|_| Value::String(payload_text.to_owned()))
    }
}
