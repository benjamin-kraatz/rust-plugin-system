use std::sync::OnceLock;

use dioxus::prelude::*;
use host_core::Playground;

static APP_DATA: OnceLock<Result<Vec<PluginCard>, String>> = OnceLock::new();

#[derive(Clone)]
struct PluginCard {
    name: String,
    description: String,
    action_count: usize,
}

fn main() {
    let _ = APP_DATA.set(
        Playground::load_default()
            .map(|playground| {
                playground
                    .manifests()
                    .into_iter()
                    .map(|manifest| PluginCard {
                        name: manifest.name,
                        description: manifest.description,
                        action_count: manifest.actions.len(),
                    })
                    .collect()
            })
            .map_err(|error| error.to_string()),
    );

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let data = APP_DATA
        .get()
        .expect("dioxus app data should be initialized");

    rsx! {
        div {
            style: "font-family: sans-serif; padding: 24px;",
            h1 { "Dioxus Desktop Host" }
            p { "This host loads plugin metadata at runtime and shows how a component-oriented UI can surface the catalog." }
            match data {
                Ok(cards) => rsx! {
                    div {
                        for card in cards.iter() {
                            article {
                                style: "border: 1px solid #ddd; border-radius: 8px; padding: 12px; margin-bottom: 12px;",
                                h2 { "{card.name}" }
                                p { "{card.description}" }
                                p { "Actions: {card.action_count}" }
                            }
                        }
                    }
                },
                Err(error) => rsx! {
                    p { style: "color: red;", "{error}" }
                },
            }
        }
    }
}
