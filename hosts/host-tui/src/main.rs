use std::io;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use host_core::{Playground, render_response};
use plugin_protocol::HostKind;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::{Terminal, prelude::*};

fn main() -> Result<()> {
    let playground = Playground::load_default()?;
    let manifests = playground.manifests();
    let mut selected = 0usize;
    let mut output = String::from("Press Enter to run the first action of the selected plugin.");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(
        &mut terminal,
        &playground,
        &manifests,
        &mut selected,
        &mut output,
    );

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    playground: &Playground,
    manifests: &[plugin_manifest::PluginManifest],
    selected: &mut usize,
    output: &mut String,
) -> Result<()> {
    loop {
        terminal.draw(|frame| render_ui(frame, manifests, *selected, output))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Up => {
                    if *selected > 0 {
                        *selected -= 1;
                    }
                }
                KeyCode::Down => {
                    if *selected + 1 < manifests.len() {
                        *selected += 1;
                    }
                }
                KeyCode::Enter => {
                    if let Some(manifest) = manifests.get(*selected)
                        && let Some(action) = manifest.actions.first()
                    {
                        match playground.invoke_text(
                            &manifest.id,
                            &action.id,
                            "{}",
                            HostKind::Tui,
                        ) {
                            Ok(response) => *output = render_response(&response),
                            Err(error) => *output = error.to_string(),
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn render_ui(
    frame: &mut Frame,
    manifests: &[plugin_manifest::PluginManifest],
    selected: usize,
    output: &str,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(frame.area());

    let items = manifests
        .iter()
        .enumerate()
        .map(|(index, manifest)| {
            let prefix = if index == selected { "> " } else { "  " };
            ListItem::new(format!(
                "{prefix}{} ({})",
                manifest.name,
                manifest.actions.len()
            ))
        })
        .collect::<Vec<_>>();

    let list = List::new(items).block(
        Block::default()
            .title("Runtime-loaded plugins")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    let details = manifests
        .get(selected)
        .map(|manifest| {
            let actions = manifest
                .actions
                .iter()
                .map(|action| format!("- {} :: {}", action.id, action.description))
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "{}\n\n{}\n\nActions\n{}\n\n{}",
                manifest.name, manifest.description, actions, output
            )
        })
        .unwrap_or_else(|| output.to_owned());

    let paragraph = Paragraph::new(details)
        .block(
            Block::default()
                .title("Details / Output")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(list, chunks[0]);
    frame.render_widget(paragraph, chunks[1]);
}
