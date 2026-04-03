use host_core::{Playground, default_payload_text, render_response, supports_host};
use iced::widget::{button, column, container, row, rule, scrollable, text, text_editor};
use iced::{Background, Border, Color, Element, Length, Shadow, Theme, theme::Palette};
use plugin_manifest::{PluginAction, PluginArchitecture, PluginManifest, SkillLevel};
use plugin_protocol::HostKind;

// -- Palette constants ----------------------------------------------------------

const BG: Color = Color::from_rgb(0.043, 0.063, 0.125); // #0b1020
const PANEL: Color = Color::from_rgb(0.071, 0.098, 0.200); // #121933
const BORDER: Color = Color::from_rgb(0.165, 0.216, 0.427); // #2a376d
const TEXT: Color = Color::from_rgb(0.929, 0.949, 1.0); // #edf2ff
const MUTED: Color = Color::from_rgb(0.624, 0.694, 0.886); // #9fb1e2
const ACCENT: Color = Color::from_rgb(0.439, 0.647, 1.0); // #70a5ff
const ACCENT2: Color = Color::from_rgb(0.529, 0.941, 0.831); // #87f0d4
const SURFACE: Color = Color::from_rgb(0.09, 0.12, 0.24); // slightly lighter panel
const SELECTED_BG: Color = Color::from_rgb(0.11, 0.16, 0.35); // selected highlight

fn app_theme() -> Theme {
    Theme::custom(
        "Navy Dark",
        Palette {
            background: BG,
            text: TEXT,
            primary: ACCENT,
            success: ACCENT2,
            warning: Color::from_rgb(1.0, 0.76, 0.28),
            danger: Color::from_rgb(1.0, 0.33, 0.38),
        },
    )
}

// -- Application ----------------------------------------------------------------

pub fn main() -> iced::Result {
    iced::application(IcedHostApp::default, update, view)
        .theme(|_: &IcedHostApp| app_theme())
        .run()
}

struct IcedHostApp {
    playground: Option<Playground>,
    manifests: Vec<PluginManifest>,
    error: Option<String>,
    selected_plugin_id: Option<String>,
    selected_action_id: Option<String>,
    payload_content: text_editor::Content,
    output: String,
    status: String,
}

#[derive(Debug, Clone)]
enum Message {
    SelectPlugin(String),
    SelectAction(String),
    PayloadEditorAction(text_editor::Action),
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
                let payload_text = manifests
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
                    payload_content: text_editor::Content::with_text(&payload_text),
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
                payload_content: text_editor::Content::with_text("{}"),
                output: String::new(),
                status: String::new(),
            },
        }
    }
}

// -- Update ---------------------------------------------------------------------

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
        Message::PayloadEditorAction(action) => {
            state.payload_content.perform(action);
        }
        Message::ApplyTemplate => {
            state.sync_payload();
            state.status = "Applied payload template".to_owned();
        }
        Message::ClearPayload => {
            state.payload_content = text_editor::Content::with_text("{}");
            state.status = "Cleared payload".to_owned();
        }
        Message::InvokeSelected => {
            state.invoke_selected();
        }
    }
}

// -- View -----------------------------------------------------------------------

fn view(state: &IcedHostApp) -> Element<'_, Message> {
    if let Some(error) = &state.error {
        return container(
            column![
                section_header("Error"),
                text(error).color(Color::from_rgb(1.0, 0.33, 0.38)),
            ]
            .spacing(8),
        )
        .padding(20)
        .style(|_| panel_style())
        .into();
    }

    let Some(playground) = &state.playground else {
        return styled_panel(text("No playground loaded").color(MUTED)).into();
    };

    // -- Metric cards -----------------------------------------------------------
    let metrics = row![
        metric_card("Plugins", state.manifests.len()),
        metric_card(
            "Iced-ready",
            state
                .manifests
                .iter()
                .filter(|m| supports_host(m, HostKind::Iced))
                .count(),
        ),
        metric_card("Warnings", playground.warnings().len()),
    ]
    .spacing(12);

    // -- Sidebar: plugin catalogue ----------------------------------------------
    let sidebar_list = state
        .manifests
        .iter()
        .fold(column![].spacing(6), |col, manifest| {
            let is_selected = state.selected_plugin_id.as_deref() == Some(manifest.id.as_str());
            col.push(plugin_card(manifest, is_selected))
        });

    let sidebar = column![
        section_header("Plugin Catalog"),
        rule::horizontal(1),
        scrollable(sidebar_list).height(Length::Fill),
    ]
    .spacing(12);

    let sidebar_panel = container(sidebar)
        .padding(16)
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .style(|_| panel_style());

    // -- Main area: details + payload + output ----------------------------------
    let main_content = if let Some(manifest) = state.selected_manifest() {
        build_details(state, manifest)
    } else {
        column![text("Select a plugin from the catalog.").color(MUTED)]
    };

    let main_panel = container(scrollable(main_content.spacing(16)))
        .padding(16)
        .width(Length::FillPortion(3))
        .height(Length::Fill)
        .style(|_| panel_style());

    // -- Root layout ------------------------------------------------------------
    column![
        container(metrics).padding([12, 16]),
        row![sidebar_panel, main_panel]
            .spacing(12)
            .height(Length::Fill),
        container(
            text(format!("Status: {}", state.status))
                .size(13)
                .color(MUTED)
        )
        .padding([6, 16]),
    ]
    .spacing(0)
    .into()
}

