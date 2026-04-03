use eframe::egui;
use host_core::{Playground, default_payload_text, render_response, supports_host};
use plugin_manifest::{PluginAction, PluginArchitecture, PluginManifest, SkillLevel};
use plugin_protocol::HostKind;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Rust Plugin Playground - egui Host",
        options,
        Box::new(|_creation_context| Ok(Box::new(EguiHostApp::default()))),
    )
}

struct EguiHostApp {
    playground: Option<Playground>,
    manifests: Vec<PluginManifest>,
    error: Option<String>,
    selected_plugin_id: Option<String>,
    selected_action_id: Option<String>,
    payload_input: String,
    output: String,
    status: String,
}

impl Default for EguiHostApp {
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
                        "Select an action, inspect its metadata, tweak the payload, and run it."
                            .to_owned(),
                    status: "Inspector ready".to_owned(),
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

impl eframe::App for EguiHostApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(error) = &self.error {
            ui.colored_label(egui::Color32::RED, error);
            return;
        }

        ui.heading("egui Host Inspector");
        ui.label("A quick visual dashboard for comparing plugin metadata, host support, payload templates, and invocation output.");
        ui.separator();

        ui.horizontal_wrapped(|ui| {
            metric_card(ui, "Plugins", self.manifests.len().to_string());
            metric_card(
                ui,
                "Actions",
                self.manifests
                    .iter()
                    .map(|manifest| manifest.actions.len())
                    .sum::<usize>()
                    .to_string(),
            );
            metric_card(
                ui,
                "egui-ready",
                self.manifests
                    .iter()
                    .filter(|manifest| supports_host(manifest, HostKind::Egui))
                    .count()
                    .to_string(),
            );
            if let Some(playground) = &self.playground {
                metric_card(ui, "Warnings", playground.warnings().len().to_string());
            }
        });

        ui.separator();
        let manifests = self.manifests.clone();
        ui.columns(2, |columns| {
            columns[0].heading("Catalog");
            columns[0].label("Select a plugin to inspect its host fit and action templates.");
            columns[0].separator();
            egui::ScrollArea::vertical().show(&mut columns[0], |ui| {
                for manifest in &manifests {
                    ui.group(|ui| {
                        let selected =
                            self.selected_plugin_id.as_deref() == Some(manifest.id.as_str());
                        if ui
                            .selectable_label(
                                selected,
                                format!("{} · {} action(s)", manifest.name, manifest.actions.len()),
                            )
                            .clicked()
                        {
                            self.select_plugin(manifest.id.clone());
                        }
                        ui.small(format!(
                            "{} · {} · {}",
                            format_architecture(manifest),
                            format_skill(manifest),
                            if supports_host(manifest, HostKind::Egui) {
                                "works in egui"
                            } else {
                                "cross-host only"
                            }
                        ));
                        if !manifest.tags.is_empty() {
                            ui.small(format!("tags: {}", manifest.tags.join(", ")));
                        }
                    });
                    ui.add_space(6.0);
                }
            });

            columns[1].heading("Selected plugin");
            columns[1].small(&self.status);
            if let Some(playground) = &self.playground
                && !playground.warnings().is_empty()
            {
                columns[1].collapsing("Discovery warnings", |ui| {
                    for warning in playground.warnings() {
                        ui.label(warning);
                    }
                });
            }

            let selected_manifest = self.selected_plugin().cloned();
            if let Some(manifest) = selected_manifest {
                columns[1].group(|ui| {
                    ui.heading(&manifest.name);
                    ui.label(&manifest.description);
                    egui::Grid::new("manifest_metadata")
                        .num_columns(2)
                        .show(ui, |ui| {
                            ui.label("Version");
                            ui.monospace(&manifest.version);
                            ui.end_row();
                            ui.label("Architecture");
                            ui.label(format_architecture(&manifest));
                            ui.end_row();
                            ui.label("Skill");
                            ui.label(format_skill(&manifest));
                            ui.end_row();
                            ui.label("Supported hosts");
                            ui.label(supported_hosts_text(&manifest));
                            ui.end_row();
                            ui.label("Tags");
                            ui.label(tags_text(&manifest));
                            ui.end_row();
                        });
                });

                columns[1].add_space(8.0);
                columns[1].label("Actions");
                for action in &manifest.actions {
                    let selected = self.selected_action_id.as_deref() == Some(action.id.as_str());
                    if columns[1]
                        .selectable_label(
                            selected,
                            format!("{} · {}", action.label, action.description),
                        )
                        .clicked()
                    {
                        self.select_action(action.id.clone());
                    }
                }

                if let Some(action) = selected_action(&manifest, self.selected_action_id.as_deref())
                {
                    let action = action.clone();
                    columns[1].separator();
                    columns[1].label(format!("Payload for {}", action.label));
                    if let Some(payload_hint) = &action.payload_hint {
                        columns[1].small(format!("Hint: {payload_hint}"));
                    } else {
                        columns[1].small("Hint: default empty JSON object");
                    }
                    columns[1].add(
                        egui::TextEdit::multiline(&mut self.payload_input)
                            .desired_rows(8)
                            .desired_width(f32::INFINITY),
                    );
                    columns[1].horizontal(|ui| {
                        if ui.button("Apply template").clicked() {
                            self.payload_input = default_payload_text(&action);
                        }
                        if ui.button("Clear").clicked() {
                            self.payload_input = "{}".to_owned();
                        }
                        if ui.button("Run action").clicked() {
                            self.invoke_selected();
                        }
                    });
                }

                columns[1].separator();
                columns[1].label("Capabilities");
                if manifest.capabilities.is_empty() {
                    columns[1].small("none");
                } else {
                    for capability in &manifest.capabilities {
                        columns[1]
                            .small(format!("• {} — {}", capability.key, capability.description));
                    }
                }
                if !manifest.notes.is_empty() {
                    columns[1].separator();
                    columns[1].label("Notes");
                    for note in &manifest.notes {
                        columns[1].small(format!("• {note}"));
                    }
                }
            } else {
                columns[1].label("No plugin selected");
            }
        });

