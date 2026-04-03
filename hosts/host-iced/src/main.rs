use host_core::{Playground, default_payload_text, render_response, supports_host};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};
use plugin_manifest::{PluginAction, PluginArchitecture, PluginManifest, SkillLevel};
use plugin_protocol::HostKind;

pub fn main() -> iced::Result {
    iced::application(IcedHostApp::default, update, view).run()
}

struct IcedHostApp {
    playground: Option<Playground>,
    manifests: Vec<PluginManifest>,
    error: Option<String>,
    selected_plugin_id: Option<String>,
    selected_action_id: Option<String>,
    payload_input: String,
    output: String,
    status: String,
}

#[derive(Debug, Clone)]
enum Message {
    SelectPlugin(String),
    SelectAction(String),
    PayloadChanged(String),
    ApplyTemplate,
    ClearPayload,
    InvokeSelected,
}

impl Default for IcedHostApp {
    fn default() -> Self {
        match Playground::load_default() {
            Ok(playground) => {
                let manifests = playground.manifests();
                let selected_plugin_id = manifests.first().map(|manifest| manifest.id.clone());
                let selected_action_id = manifests
                    .first()
                    .and_then(|manifest| manifest.actions.first())
                    .map(|action| action.id.clone());
                let payload_input = manifests
                    .first()
                    .and_then(|manifest| manifest.actions.first())
                    .map(default_payload_text)
                    .unwrap_or_else(|| "{}".to_owned());
                Self {
                    playground: Some(playground),
                    manifests,
                    error: None,
                    selected_plugin_id,
                    selected_action_id,
                    payload_input,
                    output:
                        "Iced host is ready to compare plugin metadata and explicit action state."
                            .to_owned(),
                    status: "Idle".to_owned(),
                }
            }
            Err(error) => Self {
                playground: None,
                manifests: Vec::new(),
                error: Some(error.to_string()),
                selected_plugin_id: None,
                selected_action_id: None,
                payload_input: "{}".to_owned(),
                output: String::new(),
                status: String::new(),
            },
        }
    }
}

fn update(state: &mut IcedHostApp, message: Message) {
    match message {
        Message::SelectPlugin(plugin_id) => {
            state.selected_plugin_id = Some(plugin_id);
            state.selected_action_id = state
                .selected_manifest()
                .and_then(|manifest| manifest.actions.first())
                .map(|action| action.id.clone());
            state.sync_payload();
            state.status = "Selected plugin".to_owned();
        }
        Message::SelectAction(action_id) => {
            state.selected_action_id = Some(action_id);
            state.sync_payload();
            state.status = "Loaded action payload template".to_owned();
        }
        Message::PayloadChanged(payload) => {
            state.payload_input = payload;
        }
        Message::ApplyTemplate => {
            state.sync_payload();
            state.status = "Applied payload template".to_owned();
        }
        Message::ClearPayload => {
            state.payload_input = "{}".to_owned();
            state.status = "Cleared payload".to_owned();
        }
        Message::InvokeSelected => {
            state.invoke_selected();
        }
    }
}

