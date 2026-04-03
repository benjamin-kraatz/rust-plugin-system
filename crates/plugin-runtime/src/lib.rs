use plugin_manifest::PluginManifest;
use plugin_protocol::{OutputBlock, PluginResponse};

#[derive(Debug, Clone)]
pub struct PluginSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub action_count: usize,
    pub supported_hosts: Vec<String>,
    pub tags: Vec<String>,
}

impl From<&PluginManifest> for PluginSummary {
    fn from(manifest: &PluginManifest) -> Self {
        Self {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            description: manifest.description.clone(),
            action_count: manifest.actions.len(),
            supported_hosts: manifest
                .supported_hosts
                .iter()
                .map(|host| host.label().to_owned())
                .collect(),
            tags: manifest.tags.clone(),
        }
    }
}

pub fn render_response(response: &PluginResponse) -> String {
    let mut rendered = String::new();
    rendered.push_str(&format!("{}\n{}\n", response.title, response.summary));

    for OutputBlock { kind, title, body } in &response.outputs {
        let label = title.as_deref().unwrap_or("Output");
        rendered.push_str(&format!("\n[{kind:?}] {label}\n{body}\n"));
    }

    if !response.suggested_next_steps.is_empty() {
        rendered.push_str("\nNext steps:\n");
        for next_step in &response.suggested_next_steps {
            rendered.push_str(&format!("- {next_step}\n"));
        }
    }

    rendered
}