        ui.separator();
        ui.heading("Invocation output");
        ui.add(
            egui::TextEdit::multiline(&mut self.output)
                .desired_rows(16)
                .desired_width(f32::INFINITY),
        );
    }
}

impl EguiHostApp {
    fn select_plugin(&mut self, plugin_id: String) {
        self.selected_plugin_id = Some(plugin_id.clone());
        self.selected_action_id = selected_manifest(&self.manifests, Some(plugin_id.as_str()))
            .and_then(|manifest| manifest.actions.first())
            .map(|action| action.id.clone());
        self.sync_payload_from_selection();
        self.status = "Updated inspector selection".to_owned();
    }

    fn select_action(&mut self, action_id: String) {
        self.selected_action_id = Some(action_id);
        self.sync_payload_from_selection();
        self.status = "Loaded action template".to_owned();
    }

    fn sync_payload_from_selection(&mut self) {
        if let Some(action) = self
            .selected_plugin()
            .and_then(|manifest| selected_action(manifest, self.selected_action_id.as_deref()))
        {
            self.payload_input = default_payload_text(action);
        }
    }

    fn selected_plugin(&self) -> Option<&PluginManifest> {
        selected_manifest(&self.manifests, self.selected_plugin_id.as_deref())
    }

    fn invoke_selected(&mut self) {
        let Some(playground) = &self.playground else {
            return;
        };
        let Some((plugin_id, plugin_name)) = self
            .selected_plugin()
            .map(|manifest| (manifest.id.clone(), manifest.name.clone()))
        else {
            self.output = "No plugin selected".to_owned();
            return;
        };
        let Some((action_id, action_label)) = self.selected_plugin().and_then(|manifest| {
            selected_action(manifest, self.selected_action_id.as_deref())
                .map(|action| (action.id.clone(), action.label.clone()))
        }) else {
            self.output = "No action selected".to_owned();
            return;
        };
        match playground.invoke_text(&plugin_id, &action_id, &self.payload_input, HostKind::Egui) {
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

fn metric_card(ui: &mut egui::Ui, label: &str, value: String) {
    ui.group(|ui| {
        ui.label(label);
        ui.heading(value);
    });
}

fn selected_manifest<'a>(
    manifests: &'a [PluginManifest],
    plugin_id: Option<&str>,
) -> Option<&'a PluginManifest> {
    let selected = plugin_id?;
    manifests.iter().find(|manifest| manifest.id == selected)
}

fn selected_action<'a>(
    manifest: &'a PluginManifest,
    action_id: Option<&str>,
) -> Option<&'a PluginAction> {
    let selected = action_id?;
    manifest.actions.iter().find(|action| action.id == selected)
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
