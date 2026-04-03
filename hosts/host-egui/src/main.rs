use eframe::egui;
use host_core::{Playground, render_response};
use plugin_manifest::PluginManifest;
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
    error: Option<String>,
    selected_plugin_id: Option<String>,
    output: String,
}

impl Default for EguiHostApp {
    fn default() -> Self {
        match Playground::load_default() {
            Ok(playground) => {
                let selected_plugin_id = playground
                    .manifests()
                    .first()
                    .map(|manifest| manifest.id.clone());
                Self {
                    playground: Some(playground),
                    error: None,
                    selected_plugin_id,
                    output: "Select a plugin action to invoke it.".to_owned(),
                }
            }
            Err(error) => Self {
                playground: None,
                error: Some(error.to_string()),
                selected_plugin_id: None,
                output: String::new(),
            },
        }
    }
}

impl eframe::App for EguiHostApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("egui Host");

        if let Some(error) = &self.error {
            ui.colored_label(egui::Color32::RED, error);
            return;
        }

        ui.columns(2, |columns| {
            columns[0].heading("Plugins");
            if let Some(playground) = &self.playground {
                for manifest in playground.manifests() {
                    let selected = self.selected_plugin_id.as_deref() == Some(manifest.id.as_str());
                    if columns[0]
                        .selectable_label(selected, manifest.name.clone())
                        .clicked()
                    {
                        self.selected_plugin_id = Some(manifest.id);
                    }
                }

                columns[1].heading("Details");
                if !playground.warnings().is_empty() {
                    columns[1].collapsing("Discovery warnings", |ui| {
                        for warning in playground.warnings() {
                            ui.label(warning);
                        }
                    });
                }

                if let Some(manifest) =
                    selected_manifest(playground, self.selected_plugin_id.as_deref())
                {
                    columns[1].label(&manifest.description);
                    columns[1].separator();
                    columns[1].label("Actions");

                    for action in &manifest.actions {
                        if columns[1].button(format!("Run {}", action.label)).clicked() {
                            match playground.invoke_text(
                                &manifest.id,
                                &action.id,
                                "{}",
                                HostKind::Egui,
                            ) {
                                Ok(response) => self.output = render_response(&response),
                                Err(error) => self.output = error.to_string(),
                            }
                        }
                    }

                    columns[1].separator();
                    columns[1].label("Output");
                    columns[1].add(
                        egui::TextEdit::multiline(&mut self.output)
                            .desired_rows(20)
                            .desired_width(f32::INFINITY),
                    );
                }
            }
        });
    }
}

fn selected_manifest<'a>(
    playground: &'a Playground,
    plugin_id: Option<&str>,
) -> Option<PluginManifest> {
    let selected = plugin_id?;
    playground
        .manifests()
        .into_iter()
        .find(|manifest| manifest.id == selected)
}
