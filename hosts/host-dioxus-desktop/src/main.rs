use dioxus::prelude::*;
use host_core::{Playground, default_payload_text, render_response, supports_host};
use plugin_manifest::{PluginArchitecture, PluginManifest, SkillLevel};
use plugin_protocol::HostKind;

// ---------------------------------------------------------------------------
// Design-system: dark navy/blue palette with consistent style functions
// ---------------------------------------------------------------------------

mod styles {
    pub const BG: &str = "#0b1020";
    pub const PANEL: &str = "#121933";
    pub const PANEL_2: &str = "#182042";
    pub const BORDER: &str = "#2a376d";
    pub const TEXT: &str = "#edf2ff";
    pub const MUTED: &str = "#9fb1e2";
    pub const ACCENT: &str = "#70a5ff";
    pub const ACCENT_2: &str = "#87f0d4";
}

fn page_style() -> String {
    format!(
        "display: flex; flex-direction: column; font-family: 'Inter', 'Segoe UI', system-ui, sans-serif; \
         background: {}; color: {}; min-height: 100vh; margin: 0; padding: 0;",
        styles::BG,
        styles::TEXT,
    )
}

fn sidebar_style() -> String {
    format!(
        "width: 300px; min-width: 300px; background: {}; border-right: 1px solid {}; \
         padding: 20px 16px; overflow-y: auto; height: 100vh; box-sizing: border-box;",
        styles::PANEL,
        styles::BORDER,
    )
}

fn main_content_style() -> String {
    format!(
        "flex: 1; padding: 28px 32px 80px 32px; overflow-y: auto; height: 100vh; \
         box-sizing: border-box; background: {};",
        styles::BG,
    )
}

fn card_style() -> String {
    format!(
        "border: 1px solid {}; border-radius: 10px; padding: 16px; background: {};",
        styles::BORDER,
        styles::PANEL,
    )
}

fn metric_card_style() -> String {
    format!(
        "border: 1px solid {}; border-radius: 10px; padding: 18px 20px; background: {}; \
         text-align: center; min-width: 0;",
        styles::BORDER,
        styles::PANEL,
    )
}

fn plugin_button_style(selected: bool) -> String {
    if selected {
        format!(
            "display: block; width: 100%; margin-bottom: 10px; padding: 12px 14px; \
             border-radius: 8px; border: 1px solid {}; background: {}; color: {}; \
             text-align: left; cursor: pointer; transition: border-color 0.15s;",
            styles::ACCENT,
            styles::PANEL_2,
            styles::TEXT,
        )
    } else {
        format!(
            "display: block; width: 100%; margin-bottom: 10px; padding: 12px 14px; \
             border-radius: 8px; border: 1px solid {}; background: {}; color: {}; \
             text-align: left; cursor: pointer; transition: border-color 0.15s;",
            styles::BORDER,
            styles::PANEL,
            styles::TEXT,
        )
    }
}

fn action_button_style(selected: bool) -> String {
    if selected {
        format!(
            "display: block; width: 100%; margin-bottom: 8px; padding: 10px 14px; \
             border-radius: 8px; border: 1px solid {}; background: {}; color: {}; \
             text-align: left; cursor: pointer;",
            styles::ACCENT,
            styles::PANEL_2,
            styles::TEXT,
        )
    } else {
        format!(
            "display: block; width: 100%; margin-bottom: 8px; padding: 10px 14px; \
             border-radius: 8px; border: 1px solid {}; background: {}; color: {}; \
             text-align: left; cursor: pointer;",
            styles::BORDER,
            styles::PANEL,
            styles::TEXT,
        )
    }
}

fn primary_button_style() -> String {
    format!(
        "padding: 10px 22px; border-radius: 8px; border: none; \
         background: linear-gradient(135deg, {}, {}); color: {}; \
         font-weight: 600; cursor: pointer; font-size: 14px;",
        styles::ACCENT,
        styles::ACCENT_2,
        styles::BG,
    )
}

fn secondary_button_style() -> String {
    format!(
        "padding: 10px 18px; border-radius: 8px; border: 1px solid {}; \
         background: {}; color: {}; cursor: pointer; font-size: 14px;",
        styles::BORDER,
        styles::PANEL_2,
        styles::MUTED,
    )
}