// -- Detail panel builder -------------------------------------------------------

fn build_details<'a>(
    state: &'a IcedHostApp,
    manifest: &'a PluginManifest,
) -> iced::widget::Column<'a, Message> {
    // Metadata header
    let metadata = column![
        text(&manifest.name).size(26).color(TEXT),
        text(&manifest.description).size(14).color(MUTED),
        text(format!(
            "v{}  ·  {}  ·  {}",
            manifest.version,
            format_architecture(manifest),
            format_skill(manifest)
        ))
        .size(13)
        .color(MUTED),
        badge_row(manifest),
    ]
    .spacing(4);

    // Actions
    let actions_header = section_header("Actions");
    let action_list = manifest
        .actions
        .iter()
        .fold(column![].spacing(4), |col, action| {
            let is_selected = state.selected_action_id.as_deref() == Some(action.id.as_str());
            col.push(action_button(action, is_selected))
        });

    // Payload editor
    let payload_section = column![
        section_header("Payload Editor"),
        container(
            text_editor(&state.payload_content)
                .on_action(Message::PayloadEditorAction)
                .height(180)
                .style(|theme, status| {
                    let mut base = text_editor::default(theme, status);
                    base.background = Background::Color(SURFACE);
                    base.border = Border {
                        radius: 6.0.into(),
                        width: 1.0,
                        color: BORDER,
                    };
                    base.value = TEXT;
                    base.placeholder = MUTED;
                    base.selection = Color::from_rgba(0.439, 0.647, 1.0, 0.3);
                    base
                }),
        ),
        row![
            secondary_button("Apply template", Message::ApplyTemplate),
            secondary_button("Clear", Message::ClearPayload),
            primary_button("Run action", Message::InvokeSelected),
        ]
        .spacing(8),
    ]
    .spacing(8);

    // Capabilities
    let capabilities = if manifest.capabilities.is_empty() {
        column![text("No declared capabilities.").size(13).color(MUTED)]
    } else {
        manifest
            .capabilities
            .iter()
            .fold(column![].spacing(2), |col, cap| {
                col.push(
                    text(format!("•  {} — {}", cap.key, cap.description))
                        .size(13)
                        .color(MUTED),
                )
            })
    };

    // Output
    let output_section = column![
        section_header("Invocation Output"),
        container(scrollable(text(&state.output).size(13).color(ACCENT2)).height(200))
            .padding(12)
            .width(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(Color::from_rgb(0.04, 0.05, 0.10))),
                border: Border {
                    radius: 6.0.into(),
                    width: 1.0,
                    color: BORDER,
                },
                shadow: Shadow::default(),
                snap: false,
                text_color: Some(ACCENT2),
            }),
    ]
    .spacing(8);

    column![
        metadata,
        rule::horizontal(1),
        actions_header,
        action_list,
        rule::horizontal(1),
        payload_section,
        rule::horizontal(1),
        column![section_header("Capabilities"), capabilities].spacing(6),
        rule::horizontal(1),
        output_section,
    ]
}

// -- Reusable styled widgets ----------------------------------------------------

fn section_header(label: &str) -> Element<'_, Message> {
    text(label).size(20).color(ACCENT).into()
}

fn styled_panel(
    content: impl Into<Element<'static, Message>>,
) -> container::Container<'static, Message> {
    container(content).padding(20).style(|_| panel_style())
}

fn panel_style() -> container::Style {
    container::Style {
        background: Some(Background::Color(PANEL)),
        border: Border {
            radius: 10.0.into(),
            width: 1.0,
            color: BORDER,
        },
        shadow: Shadow::default(),
        snap: false,
        text_color: Some(TEXT),
    }
}

fn plugin_card(manifest: &PluginManifest, selected: bool) -> Element<'_, Message> {
    let host_fit = if supports_host(manifest, HostKind::Iced) {
        "Iced-ready"
    } else {
        "Cross-host"
    };

    let card_bg = if selected { SELECTED_BG } else { SURFACE };
    let border_color = if selected { ACCENT } else { BORDER };

    let content = column![
        text(&manifest.name).size(15).color(TEXT),
        text(&manifest.description).size(12).color(MUTED),
        text(format!(
            "{} actions  ·  {}",
            manifest.actions.len(),
            host_fit
        ))
        .size(11)
        .color(MUTED),
    ]
    .spacing(2);

    button(container(content).padding(4).width(Length::Fill))
        .width(Length::Fill)
        .padding(8)
        .on_press(Message::SelectPlugin(manifest.id.clone()))
        .style(move |_theme, status| {
            let bg = match status {
                button::Status::Hovered => lighten(card_bg, 0.06),
                button::Status::Pressed => lighten(card_bg, 0.10),
                _ => card_bg,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: TEXT,
                border: Border {
                    radius: 8.0.into(),
                    width: if selected { 1.5 } else { 1.0 },
                    color: border_color,
                },
                shadow: Shadow::default(),
                snap: false,
            }
        })
        .into()
}

