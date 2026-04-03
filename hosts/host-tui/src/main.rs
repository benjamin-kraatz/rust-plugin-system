use std::io;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use host_core::{Playground, default_payload_text, render_response, supports_host};
use plugin_manifest::{PluginAction, PluginArchitecture, PluginManifest, SkillLevel};
use plugin_protocol::HostKind;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::*;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

fn main() -> Result<()> {
    let playground = Playground::load_default()?;
    let manifests = playground.manifests();
    let mut app = TuiApp::new(playground, manifests);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut TuiApp) -> Result<()> {
    loop {
        terminal.draw(|frame| render_ui(frame, app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Tab => app.cycle_focus(),
                KeyCode::BackTab => app.reverse_focus(),
                KeyCode::Up => app.move_up(),
                KeyCode::Down => app.move_down(),
                KeyCode::Enter => app.enter_key(),
                KeyCode::Backspace => app.backspace_payload(),
                KeyCode::Char('r') => app.invoke_selected(),
                KeyCode::Char('t') => app.apply_template(),
                KeyCode::Char('c') => app.clear_payload(),
                KeyCode::Char(ch) => app.push_payload_char(ch),
                _ => {}
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusPane {
    Plugins,
    Actions,
    Payload,
}

struct TuiApp {
    playground: Playground,
    manifests: Vec<PluginManifest>,
    plugin_index: usize,
    action_index: usize,
    focus: FocusPane,
    payload_input: String,
    output: String,
    status: String,
}

impl TuiApp {
    fn new(playground: Playground, manifests: Vec<PluginManifest>) -> Self {
        let payload_input = manifests
            .first()
            .and_then(|manifest| manifest.actions.first())
            .map(default_payload_text)
            .unwrap_or_else(|| "{}".to_owned());

        Self {
            playground,
            manifests,
            plugin_index: 0,
            action_index: 0,
            focus: FocusPane::Plugins,
            payload_input,
            output: "Keyboard-first host ready. Tab between panes, Enter to select, r to run, t to load a template, c to clear, q to quit.".to_owned(),
            status: "Loaded runtime plugin catalog".to_owned(),
        }
    }

    fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            FocusPane::Plugins => FocusPane::Actions,
            FocusPane::Actions => FocusPane::Payload,
            FocusPane::Payload => FocusPane::Plugins,
        };
    }

    fn reverse_focus(&mut self) {
        self.focus = match self.focus {
            FocusPane::Plugins => FocusPane::Payload,
            FocusPane::Actions => FocusPane::Plugins,
            FocusPane::Payload => FocusPane::Actions,
        };
    }

    fn move_up(&mut self) {
        match self.focus {
            FocusPane::Plugins => {
                if self.plugin_index > 0 {
                    self.plugin_index -= 1;
                    self.action_index = 0;
                    self.sync_payload();
                    self.status = "Selected plugin".to_owned();
                }
            }
            FocusPane::Actions => {
                if self.action_index > 0 {
                    self.action_index -= 1;
                    self.sync_payload();
                    self.status = "Selected action".to_owned();
                }
            }
            FocusPane::Payload => {}
        }
    }

    fn move_down(&mut self) {
        match self.focus {
            FocusPane::Plugins => {
                if self.plugin_index + 1 < self.manifests.len() {
                    self.plugin_index += 1;
                    self.action_index = 0;
                    self.sync_payload();
                    self.status = "Selected plugin".to_owned();
                }
            }
            FocusPane::Actions => {
                if let Some(manifest) = self.selected_manifest()
                    && self.action_index + 1 < manifest.actions.len()
                {
                    self.action_index += 1;
                    self.sync_payload();
                    self.status = "Selected action".to_owned();
                }
            }
            FocusPane::Payload => {}
        }
    }

    fn enter_key(&mut self) {
        match self.focus {
            FocusPane::Plugins => {
                self.action_index = 0;
                self.sync_payload();
                self.status = "Loaded first action for selected plugin".to_owned();
            }
            FocusPane::Actions => {
                self.sync_payload();
                self.status = "Loaded action payload template".to_owned();
            }
            FocusPane::Payload => self.payload_input.push('\n'),
        }
    }

    fn push_payload_char(&mut self, ch: char) {
        if self.focus == FocusPane::Payload {
            self.payload_input.push(ch);
        }
    }

    fn backspace_payload(&mut self) {
        if self.focus == FocusPane::Payload {
            self.payload_input.pop();
        }
    }

    fn clear_payload(&mut self) {
        self.payload_input = "{}".to_owned();
        self.status = "Cleared payload editor".to_owned();
    }

    fn apply_template(&mut self) {
        self.sync_payload();
        self.status = "Applied selected action template".to_owned();
    }

    fn sync_payload(&mut self) {
        if let Some(action) = self.selected_action() {
            self.payload_input = default_payload_text(action);
        }
    }

    fn selected_manifest(&self) -> Option<&PluginManifest> {
        self.manifests.get(self.plugin_index)
    }

    fn selected_action(&self) -> Option<&PluginAction> {
        self.selected_manifest()
            .and_then(|manifest| manifest.actions.get(self.action_index))
    }

    fn invoke_selected(&mut self) {
        let Some(manifest) = self.selected_manifest() else {
            self.output = "No plugin selected".to_owned();
            return;
        };
        let plugin_id = manifest.id.clone();
        let plugin_name = manifest.name.clone();
        let Some(action) = self.selected_action() else {
            self.output = "No action selected".to_owned();
            return;
        };
        let action_id = action.id.clone();
        let action_label = action.label.clone();

        match self.playground.invoke_text(
            &plugin_id,
            &action_id,
            &self.payload_input,
            HostKind::Tui,
        ) {
            Ok(response) => {
                self.output = render_response(&response);
                self.status = format!("Ran {plugin_name} / {action_label}");
            }
            Err(error) => {
                self.output = error.to_string();
                self.status = format!("Invocation failed for {plugin_name} / {action_label}");
            }
        }
    }
}

fn render_ui(frame: &mut Frame, app: &TuiApp) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(50),
        ])
        .split(frame.area());

    let detail_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(14),
            Constraint::Length(10),
            Constraint::Min(10),
        ])
        .split(layout[2]);

    let plugin_items = app
        .manifests
        .iter()
        .enumerate()
        .map(|(index, manifest)| {
            let marker = if index == app.plugin_index { ">" } else { " " };
            let host_fit = if supports_host(manifest, HostKind::Tui) {
                "tui-ready"
            } else {
                "cross-host"
            };
            ListItem::new(format!(
                "{marker} {} [{}] {}",
                manifest.name,
                manifest.actions.len(),
                host_fit
            ))
        })
        .collect::<Vec<_>>();

    let action_items = app
        .selected_manifest()
        .map(|manifest| {
            manifest
                .actions
                .iter()
                .enumerate()
                .map(|(index, action)| {
                    let marker = if index == app.action_index { ">" } else { " " };
                    ListItem::new(format!("{marker} {} :: {}", action.label, action.id))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let plugin_block = List::new(plugin_items).block(panel_block(
        "Plugins",
        app.focus == FocusPane::Plugins,
        "Up/Down choose plugin",
    ));
    let action_block = List::new(action_items).block(panel_block(
        "Actions",
        app.focus == FocusPane::Actions,
        "Enter loads template",
    ));

    let details = app
        .selected_manifest()
        .map(|manifest| {
            let action_hint = app.selected_action().map(|action| {
                format!(
                    "Selected action: {}\nPayload hint:\n{}",
                    action.label,
                    default_payload_text(action)
                )
            });
            format!(
                "{}\n{}\n\nArchitecture: {}\nSkill: {}\nSupported hosts: {}\nTags: {}\nCapabilities:\n{}\nNotes:\n{}\n\n{}\nStatus: {}",
                manifest.name,
                manifest.description,
                format_architecture(manifest),
                format_skill(manifest),
                manifest
                    .supported_hosts
                    .iter()
                    .map(|host| host.label())
                    .collect::<Vec<_>>()
                    .join(", "),
                if manifest.tags.is_empty() {
                    "none".to_owned()
                } else {
                    manifest.tags.join(", ")
                },
                if manifest.capabilities.is_empty() {
                    "- none".to_owned()
                } else {
                    manifest
                        .capabilities
                        .iter()
                        .map(|capability| format!("- {} :: {}", capability.key, capability.description))
                        .collect::<Vec<_>>()
                        .join("\n")
                },
                if manifest.notes.is_empty() {
                    "- none".to_owned()
                } else {
                    manifest
                        .notes
                        .iter()
                        .map(|note| format!("- {note}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                },
                action_hint.unwrap_or_else(|| "No action selected".to_owned()),
                app.status
            )
        })
        .unwrap_or_else(|| "No plugin selected".to_owned());

    let payload = Paragraph::new(app.payload_input.as_str())
        .block(panel_block(
            "Payload editor",
            app.focus == FocusPane::Payload,
            "Type JSON · r run · t template · c clear",
        ))
        .wrap(Wrap { trim: false });
    let details = Paragraph::new(details)
        .block(panel_block("Manifest details", false, "Tab cycles panes"))
        .wrap(Wrap { trim: false });
    let output = Paragraph::new(app.output.as_str())
        .block(panel_block(
            "Result",
            false,
            "Keyboard-first comparison surface",
        ))
        .wrap(Wrap { trim: false });

    frame.render_widget(plugin_block, layout[0]);
    frame.render_widget(action_block, layout[1]);
    frame.render_widget(details, detail_layout[0]);
    frame.render_widget(payload, detail_layout[1]);
    frame.render_widget(output, detail_layout[2]);
}

fn panel_block<'a>(title: &'a str, focused: bool, subtitle: &'a str) -> Block<'a> {
    let style = if focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    Block::default()
        .title(format!("{title} · {subtitle}"))
        .borders(Borders::ALL)
        .border_style(style)
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
