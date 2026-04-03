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

    if let Some(execution) = &response.execution {
        rendered.push_str("\nExecution:\n");
        if let Some(mode) = execution.mode {
            rendered.push_str(&format!("- mode: {:?}\n", mode));
        }
        rendered.push_str(&format!("- async support: {}\n", execution.supports_async));
        rendered.push_str(&format!("- cancellable: {}\n", execution.cancellable));
        if let Some(timeout_ms) = execution.timeout_ms {
            rendered.push_str(&format!("- timeout ms: {timeout_ms}\n"));
        }
        if let Some(lifecycle_state) = execution.lifecycle_state {
            rendered.push_str(&format!("- lifecycle state: {:?}\n", lifecycle_state));
        }
        if let Some(progress_message) = &execution.progress_message {
            rendered.push_str(&format!("- progress: {progress_message}\n"));
        }
        if let Some(job) = &execution.job {
            rendered.push_str("- job:\n");
            if let Some(job_id) = &job.job_id {
                rendered.push_str(&format!("  - id: {job_id}\n"));
            }
            if let Some(state) = job.state {
                rendered.push_str(&format!("  - state: {:?}\n", state));
            }
            if let Some(progress) = &job.progress {
                rendered.push_str(&format!("  - progress: {progress}\n"));
            }
        }
    }

    if let Some(negotiation) = &response.negotiation {
        rendered.push_str("\nNegotiation:\n");
        rendered.push_str(&format!("- status: {:?}\n", negotiation.status));
        if !negotiation.summary.is_empty() {
            rendered.push_str(&format!("- summary: {}\n", negotiation.summary));
        }
        if !negotiation.granted_capabilities.is_empty() {
            rendered.push_str(&format!(
                "- granted: {}\n",
                negotiation.granted_capabilities.join(", ")
            ));
        }
        if !negotiation.missing_required.is_empty() {
            rendered.push_str("- missing required:\n");
            for requirement in &negotiation.missing_required {
                rendered.push_str(&format!(
                    "  - {}: {}\n",
                    requirement.key, requirement.detail
                ));
            }
        }
        if !negotiation.missing_optional.is_empty() {
            rendered.push_str("- missing optional:\n");
            for requirement in &negotiation.missing_optional {
                rendered.push_str(&format!(
                    "  - {}: {}\n",
                    requirement.key, requirement.detail
                ));
            }
        }
        if !negotiation.degraded_features.is_empty() {
            rendered.push_str("- degraded features:\n");
            for degraded in &negotiation.degraded_features {
                rendered.push_str(&format!(
                    "  - {} ({:?}): {}\n",
                    degraded.feature, degraded.severity, degraded.reason
                ));
                if let Some(fallback) = &degraded.fallback {
                    rendered.push_str(&format!("    fallback: {fallback}\n"));
                }
            }
        }
    }

    rendered
}