fn action_button(action: &PluginAction, selected: bool) -> Element<'_, Message> {
    let indicator = if selected { "▶  " } else { "    " };
    let label_color = if selected { ACCENT } else { TEXT };
    let bg = if selected { SELECTED_BG } else { SURFACE };
    let border_color = if selected { ACCENT } else { BORDER };

    let content = column![
        text(format!("{}{}", indicator, action.label))
            .size(14)
            .color(label_color),
        text(format!("    {}", action.description))
            .size(12)
            .color(MUTED),
    ]
    .spacing(1);

    button(content)
        .width(Length::Fill)
        .padding([6, 10])
        .on_press(Message::SelectAction(action.id.clone()))
        .style(move |_theme, status| {
            let bg = match status {
                button::Status::Hovered => lighten(bg, 0.05),
                button::Status::Pressed => lighten(bg, 0.09),
                _ => bg,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: TEXT,
                border: Border {
                    radius: 6.0.into(),
                    width: if selected { 1.0 } else { 0.5 },
                    color: border_color,
                },
                shadow: Shadow::default(),
                snap: false,
            }
        })
        .into()
}

fn primary_button(label: &str, msg: Message) -> Element<'_, Message> {
    button(text(label).size(14).color(Color::from_rgb(0.0, 0.0, 0.05)))
        .padding([6, 16])
        .on_press(msg)
        .style(|_theme, status| {
            let bg = match status {
                button::Status::Hovered => lighten(ACCENT, 0.10),
                button::Status::Pressed => lighten(ACCENT, 0.18),
                _ => ACCENT,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: Color::from_rgb(0.0, 0.0, 0.05),
                border: Border {
                    radius: 6.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: Shadow::default(),
                snap: false,
            }
        })
        .into()
}

fn secondary_button(label: &str, msg: Message) -> Element<'_, Message> {
    button(text(label).size(14).color(MUTED))
        .padding([6, 14])
        .on_press(msg)
        .style(|_theme, status| {
            let bg = match status {
                button::Status::Hovered => lighten(SURFACE, 0.05),
                button::Status::Pressed => lighten(SURFACE, 0.10),
                _ => SURFACE,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: MUTED,
                border: Border {
                    radius: 6.0.into(),
                    width: 1.0,
                    color: BORDER,
                },
                shadow: Shadow::default(),
                snap: false,
            }
        })
        .into()
}

fn metric_card<'a>(label: &'a str, value: usize) -> Element<'a, Message> {
    container(
        column![
            text(label).size(12).color(MUTED),
            text(value.to_string()).size(26).color(ACCENT),
        ]
        .spacing(2),
    )
    .padding([10, 18])
    .style(|_| container::Style {
        background: Some(Background::Color(PANEL)),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: BORDER,
        },
        shadow: Shadow::default(),
        snap: false,
        text_color: None,
    })
    .into()
}

fn badge_row(manifest: &PluginManifest) -> Element<'_, Message> {
    let mut badges = row![].spacing(6);

    for host in &manifest.supported_hosts {
        badges = badges.push(badge(host.label(), ACCENT));
    }
    badges = badges.push(badge(format_architecture(manifest), ACCENT2));
    badges = badges.push(badge(format_skill(manifest), MUTED));

    for tag in &manifest.tags {
        badges = badges.push(badge(tag, MUTED));
    }

    badges.into()
}

fn badge<'a>(label: &'a str, color: Color) -> Element<'a, Message> {
    container(text(label).size(11).color(color))
        .padding([2, 8])
        .style(move |_| container::Style {
            background: Some(Background::Color(Color::from_rgba(
                color.r, color.g, color.b, 0.12,
            ))),
            border: Border {
                radius: 4.0.into(),
                width: 0.5,
                color: Color::from_rgba(color.r, color.g, color.b, 0.4),
            },
            shadow: Shadow::default(),
            snap: false,
            text_color: Some(color),
        })
        .into()
}

// -- Helpers --------------------------------------------------------------------

fn lighten(c: Color, amount: f32) -> Color {
    Color::from_rgb(
        (c.r + amount).min(1.0),
        (c.g + amount).min(1.0),
        (c.b + amount).min(1.0),
    )
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
            self.payload_content = text_editor::Content::with_text(&default_payload_text(action));
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
        let payload_text = self.payload_content.text();
        match playground.invoke_text(&plugin_id, &action_id, &payload_text, HostKind::Iced) {
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
