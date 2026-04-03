use eframe::egui;
use host_core::{Playground, default_payload_text, render_response, supports_host};
use plugin_manifest::{PluginAction, PluginArchitecture, PluginManifest, SkillLevel};
use plugin_protocol::HostKind;

// Color palette — dark navy/blue matching the web host
const BG_DARK: egui::Color32 = egui::Color32::from_rgb(0x0b, 0x10, 0x20);
const PANEL_BG: egui::Color32 = egui::Color32::from_rgb(0x12, 0x19, 0x33);
const BORDER: egui::Color32 = egui::Color32::from_rgb(0x2a, 0x37, 0x6d);
const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(0xed, 0xf2, 0xff);
const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(0x9f, 0xb1, 0xe2);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(0x70, 0xa5, 0xff);
const ACCENT_MINT: egui::Color32 = egui::Color32::from_rgb(0x87, 0xf0, 0xd4);
const HOVER_BG: egui::Color32 = egui::Color32::from_rgb(0x1a, 0x25, 0x45);
const ACTIVE_BG: egui::Color32 = egui::Color32::from_rgb(0x20, 0x2d, 0x55);

const ARCH_NATIVE_COLOR: egui::Color32 = egui::Color32::from_rgb(0x66, 0xd9, 0x9e);
const ARCH_ABI_COLOR: egui::Color32 = egui::Color32::from_rgb(0x70, 0xa5, 0xff);
const ARCH_WASM_COLOR: egui::Color32 = egui::Color32::from_rgb(0xff, 0xb0, 0x66);

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
    invocation_count: u32,
    visuals_applied: bool,
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
                    invocation_count: 0,
                    visuals_applied: false,
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
                invocation_count: 0,
                visuals_applied: false,
            },
        }
    }
}

fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG_DARK;
    visuals.window_fill = PANEL_BG;
    visuals.faint_bg_color = PANEL_BG;

    visuals.widgets.noninteractive.bg_fill = PANEL_BG;
    visuals.widgets.noninteractive.weak_bg_fill = PANEL_BG;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT_MUTED);
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, BORDER);

    visuals.widgets.inactive.bg_fill = PANEL_BG;
    visuals.widgets.inactive.weak_bg_fill = PANEL_BG;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(0.5, BORDER);

    visuals.widgets.hovered.bg_fill = HOVER_BG;
    visuals.widgets.hovered.weak_bg_fill = HOVER_BG;
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, ACCENT);

    visuals.widgets.active.bg_fill = ACTIVE_BG;
    visuals.widgets.active.weak_bg_fill = ACTIVE_BG;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, ACCENT_MINT);

    visuals.selection.bg_fill = ACCENT_MINT.gamma_multiply(0.2);
    visuals.selection.stroke = egui::Stroke::new(1.5, ACCENT_MINT);

    let rounding = egui::CornerRadius::same(8);
    for ws in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
    ] {
        ws.corner_radius = rounding;
    }
    visuals.window_corner_radius = rounding;

    ctx.set_visuals(visuals);
}

