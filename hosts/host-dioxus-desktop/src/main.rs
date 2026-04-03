use dioxus::prelude::*;
use host_core::{Playground, default_payload_text, render_response, supports_host};
use plugin_manifest::{PluginArchitecture, PluginManifest, SkillLevel};
use plugin_protocol::HostKind;

struct LoadedApp {
    playground: Playground,
    manifests: Vec<PluginManifest>,
}

fn main() {
    dioxus::launch(App);
}

fn load_app() -> Result<LoadedApp, String> {
    Playground::load_default()
        .map(|playground| LoadedApp {
            manifests: playground.manifests(),
            playground,
        })
        .map_err(|error| error.to_string())
}

#[component]
fn App() -> Element {
    let app = use_signal(load_app);
    let mut selected_plugin_id = use_signal(|| None::<String>);
    let mut selected_action_id = use_signal(|| None::<String>);
    let mut payload_input = use_signal(|| "{}".to_owned());
    let mut output = use_signal(|| "Reactive desktop host ready.".to_owned());
    let mut status = use_signal(|| "Ready".to_owned());

    let manifests = app
        .read()
        .as_ref()
        .map(|loaded| loaded.manifests.clone())
        .unwrap_or_default();

    if selected_plugin_id.read().is_none()
        && let Some(manifest) = manifests.first()
    {
        selected_plugin_id.set(Some(manifest.id.clone()));
        selected_action_id.set(manifest.actions.first().map(|action| action.id.clone()));
        payload_input.set(
            manifest
                .actions
                .first()
                .map(default_payload_text)
                .unwrap_or_else(|| "{}".to_owned()),
        );
    }

    let current_manifest = manifests
        .iter()
        .find(|manifest| selected_plugin_id.read().as_deref() == Some(manifest.id.as_str()))
        .cloned();
    let current_action = current_manifest.as_ref().and_then(|manifest| {
        manifest
            .actions
            .iter()
            .find(|action| selected_action_id.read().as_deref() == Some(action.id.as_str()))
            .cloned()
    });
    let warnings = app
        .read()
        .as_ref()
        .map(|loaded| loaded.playground.warnings().to_vec())
        .unwrap_or_default();
    let plugin_dir = app
        .read()
        .as_ref()
        .map(|loaded| loaded.playground.plugin_dir().display().to_string())
        .unwrap_or_else(|_| "unavailable".to_owned());
    let load_error = app.read().as_ref().err().cloned();

    rsx! {
        div {
            style: "font-family: sans-serif; padding: 24px; background: #0c1220; color: #f4f7fb; min-height: 100vh;",
            h1 { "Dioxus Desktop Host" }
            p { "Reactive desktop surface for explicit plugin/action selection, payload templates, and local invocation." }

            if let Some(error) = load_error {
                p { style: "color: #ff7b7b;", "{error}" }
            } else {
                div {
                    style: "display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 12px; margin-bottom: 18px;",
                    div { style: panel_style(), p { "Plugins" } h3 { "{manifests.len()}" } }
                    div {
                        style: panel_style(),
                        p { "Dioxus-ready" }
                        h3 { "{manifests.iter().filter(|manifest| supports_host(manifest, HostKind::Dioxus)).count()}" }
                    }
                    div { style: panel_style(), p { "Warnings" } h3 { "{warnings.len()}" } }
                    div { style: panel_style(), p { "Plugin dir" } h3 { "{plugin_dir}" } }
                }

                div {
                    style: "display: grid; grid-template-columns: 300px minmax(0, 1fr) minmax(0, 1.1fr); gap: 18px; align-items: start;",

                    section {
                        style: panel_style(),
                        h2 { "Catalog" }
                        p { "Desktop-ready comparison surface" }
                        for manifest in manifests.iter() {
                            {
                                let plugin_id = manifest.id.clone();
                                let plugin_name = manifest.name.clone();
                                let first_action = manifest.actions.first().cloned();
                                let selected = selected_plugin_id.read().as_deref() == Some(plugin_id.as_str());
                                rsx! {
                                    button {
                                        style: if selected { selected_button_style() } else { button_style() },
                                        onclick: move |_| {
                                            selected_plugin_id.set(Some(plugin_id.clone()));
                                            selected_action_id.set(first_action.as_ref().map(|action| action.id.clone()));
                                            payload_input.set(
                                                first_action
                                                    .as_ref()
                                                    .map(default_payload_text)
                                                    .unwrap_or_else(|| "{}".to_owned())
                                            );
                                            status.set(format!("Selected plugin {}", plugin_name));
                                        },
                                        strong { "{manifest.name}" }
                                        p { "{manifest.description}" }
                                        small { "{manifest.actions.len()} action(s) · {format_architecture(manifest)} · {format_skill(manifest)}" }
                                        small { "Hosts: {supported_hosts_text(manifest)}" }
                                    }
                                }
                            }
                        }
                    }

                    section {
                        style: panel_style(),
                        h2 { "Manifest inspector" }
                        if let Some(manifest) = current_manifest.as_ref() {
                            h3 { "{manifest.name}" }
                            p { "{manifest.description}" }
                            ul {
                                li { "ID: {manifest.id}" }
                                li { "Version: {manifest.version}" }
                                li { "Architecture: {format_architecture(manifest)}" }
                                li { "Skill level: {format_skill(manifest)}" }
                                li { "Supported hosts: {supported_hosts_text(manifest)}" }
                                li { "Tags: {tags_text(manifest)}" }
                            }
                            h4 { "Capabilities" }
                            ul {
                                if manifest.capabilities.is_empty() {
                                    li { "none" }
                                } else {
                                    for capability in manifest.capabilities.iter() {
                                        li { "{capability.key} — {capability.description}" }
                                    }
                                }
                            }
                            h4 { "Notes" }
                            ul {
                                if manifest.notes.is_empty() {
                                    li { "none" }
                                } else {
                                    for note in manifest.notes.iter() {
                                        li { "{note}" }
                                    }
                                }
                            }
                            if !warnings.is_empty() {
                                h4 { "Discovery warnings" }
                                ul {
                                    for warning in warnings.iter() {
                                        li { "{warning}" }
                                    }
                                }
                            }
                        } else {
                            p { "No plugin selected" }
                        }
                    }

                    section {
                        style: panel_style(),
                        h2 { "Action studio" }
                        p { "{status.read()}" }
                        if let Some(manifest) = current_manifest.as_ref() {
                            h3 { "Actions" }
                            for action in manifest.actions.iter() {
                                {
                                    let action_id = action.id.clone();
                                    let action_label = action.label.clone();
                                    let action_template = default_payload_text(action);
                                    let selected = current_action
                                        .as_ref()
                                        .map(|selected| selected.id.as_str())
                                        == Some(action.id.as_str());
                                    rsx! {
                                        button {
                                            style: if selected { selected_button_style() } else { button_style() },
                                            onclick: move |_| {
                                                selected_action_id.set(Some(action_id.clone()));
                                                payload_input.set(action_template.clone());
                                                status.set(format!("Selected action {}", action_label));
                                            },
                                            strong { "{action.label}" }
                                            p { "{action.description}" }
                                        }
                                    }
                                }
                            }

                            if let Some(action) = current_action.as_ref() {
                                h3 { "Payload editor" }
                                p { "Payload hint" }
                                pre {
                                    style: code_style(),
                                    "{default_payload_text(action)}"
                                }
                                textarea {
                                    style: textarea_style(),
                                    value: "{payload_input.read()}",
                                    oninput: move |event| payload_input.set(event.value()),
                                }
                                div {
                                    style: "display: flex; gap: 10px; margin-top: 10px;",
                                    button {
                                        style: button_style(),
                                        onclick: {
                                            let action = action.clone();
                                            move |_| {
                                                payload_input.set(default_payload_text(&action));
                                                status.set("Applied payload template".to_owned());
                                            }
                                        },
                                        "Apply template"
                                    }
                                    button {
                                        style: button_style(),
                                        onclick: move |_| {
                                            payload_input.set("{}".to_owned());
                                            status.set("Cleared payload".to_owned());
                                        },
                                        "Clear"
                                    }
                                    button {
                                        style: selected_button_style(),
                                        onclick: move |_| {
                                            let plugin_id = selected_plugin_id.read().clone();
                                            let action_id = selected_action_id.read().clone();
                                            let payload = payload_input.read().clone();
                                            if let (Some(plugin_id), Some(action_id)) = (plugin_id, action_id)
                                                && let Ok(loaded) = &*app.read()
                                            {
                                                match loaded.playground.invoke_text(&plugin_id, &action_id, &payload, HostKind::Dioxus) {
                                                    Ok(response) => {
                                                        output.set(render_response(&response));
                                                        status.set(format!("Ran {} / {}", plugin_id, action_id));
                                                    }
                                                    Err(error) => {
                                                        output.set(format!("Invocation failed\n\n{error}"));
                                                        status.set(format!("Invocation failed for {} / {}", plugin_id, action_id));
                                                    }
                                                }
                                            }
                                        },
                                        "Run action"
                                    }
                                }
                            }
                        } else {
                            p { "No plugin selected" }
                        }

                        h3 { "Invocation output" }
                        pre {
                            style: code_style(),
                            "{output.read()}"
                        }
                    }
                }
            }
        }
    }
}

