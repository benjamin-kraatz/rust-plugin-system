use host_core::{Playground, render_response};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length};
use plugin_manifest::PluginManifest;
use plugin_protocol::HostKind;

pub fn main() -> iced::Result {
    iced::application(IcedHostApp::default, update, view).run()
}

struct IcedHostApp {
    playground: Option<Playground>,
    error: Option<String>,
    selected_plugin_id: Option<String>,
    output: String,
}

#[derive(Debug, Clone)]
enum Message {
    SelectPlugin(String),
    Invoke {
        plugin_id: String,
        action_id: String,
    },
}

impl Default for IcedHostApp {
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
                    output: "Select an action to invoke a runtime-loaded plugin.".to_owned(),
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

fn update(state: &mut IcedHostApp, message: Message) {
    match message {
        Message::SelectPlugin(plugin_id) => {
            state.selected_plugin_id = Some(plugin_id);
        }
        Message::Invoke {
            plugin_id,
            action_id,
        } => {
            if let Some(playground) = &state.playground {
                match playground.invoke_text(&plugin_id, &action_id, "{}", HostKind::Iced) {
                    Ok(response) => state.output = render_response(&response),
                    Err(error) => state.output = error.to_string(),
                }
            }
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

    let manifests = playground.manifests();
    let sidebar = manifests
        .iter()
        .fold(column![text("Plugins").size(28)], |column, manifest| {
            column.push(
                button(text(manifest.name.clone()))
                    .width(Length::Fill)
                    .on_press(Message::SelectPlugin(manifest.id.clone())),
            )
        });

    let details = if let Some(manifest) =
        selected_manifest(&manifests, state.selected_plugin_id.as_deref())
    {
        manifest.actions.iter().fold(
            column![
                text(manifest.name.clone()).size(28),
                text(manifest.description.clone()),
                text("Actions").size(22),
            ],
            |column, action| {
                column.push(
                    button(text(action.label.clone())).on_press(Message::Invoke {
                        plugin_id: manifest.id.clone(),
                        action_id: action.id.clone(),
                    }),
                )
            },
        )
    } else {
        column![text("No plugin selected")]
    };

    row![
        container(scrollable(sidebar.spacing(8))).width(Length::FillPortion(1)),
        container(scrollable(
            details
                .spacing(12)
                .push(text("Output").size(22))
                .push(text(state.output.clone()))
        ))
        .width(Length::FillPortion(2)),
    ]
    .spacing(16)
    .padding(16)
    .into()
}

fn selected_manifest<'a>(
    manifests: &'a [PluginManifest],
    plugin_id: Option<&str>,
) -> Option<&'a PluginManifest> {
    let plugin_id = plugin_id?;
    manifests.iter().find(|manifest| manifest.id == plugin_id)
}