fn view(state: &IcedHostApp) -> Element<'_, Message> {
    if let Some(error) = &state.error {
        return container(text(error)).into();
    }

    let Some(playground) = &state.playground else {
        return container(text("No playground loaded")).into();
    };

    let sidebar =
        state
            .manifests
            .iter()
            .fold(column![text("Plugins").size(28)], |column, manifest| {
                let host_fit = if supports_host(manifest, HostKind::Iced) {
                    "Iced-ready"
                } else {
                    "Cross-host"
                };
                column.push(
                    button(text(format!(
                        "{} · {} · {}",
                        manifest.name,
                        manifest.actions.len(),
                        host_fit
                    )))
                    .width(Length::Fill)
                    .on_press(Message::SelectPlugin(manifest.id.clone())),
                )
            });

    let details = if let Some(manifest) = state.selected_manifest() {
        let metadata = column![
            text(manifest.name.clone()).size(28),
            text(manifest.description.clone()),
            text(format!(
                "Version {} · {} · {}",
                manifest.version,
                format_architecture(manifest),
                format_skill(manifest)
            )),
            text(format!(
                "Supported hosts: {}",
                supported_hosts_text(manifest)
            )),
            text(format!("Tags: {}", tags_text(manifest))),
            text("Actions").size(22),
        ];

        let actions = manifest.actions.iter().fold(metadata, |column, action| {
            let selected = state.selected_action_id.as_deref() == Some(action.id.as_str());
            let label = if selected {
                format!("▶ {} — {}", action.label, action.description)
            } else {
                format!("{} — {}", action.label, action.description)
            };
            column.push(button(text(label)).on_press(Message::SelectAction(action.id.clone())))
        });

        let payload_controls = column![
            text("Payload template / editor").size(22),
            text_input("JSON payload", &state.payload_input)
                .on_input(Message::PayloadChanged)
                .padding(8)
                .size(16),
            row![
                button("Apply template").on_press(Message::ApplyTemplate),
                button("Clear").on_press(Message::ClearPayload),
                button("Run action").on_press(Message::InvokeSelected),
            ]
            .spacing(8),
        ]
        .spacing(8);

        let capabilities = if manifest.capabilities.is_empty() {
            column![text("Capabilities: none")]
        } else {
            manifest.capabilities.iter().fold(
                column![text("Capabilities").size(22)],
                |column, capability| {
                    column.push(text(format!(
                        "• {} — {}",
                        capability.key, capability.description
                    )))
                },
            )
        };

        column![
            actions.spacing(10),
            payload_controls,
            capabilities.spacing(8),
            text(format!("State: {}", state.status)).size(16),
        ]
    } else {
        column![text("No plugin selected")]
    };

    let summary = row![
        metric_panel("Plugins", state.manifests.len()),
        metric_panel(
            "Iced-ready",
            state
                .manifests
                .iter()
                .filter(|manifest| supports_host(manifest, HostKind::Iced))
                .count()
        ),
        metric_panel("Warnings", playground.warnings().len()),
    ]
    .spacing(12);

    row![
        container(scrollable(sidebar.spacing(8))).width(Length::FillPortion(1)),
        container(scrollable(
            details
                .spacing(12)
                .push(summary)
                .push(text("Output").size(22))
                .push(text(state.output.clone()))
        ))
        .width(Length::FillPortion(2)),
    ]
    .spacing(16)
    .padding(16)
    .into()
}

impl IcedHostApp {
    fn selected_manifest(&self) -> Option<&PluginManifest> {
        selected_manifest(&self.manifests, self.selected_plugin_id.as_deref())
    }

    fn sync_payload(&mut self) {
        if let Some(action) = self
            .selected_manifest()
            .and_then(|manifest| selected_action(manifest, self.selected_action_id.as_deref()))
        {
            self.payload_input = default_payload_text(action);
        }
    }

    fn invoke_selected(&mut self) {
        let Some(playground) = &self.playground else {
            return;
        };
        let Some((plugin_id, plugin_name)) = self
            .selected_manifest()
            .map(|manifest| (manifest.id.clone(), manifest.name.clone()))
        else {
            self.output = "No plugin selected".to_owned();
            return;
        };
        let Some((action_id, action_label)) = self.selected_manifest().and_then(|manifest| {
            selected_action(manifest, self.selected_action_id.as_deref())
                .map(|action| (action.id.clone(), action.label.clone()))
        }) else {
            self.output = "No action selected".to_owned();
            return;
        };
        match playground.invoke_text(&plugin_id, &action_id, &self.payload_input, HostKind::Iced) {
            Ok(response) => {
                self.output = render_response(&response);
                self.status = format!("Ran {} / {}", plugin_name, action_label);
            }
            Err(error) => {
                self.output = error.to_string();
                self.status = format!("Invocation failed for {} / {}", plugin_name, action_label);
            }
        }
    }
}

fn selected_manifest<'a>(
    manifests: &'a [PluginManifest],
    plugin_id: Option<&str>,
) -> Option<&'a PluginManifest> {
    let plugin_id = plugin_id?;
    manifests.iter().find(|manifest| manifest.id == plugin_id)
}

fn selected_action<'a>(
    manifest: &'a PluginManifest,
    action_id: Option<&str>,
) -> Option<&'a PluginAction> {
    let action_id = action_id?;
    manifest
        .actions
        .iter()
        .find(|action| action.id == action_id)
}

fn metric_panel<'a>(label: &'a str, value: usize) -> Element<'a, Message> {
    container(column![text(label), text(value.to_string()).size(24)].spacing(4))
        .padding(10)
        .width(Length::Shrink)
        .into()
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
