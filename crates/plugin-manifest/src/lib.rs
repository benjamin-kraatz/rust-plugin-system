pub use plugin_capabilities::{Capability, HostKind, PluginArchitecture, SkillLevel};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginAction {
    pub id: String,
    pub label: String,
    pub description: String,
    pub payload_hint: Option<String>,
}

impl PluginAction {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: description.into(),
            payload_hint: None,
        }
    }

    pub fn with_payload_hint(mut self, payload_hint: impl Into<String>) -> Self {
        self.payload_hint = Some(payload_hint.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub architecture: PluginArchitecture,
    pub skill_level: SkillLevel,
    pub supported_hosts: Vec<HostKind>,
    pub capabilities: Vec<Capability>,
    pub tags: Vec<String>,
    pub actions: Vec<PluginAction>,
    pub notes: Vec<String>,
}

impl PluginManifest {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        architecture: PluginArchitecture,
        skill_level: SkillLevel,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
            description: description.into(),
            architecture,
            skill_level,
            supported_hosts: vec![HostKind::Any],
            capabilities: Vec::new(),
            tags: Vec::new(),
            actions: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub fn with_supported_hosts(mut self, supported_hosts: Vec<HostKind>) -> Self {
        self.supported_hosts = supported_hosts;
        self
    }

    pub fn with_capabilities(mut self, capabilities: Vec<Capability>) -> Self {
        self.capabilities = capabilities;
        self
    }

    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags = tags.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_actions(mut self, actions: Vec<PluginAction>) -> Self {
        self.actions = actions;
        self
    }

    pub fn with_notes(mut self, notes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.notes = notes.into_iter().map(Into::into).collect();
        self
    }
}