fn format_architecture(manifest: &PluginManifest) -> &'static str {
    match manifest.architecture {
        PluginArchitecture::NativeJson => "Native JSON",
        PluginArchitecture::AbiStable => "ABI-stable",
        PluginArchitecture::Wasm => "Wasm",
    }
}

fn format_skill(manifest: &PluginManifest) -> &'static str {
    match manifest.skill_level {
        SkillLevel::Basic => "Basic",
        SkillLevel::Intermediate => "Intermediate",
        SkillLevel::Advanced => "Advanced",
        SkillLevel::Expert => "Expert",
    }
}

fn supported_hosts_text(manifest: &PluginManifest) -> String {
    manifest
        .supported_hosts
        .iter()
        .map(|host| host.label())
        .collect::<Vec<_>>()
        .join(", ")
}

fn tags_text(manifest: &PluginManifest) -> String {
    if manifest.tags.is_empty() {
        "none".to_owned()
    } else {
        manifest.tags.join(", ")
    }
}

fn panel_style() -> &'static str {
    "border: 1px solid #2b3550; border-radius: 12px; padding: 14px; background: #121a2d;"
}

fn button_style() -> &'static str {
    "display: block; width: 100%; margin-bottom: 8px; padding: 10px; border-radius: 8px; border: 1px solid #32405e; background: #0f1727; color: inherit; text-align: left; cursor: pointer;"
}

fn selected_button_style() -> &'static str {
    "display: block; width: 100%; margin-bottom: 8px; padding: 10px; border-radius: 8px; border: 1px solid #7bdff2; background: #18233a; color: inherit; text-align: left; cursor: pointer;"
}

fn code_style() -> &'static str {
    "background: #091120; border: 1px solid #27324e; border-radius: 8px; padding: 12px; white-space: pre-wrap; overflow-x: auto;"
}

fn textarea_style() -> &'static str {
    "width: 100%; min-height: 180px; padding: 10px; border-radius: 8px; background: #091120; color: #f4f7fb; border: 1px solid #27324e;"
}
