pub use plugin_capabilities::HostKind;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationContext {
    pub host: HostKind,
    pub workspace_root: Option<String>,
    pub plugin_dir: Option<String>,
    pub mode: Option<String>,
}

impl InvocationContext {
    pub fn for_host(host: HostKind) -> Self {
        Self {
            host,
            workspace_root: None,
            plugin_dir: None,
            mode: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRequest {
    pub plugin_id: String,
    pub action_id: String,
    pub payload: Value,
    pub context: InvocationContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputKind {
    Text,
    Json,
    Code,
    Markdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputBlock {
    pub kind: OutputKind,
    pub title: Option<String>,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponse {
    pub plugin_id: String,
    pub action_id: String,
    pub title: String,
    pub summary: String,
    pub success: bool,
    pub outputs: Vec<OutputBlock>,
    pub suggested_next_steps: Vec<String>,
}

impl PluginResponse {
    pub fn ok(
        plugin_id: impl Into<String>,
        action_id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            action_id: action_id.into(),
            title: title.into(),
            summary: summary.into(),
            success: true,
            outputs: Vec::new(),
            suggested_next_steps: Vec::new(),
        }
    }

    pub fn error(
        plugin_id: impl Into<String>,
        action_id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            action_id: action_id.into(),
            title: title.into(),
            summary: summary.into(),
            success: false,
            outputs: Vec::new(),
            suggested_next_steps: Vec::new(),
        }
    }

    pub fn with_output(
        mut self,
        kind: OutputKind,
        title: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        self.outputs.push(OutputBlock {
            kind,
            title: Some(title.into()),
            body: body.into(),
        });
        self
    }

    pub fn with_next_step(mut self, next_step: impl Into<String>) -> Self {
        self.suggested_next_steps.push(next_step.into());
        self
    }
}