fn status_bar_style() -> String {
    format!(
        "position: fixed; bottom: 0; left: 300px; right: 0; \
         padding: 8px 32px; background: {}; border-top: 1px solid {}; \
         color: {}; font-size: 13px; display: flex; gap: 24px; \
         align-items: center; z-index: 10;",
        styles::PANEL,
        styles::BORDER,
        styles::MUTED,
    )
}

fn code_block_style() -> String {
    format!(
        "background: {}; border: 1px solid {}; border-radius: 8px; padding: 14px; \
         white-space: pre-wrap; overflow-x: auto; font-family: 'JetBrains Mono', \
         'Fira Code', 'Cascadia Code', monospace; font-size: 13px; color: {}; \
         line-height: 1.5;",
        styles::BG,
        styles::BORDER,
        styles::TEXT,
    )
}

fn textarea_style() -> String {
    format!(
        "width: 100%; min-height: 160px; padding: 12px; border-radius: 8px; \
         background: {}; color: {}; border: 1px solid {}; \
         font-family: 'JetBrains Mono', 'Fira Code', monospace; font-size: 13px; \
         line-height: 1.5; box-sizing: border-box; resize: vertical;",
        styles::BG,
        styles::TEXT,
        styles::BORDER,
    )
}

fn badge_style(color: &str) -> String {
    format!(
        "display: inline-block; padding: 2px 10px; border-radius: 12px; \
         font-size: 11px; font-weight: 600; background: {color}22; color: {color}; \
         border: 1px solid {color}55; margin-right: 6px;",
    )
}

fn section_header_style() -> String {
    format!(
        "font-size: 13px; font-weight: 700; text-transform: uppercase; \
         letter-spacing: 0.08em; color: {}; margin: 0 0 12px 0; padding-bottom: 8px; \
         border-bottom: 1px solid {};",
        styles::MUTED,
        styles::BORDER,
    )
}

fn details_summary_style() -> String {
    format!(
        "font-size: 13px; font-weight: 700; text-transform: uppercase; \
         letter-spacing: 0.08em; color: {}; cursor: pointer; \
         padding: 10px 0; user-select: none;",
        styles::MUTED,
    )
}

fn detail_row_style() -> String {
    format!(
        "display: flex; justify-content: space-between; padding: 6px 0; \
         border-bottom: 1px solid {}20; font-size: 14px;",
        styles::BORDER,
    )
}

// ---------------------------------------------------------------------------
// Data helpers
// ---------------------------------------------------------------------------

struct LoadedApp {
    playground: Playground,
    manifests: Vec<PluginManifest>,
}

fn load_app() -> Result<LoadedApp, String> {
    Playground::load_default()
        .map(|playground| LoadedApp {
            manifests: playground.manifests(),
            playground,
        })
        .map_err(|error| error.to_string())
}

fn format_architecture(manifest: &PluginManifest) -> &'static str {
    match manifest.architecture {
        PluginArchitecture::NativeJson => "Native JSON",
        PluginArchitecture::AbiStable => "ABI-stable",
        PluginArchitecture::Wasm => "Wasm",
    }
}