impl eframe::App for EguiHostApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if !self.visuals_applied {
            apply_theme(ui.ctx());
            self.visuals_applied = true;
        }

        if let Some(error) = &self.error {
            ui.colored_label(egui::Color32::RED, error);
            return;
        }

        let total_actions: usize = self.manifests.iter().map(|m| m.actions.len()).sum();

        // Bottom status bar
        egui::Panel::bottom("status_bar")
            .frame(
                egui::Frame::new()
                    .fill(PANEL_BG)
                    .inner_margin(egui::Margin::symmetric(12, 6))
                    .stroke(egui::Stroke::new(1.0, BORDER)),
            )
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(&self.status)
                            .color(TEXT_MUTED)
                            .small(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} plugins · {} actions · {} runs",
                                self.manifests.len(),
                                total_actions,
                                self.invocation_count,
                            ))
                            .color(TEXT_MUTED)
                            .small(),
                        );
                    });
                });
            });

        // Left sidebar: plugin catalog
        let manifests = self.manifests.clone();
        egui::Panel::left("catalog")
            .default_size(280.0)
            .frame(
                egui::Frame::new()
                    .fill(PANEL_BG)
                    .inner_margin(egui::Margin::same(12))
                    .stroke(egui::Stroke::new(1.0, BORDER)),
            )
            .show_inside(ui, |ui| {
                ui.label(
                    egui::RichText::new("Plugin Catalog")
                        .color(TEXT_PRIMARY)
                        .strong()
                        .size(16.0),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Select a plugin to inspect")
                        .color(TEXT_MUTED)
                        .small(),
                );
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for manifest in &manifests {
                        let selected =
                            self.selected_plugin_id.as_deref() == Some(manifest.id.as_str());

                        let card_frame = if selected {
                            egui::Frame::new()
                                .fill(ACTIVE_BG)
                                .corner_radius(egui::CornerRadius::same(8))
                                .stroke(egui::Stroke::new(1.5, ACCENT_MINT))
                                .inner_margin(egui::Margin::same(10))
                        } else {
                            egui::Frame::new()
                                .fill(BG_DARK)
                                .corner_radius(egui::CornerRadius::same(8))
                                .stroke(egui::Stroke::new(0.5, BORDER))
                                .inner_margin(egui::Margin::same(10))
                        };

                        let response = card_frame
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(&manifest.name)
                                        .color(TEXT_PRIMARY)
                                        .strong(),
                                );
                                ui.label(
                                    egui::RichText::new(&manifest.description)
                                        .color(TEXT_MUTED)
                                        .small(),
                                );
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    architecture_badge(ui, manifest);
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{} action{}",
                                            manifest.actions.len(),
                                            if manifest.actions.len() == 1 { "" } else { "s" }
                                        ))
                                        .color(TEXT_MUTED)
                                        .small(),
                                    );
                                });
                            })
                            .response;

                        if response.interact(egui::Sense::click()).clicked() {
                            self.select_plugin(manifest.id.clone());
                        }
                        ui.add_space(4.0);
                    }
                });
            });

        // Central panel: main content
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(BG_DARK)
                    .inner_margin(egui::Margin::same(16)),
            )
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let selected_manifest = self.selected_plugin().cloned();

                    if let Some(manifest) = selected_manifest {
                        // Header: plugin name + version
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&manifest.name)
                                    .color(TEXT_PRIMARY)
                                    .strong()
                                    .size(22.0),
                            );
                            ui.add_space(8.0);
                            version_badge(ui, &manifest.version);
                        });
                        ui.add_space(2.0);
                        ui.label(egui::RichText::new(&manifest.description).color(TEXT_MUTED));
                        ui.add_space(12.0);

                        // Discovery warnings
                        if let Some(playground) = &self.playground {
                            let warnings = playground.warnings();
                            if !warnings.is_empty() {
                                egui::CollapsingHeader::new(
                                    egui::RichText::new(format!(
                                        "⚠ Discovery Warnings ({})",
                                        warnings.len()
                                    ))
                                    .color(ARCH_WASM_COLOR),
                                )
                                .show(ui, |ui| {
                                    for warning in warnings {
                                        ui.label(
                                            egui::RichText::new(format!("  • {warning}"))
                                                .color(ARCH_WASM_COLOR)
                                                .small(),
                                        );
                                    }
                                });
                                ui.add_space(8.0);
                            }
                        }

                        // Manifest Details
                        egui::CollapsingHeader::new(
                            egui::RichText::new("Manifest Details")
                                .color(TEXT_PRIMARY)
                                .strong(),
                        )
                        .show(ui, |ui| {
                            section_frame().show(ui, |ui| {
                                egui::Grid::new("manifest_metadata")
                                    .num_columns(2)
                                    .spacing([20.0, 6.0])
                                    .show(ui, |ui| {
                                        grid_label(ui, "Version");
                                        ui.monospace(
                                            egui::RichText::new(&manifest.version).color(ACCENT),
                                        );
                                        ui.end_row();

                                        grid_label(ui, "Architecture");
                                        architecture_badge(ui, &manifest);
                                        ui.end_row();

                                        grid_label(ui, "Skill level");
                                        ui.label(
                                            egui::RichText::new(format_skill(&manifest))
                                                .color(TEXT_PRIMARY),
                                        );
                                        ui.end_row();

                                        grid_label(ui, "Hosts");
                                        ui.label(
                                            egui::RichText::new(supported_hosts_text(&manifest))
                                                .color(TEXT_PRIMARY),
                                        );
                                        ui.end_row();

                                        grid_label(ui, "Tags");
                                        ui.label(
                                            egui::RichText::new(tags_text(&manifest))
                                                .color(TEXT_PRIMARY),
                                        );
                                        ui.end_row();

                                        grid_label(ui, "egui support");
                                        let egui_ok = supports_host(&manifest, HostKind::Egui);
                                        ui.label(
                                            egui::RichText::new(if egui_ok {
                                                "✓ yes"
                                            } else {
                                                "✗ no"
                                            })
                                            .color(if egui_ok {
                                                ARCH_NATIVE_COLOR
                                            } else {
                                                TEXT_MUTED
                                            }),
                                        );
                                        ui.end_row();
                                    });
                            });
                        });
                        ui.add_space(8.0);

                        // Actions
                        egui::CollapsingHeader::new(
                            egui::RichText::new(format!(
                                "Actions ({})",
                                manifest.actions.len()
                            ))
                            .color(TEXT_PRIMARY)
                            .strong(),
                        )
                        .default_open(true)
                        .show(ui, |ui| {
                            section_frame().show(ui, |ui| {
                                for action in &manifest.actions {
                                    let selected = self.selected_action_id.as_deref()
                                        == Some(action.id.as_str());
                                    let resp = ui.selectable_label(
                                        selected,
                                        egui::RichText::new(format!(
                                            "{}  —  {}",
                                            action.label, action.description
                                        ))
                                        .color(if selected { ACCENT_MINT } else { TEXT_PRIMARY }),
                                    );
                                    if resp.clicked() {
                                        self.select_action(action.id.clone());
                                    }
                                }
                            });
                        });
                        ui.add_space(8.0);

                        // Payload Editor
                        if let Some(action) =
                            selected_action(&manifest, self.selected_action_id.as_deref())
                        {
                            let action = action.clone();
                            egui::CollapsingHeader::new(
                                egui::RichText::new(format!(
                                    "Payload Editor — {}",
                                    action.label
                                ))
                                .color(TEXT_PRIMARY)
                                .strong(),
                            )
                            .default_open(true)
                            .show(ui, |ui| {
                                section_frame().show(ui, |ui| {
                                    let hint = action
                                        .payload_hint
                                        .as_deref()
                                        .unwrap_or("default empty JSON object");
                                    ui.label(
                                        egui::RichText::new(format!("Hint: {hint}"))
                                            .color(TEXT_MUTED)
                                            .small()
                                            .italics(),
                                    );
                                    ui.add_space(6.0);
                                    ui.add(
                                        egui::TextEdit::multiline(&mut self.payload_input)
                                            .font(egui::TextStyle::Monospace)
                                            .desired_rows(8)
                                            .desired_width(f32::INFINITY),
                                    );
                                    ui.add_space(6.0);
                                    ui.horizontal(|ui| {
                                        if ui
                                            .button(
                                                egui::RichText::new("Apply Template")
                                                    .color(TEXT_PRIMARY),
                                            )
                                            .clicked()
                                        {
                                            self.payload_input = default_payload_text(&action);
                                        }
                                        if ui
                                            .button(
                                                egui::RichText::new("Clear").color(TEXT_PRIMARY),
                                            )
                                            .clicked()
                                        {
                                            self.payload_input = "{}".to_owned();
                                        }
                                        let run_button = egui::Button::new(
                                            egui::RichText::new("▶ Run").color(BG_DARK).strong(),
                                        )
                                        .fill(ACCENT);
                                        if ui.add(run_button).clicked() {
                                            self.invoke_selected();
                                        }
                                    });
                                });
                            });
                            ui.add_space(8.0);
                        }

                        // Capabilities & Notes
                        if !manifest.capabilities.is_empty() || !manifest.notes.is_empty() {
                            egui::CollapsingHeader::new(
                                egui::RichText::new("Capabilities & Notes")
                                    .color(TEXT_PRIMARY)
                                    .strong(),
                            )
                            .show(ui, |ui| {
                                section_frame().show(ui, |ui| {
                                    if !manifest.capabilities.is_empty() {
                                        ui.label(
                                            egui::RichText::new("Capabilities")
                                                .color(ACCENT)
                                                .strong()
                                                .small(),
                                        );
                                        ui.add_space(4.0);
                                        for cap in &manifest.capabilities {
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    egui::RichText::new("•").color(ACCENT_MINT),
                                                );
                                                ui.label(
                                                    egui::RichText::new(&cap.key)
                                                        .color(TEXT_PRIMARY)
                                                        .strong(),
                                                );
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "— {}",
                                                        cap.description
                                                    ))
                                                    .color(TEXT_MUTED),
                                                );
                                            });
                                        }
                                    }
                                    if !manifest.notes.is_empty() {
                                        ui.add_space(8.0);
                                        ui.label(
                                            egui::RichText::new("Notes")
                                                .color(ACCENT)
                                                .strong()
                                                .small(),
                                        );
                                        ui.add_space(4.0);
                                        for note in &manifest.notes {
                                            ui.label(
                                                egui::RichText::new(format!("  • {note}"))
                                                    .color(TEXT_MUTED),
                                            );
                                        }
                                    }
                                });
                            });
                            ui.add_space(8.0);
                        }

                        // Output
                        egui::CollapsingHeader::new(
                            egui::RichText::new("Output").color(TEXT_PRIMARY).strong(),
                        )
                        .default_open(true)
                        .show(ui, |ui| {
                            egui::Frame::new()
                                .fill(egui::Color32::from_rgb(0x08, 0x0c, 0x18))
                                .corner_radius(egui::CornerRadius::same(6))
                                .stroke(egui::Stroke::new(1.0, BORDER))
                                .inner_margin(egui::Margin::same(10))
                                .show(ui, |ui| {
                                    ui.add(
                                        egui::TextEdit::multiline(&mut self.output)
                                            .font(egui::TextStyle::Monospace)
                                            .desired_rows(10)
                                            .desired_width(f32::INFINITY),
                                    );
                                });
                        });
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label(
                                egui::RichText::new("Select a plugin from the catalog")
                                    .color(TEXT_MUTED)
                                    .size(18.0),
                            );
                        });
                    }
                });
            });
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
        self.invocation_count += 1;
    }
}

// --- Helper widgets ---

fn section_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(PANEL_BG)
        .corner_radius(egui::CornerRadius::same(6))
        .stroke(egui::Stroke::new(0.5, BORDER))
        .inner_margin(egui::Margin::same(10))
}

fn grid_label(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).color(TEXT_MUTED).small());
}

fn architecture_badge(ui: &mut egui::Ui, manifest: &PluginManifest) {
    let (label, color) = match manifest.architecture {
        PluginArchitecture::NativeJson => ("Native", ARCH_NATIVE_COLOR),
        PluginArchitecture::AbiStable => ("ABI", ARCH_ABI_COLOR),
        PluginArchitecture::Wasm => ("WASM", ARCH_WASM_COLOR),
    };
    ui.label(egui::RichText::new(label).color(color).strong().small());
}

fn version_badge(ui: &mut egui::Ui, version: &str) {
    egui::Frame::new()
        .fill(ACCENT.gamma_multiply(0.15))
        .corner_radius(egui::CornerRadius::same(4))
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(format!("v{version}"))
                    .color(ACCENT)
                    .small(),
            );
        });
}

// --- Data helpers ---

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
