use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostKind {
    Cli,
    Tui,
    Egui,
    Iced,
    Dioxus,
    Web,
    Service,
    Any,
}

impl HostKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Cli => "CLI",
            Self::Tui => "TUI",
            Self::Egui => "egui/eframe",
            Self::Iced => "Iced",
            Self::Dioxus => "Dioxus Desktop",
            Self::Web => "Web",
            Self::Service => "Service",
            Self::Any => "Any host",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginArchitecture {
    NativeJson,
    AbiStable,
    Wasm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SkillLevel {
    Basic,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    pub key: String,
    pub description: String,
}

impl Capability {
    pub fn new(key: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
        }
    }
}