fn architecture_color(manifest: &PluginManifest) -> &'static str {
    match manifest.architecture {
        PluginArchitecture::NativeJson => styles::ACCENT,
        PluginArchitecture::AbiStable => "#e8a838",
        PluginArchitecture::Wasm => styles::ACCENT_2,
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

// ---------------------------------------------------------------------------
// Reusable Dioxus components
// ---------------------------------------------------------------------------

#[component]
fn MetricCard(emoji: String, label: String, value: String) -> Element {
    rsx! {
        div {
            style: "{metric_card_style()}",
            div { style: "font-size: 24px; margin-bottom: 4px;", "{emoji}" }
            div {
                style: "font-size: 22px; font-weight: 700; color: {styles::TEXT}; margin-bottom: 2px;",
                "{value}"
            }
            div {
                style: "font-size: 12px; color: {styles::MUTED}; text-transform: uppercase; letter-spacing: 0.06em;",
                "{label}"
            }
        }
    }
}

#[component]
fn Badge(label: String, color: String) -> Element {
    rsx! {
        span { style: "{badge_style(&color)}", "{label}" }
    }
}

#[component]
fn SectionHeader(title: String) -> Element {
    rsx! {
        h3 { style: "{section_header_style()}", "{title}" }
    }
}

#[component]
fn PluginCard(manifest: PluginManifest, selected: bool, on_select: EventHandler<()>) -> Element {
    let arch_color = architecture_color(&manifest);
    let arch_label = format_architecture(&manifest).to_owned();
    let skill_label = format_skill(&manifest).to_owned();
    let action_count = manifest.actions.len();

    rsx! {
        button {
            style: "{plugin_button_style(selected)}",
            onclick: move |_| on_select.call(()),
            div {
                style: "font-weight: 600; font-size: 15px; margin-bottom: 4px; color: {styles::TEXT};",
                "{manifest.name}"
            }
            div {
                style: "font-size: 13px; color: {styles::MUTED}; margin-bottom: 8px; line-height: 1.4;",
                "{manifest.description}"
            }
            div {
                style: "display: flex; flex-wrap: wrap; gap: 4px;",
                Badge { label: arch_label, color: arch_color.to_owned() }
                Badge { label: skill_label, color: "#c084fc".to_owned() }
                Badge { label: format!("{action_count} action(s)"), color: styles::ACCENT.to_owned() }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Main application
// ---------------------------------------------------------------------------

fn main() {
    dioxus::launch(App);
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

    let dioxus_ready_count = manifests
        .iter()
        .filter(|m| supports_host(m, HostKind::Dioxus))
        .count();

    let selected_plugin_label = current_manifest
        .as_ref()
        .map(|m| m.name.clone())
        .unwrap_or_else(|| "—".to_owned());
    let selected_action_label = current_action
        .as_ref()
        .map(|a| a.label.clone())
        .unwrap_or_else(|| "—".to_owned());

    rsx! {
        div {
            style: "{page_style()} flex-direction: row;",

            // ---- Sidebar: plugin catalog ----
            aside {
                style: "{sidebar_style()}",
                div {
                    style: "font-size: 20px; font-weight: 700; margin-bottom: 4px; color: {styles::TEXT};",
                    "⚡ Plugin Catalog"
                }
                div {
                    style: "font-size: 13px; color: {styles::MUTED}; margin-bottom: 20px;",
                    "Desktop-ready comparison surface"
                }

                if let Some(ref error) = load_error {
                    div {
                        style: "color: #ff7b7b; padding: 12px; border: 1px solid #ff7b7b44; border-radius: 8px; background: #ff7b7b11;",
                        "{error}"
                    }
                }

                for manifest in manifests.iter() {
                    {
                        let plugin_id = manifest.id.clone();
                        let plugin_name = manifest.name.clone();
                        let first_action = manifest.actions.first().cloned();
                        let selected = selected_plugin_id.read().as_deref() == Some(plugin_id.as_str());
                        rsx! {
                            PluginCard {
                                manifest: manifest.clone(),
                                selected,
                                on_select: move |()| {
                                    selected_plugin_id.set(Some(plugin_id.clone()));
                                    selected_action_id.set(first_action.as_ref().map(|a| a.id.clone()));
                                    payload_input.set(
                                        first_action
                                            .as_ref()
                                            .map(default_payload_text)
                                            .unwrap_or_else(|| "{}".to_owned()),
                                    );
                                    status.set(format!("Selected plugin {}", plugin_name));
                                },
                            }
                        }
                    }
                }
            }

            // ---- Main content area ----
            main {
                style: "{main_content_style()}",

                // Header
                div {
                    style: "margin-bottom: 24px;",
                    h1 {
                        style: "font-size: 26px; font-weight: 700; margin: 0 0 4px 0; color: {styles::TEXT};",
                        "Dioxus Desktop Host"
                    }
                    p {
                        style: "margin: 0; color: {styles::MUTED}; font-size: 14px;",
                        "Reactive desktop surface for plugin/action selection, payload templates, and local invocation."
                    }
                }

                if load_error.is_none() {
                    // ---- Metric cards row ----
                    div {
                        style: "display: grid; grid-template-columns: repeat(4, 1fr); gap: 14px; margin-bottom: 24px;",
                        MetricCard { emoji: "🧩".to_owned(), label: "Plugins".to_owned(), value: manifests.len().to_string() }
                        MetricCard { emoji: "🖥️".to_owned(), label: "Dioxus-ready".to_owned(), value: dioxus_ready_count.to_string() }
                        MetricCard { emoji: "⚠️".to_owned(), label: "Warnings".to_owned(), value: warnings.len().to_string() }
                        MetricCard { emoji: "📂".to_owned(), label: "Plugin dir".to_owned(), value: plugin_dir.clone() }
                    }

                    // ---- Two-column layout: inspector + action studio ----
                    div {
                        style: "display: grid; grid-template-columns: 1fr 1.2fr; gap: 20px; align-items: start;",

                        // ---- Manifest inspector ----
                        section {
                            style: "{card_style()}",
                            SectionHeader { title: "Manifest Inspector".to_owned() }

                            if let Some(ref manifest) = current_manifest {
                                div {
                                    style: "font-size: 18px; font-weight: 600; margin-bottom: 4px;",
                                    "{manifest.name}"
                                }
                                div {
                                    style: "font-size: 14px; color: {styles::MUTED}; margin-bottom: 16px;",
                                    "{manifest.description}"
                                }

                                // Collapsible: manifest details
                                details {
                                    open: true,
                                    summary { style: "{details_summary_style()}", "📋 Manifest Details" }
                                    div {
                                        style: "padding: 8px 0;",
                                        div { style: "{detail_row_style()}", span { style: "color: {styles::MUTED};", "ID" } span { "{manifest.id}" } }
                                        div { style: "{detail_row_style()}", span { style: "color: {styles::MUTED};", "Version" } span { "{manifest.version}" } }
                                        div { style: "{detail_row_style()}", span { style: "color: {styles::MUTED};", "Architecture" } span { "{format_architecture(manifest)}" } }
                                        div { style: "{detail_row_style()}", span { style: "color: {styles::MUTED};", "Skill Level" } span { "{format_skill(manifest)}" } }
                                        div { style: "{detail_row_style()}", span { style: "color: {styles::MUTED};", "Hosts" } span { "{supported_hosts_text(manifest)}" } }
                                        div { style: "{detail_row_style()}", span { style: "color: {styles::MUTED};", "Tags" } span { "{tags_text(manifest)}" } }
                                    }
                                }

                                // Collapsible: capabilities & notes
                                details {
                                    summary { style: "{details_summary_style()}", "🔧 Capabilities & Notes" }
                                    div {
                                        style: "padding: 8px 0;",
                                        div {
                                            style: "font-size: 12px; font-weight: 600; color: {styles::MUTED}; text-transform: uppercase; margin-bottom: 6px;",
                                            "Capabilities"
                                        }
                                        if manifest.capabilities.is_empty() {
                                            div { style: "color: {styles::MUTED}; font-size: 13px; font-style: italic;", "None declared" }
                                        } else {
                                            for capability in manifest.capabilities.iter() {
                                                div {
                                                    style: "padding: 4px 0; font-size: 13px;",
                                                    span { style: "font-weight: 600; color: {styles::ACCENT};", "{capability.key}" }
                                                    span { style: "color: {styles::MUTED};", " — {capability.description}" }
                                                }
                                            }
                                        }

                                        div {
                                            style: "font-size: 12px; font-weight: 600; color: {styles::MUTED}; text-transform: uppercase; margin: 14px 0 6px 0;",
                                            "Notes"
                                        }
                                        if manifest.notes.is_empty() {
                                            div { style: "color: {styles::MUTED}; font-size: 13px; font-style: italic;", "No notes" }
                                        } else {
                                            for note in manifest.notes.iter() {
                                                div {
                                                    style: "padding: 4px 0; font-size: 13px; color: {styles::MUTED};",
                                                    "• {note}"
                                                }
                                            }
                                        }
                                    }
                                }

                                if !warnings.is_empty() {
                                    details {
                                        summary { style: "{details_summary_style()}", "⚠️ Discovery Warnings" }
                                        div {
                                            style: "padding: 8px 0;",
                                            for warning in warnings.iter() {
                                                div {
                                                    style: "padding: 4px 0; font-size: 13px; color: #fbbf24;",
                                                    "• {warning}"
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                div {
                                    style: "color: {styles::MUTED}; font-style: italic; padding: 20px 0;",
                                    "Select a plugin from the sidebar"
                                }
                            }
                        }

                        // ---- Action studio ----
                        section {
                            style: "{card_style()}",
                            SectionHeader { title: "Action Studio".to_owned() }

                            if let Some(ref manifest) = current_manifest {
                                // Action list (always visible)
                                div {
                                    style: "margin-bottom: 16px;",
                                    div {
                                        style: "font-size: 12px; font-weight: 600; color: {styles::MUTED}; text-transform: uppercase; margin-bottom: 8px;",
                                        "Available Actions"
                                    }
                                    for action in manifest.actions.iter() {
                                        {
                                            let action_id = action.id.clone();
                                            let action_label = action.label.clone();
                                            let action_template = default_payload_text(action);
                                            let selected = current_action
                                                .as_ref()
                                                .map(|s| s.id.as_str())
                                                == Some(action.id.as_str());
                                            rsx! {
                                                button {
                                                    style: "{action_button_style(selected)}",
                                                    onclick: move |_| {
                                                        selected_action_id.set(Some(action_id.clone()));
                                                        payload_input.set(action_template.clone());
                                                        status.set(format!("Selected action {}", action_label));
                                                    },
                                                    div { style: "font-weight: 600; font-size: 14px;", "{action.label}" }
                                                    div { style: "font-size: 13px; color: {styles::MUTED};", "{action.description}" }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Payload editor (always visible)
                                if let Some(ref action) = current_action {
                                    div {
                                        style: "margin-bottom: 16px;",
                                        div {
                                            style: "font-size: 12px; font-weight: 600; color: {styles::MUTED}; text-transform: uppercase; margin-bottom: 6px;",
                                            "Payload Editor"
                                        }
                                        div {
                                            style: "font-size: 12px; color: {styles::MUTED}; margin-bottom: 8px; font-style: italic;",
                                            "Template hint for this action:"
                                        }
                                        pre {
                                            style: "{code_block_style()} margin-bottom: 10px;",
                                            "{default_payload_text(action)}"
                                        }
                                        textarea {
                                            style: "{textarea_style()}",
                                            value: "{payload_input.read()}",
                                            oninput: move |event| payload_input.set(event.value()),
                                        }

                                        // Button row
                                        div {
                                            style: "display: flex; gap: 10px; margin-top: 12px;",
                                            button {
                                                style: "{primary_button_style()}",
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
                                                "▶ Run"
                                            }
                                            button {
                                                style: "{secondary_button_style()}",
                                                onclick: {
                                                    let action = action.clone();
                                                    move |_| {
                                                        payload_input.set(default_payload_text(&action));
                                                        status.set("Applied payload template".to_owned());
                                                    }
                                                },
                                                "Apply Template"
                                            }
                                            button {
                                                style: "{secondary_button_style()}",
                                                onclick: move |_| {
                                                    payload_input.set("{}".to_owned());
                                                    status.set("Cleared payload".to_owned());
                                                },
                                                "Clear"
                                            }
                                        }
                                    }
                                }

                                // Invocation output
                                div {
                                    div {
                                        style: "font-size: 12px; font-weight: 600; color: {styles::MUTED}; text-transform: uppercase; margin-bottom: 8px;",
                                        "Invocation Output"
                                    }
                                    pre {
                                        style: "{code_block_style()}",
                                        "{output.read()}"
                                    }
                                }
                            } else {
                                div {
                                    style: "color: {styles::MUTED}; font-style: italic; padding: 20px 0;",
                                    "Select a plugin from the sidebar"
                                }
                            }
                        }
                    }
                }

                // ---- Status bar ----
                div {
                    style: "{status_bar_style()}",
                    span { "Status: {status.read()}" }
                    span { style: "opacity: 0.5;", "│" }
                    span { "Plugin: {selected_plugin_label}" }
                    span { style: "opacity: 0.5;", "│" }
                    span { "Action: {selected_action_label}" }
                }
            }
        }
    }
}
