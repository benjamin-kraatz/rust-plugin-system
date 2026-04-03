# GUI Recipes

## Load plugins and invoke an action (all hosts)

```rust
let playground = Playground::load_default()?;
let manifests = playground.manifests();
let payload_text = default_payload_text(&action);
let response = playground.invoke(&plugin_id, &action_id, payload, HostKind::Egui);
let output = render_response(&response);
```

## Check whether a plugin supports the current host

```rust
if supports_host(&manifest, HostKind::Iced) { /* show it */ }
```

---

## egui: Color palette and custom Visuals

```rust
const BG_DARK: egui::Color32 = egui::Color32::from_rgb(0x0b, 0x10, 0x20);
const ACCENT_MINT: egui::Color32 = egui::Color32::from_rgb(0x87, 0xf0, 0xd4);

fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG_DARK;
    visuals.widgets.hovered.bg_fill = HOVER_BG;
    visuals.selection.stroke = egui::Stroke::new(1.5, ACCENT_MINT);
    ctx.set_visuals(visuals);
}
```

## egui: SidePanel + CentralPanel layout

```rust
egui::Panel::left("catalog").default_size(280.0)
    .frame(egui::Frame::new().fill(PANEL_BG))
    .show_inside(ui, |ui| { /* sidebar */ });
egui::CentralPanel::default()
    .show_inside(ui, |ui| { /* main content */ });
```

## egui: CollapsingHeader for metadata grid

```rust
egui::CollapsingHeader::new(egui::RichText::new("Manifest Details").strong())
    .show(ui, |ui| {
        egui::Grid::new("metadata").num_columns(2).show(ui, |ui| {
            ui.label(egui::RichText::new("Version").color(TEXT_MUTED));
            ui.monospace(egui::RichText::new(&manifest.version).color(ACCENT));
            ui.end_row();
        });
    });
```

## egui: RichText with colors and clickable card

```rust
ui.label(egui::RichText::new(&manifest.name).color(TEXT_PRIMARY).strong());
ui.label(egui::RichText::new(&manifest.description).color(TEXT_MUTED).small());

let frame = if selected {
    egui::Frame::new().fill(ACTIVE_BG).stroke(egui::Stroke::new(1.5, ACCENT_MINT))
} else {
    egui::Frame::new().fill(BG_DARK).stroke(egui::Stroke::new(0.5, BORDER))
};
let resp = frame.show(ui, |ui| { /* card */ }).response;
if resp.interact(egui::Sense::click()).clicked() { self.select_plugin(id); }
```

---

## Iced: Custom Theme with a Palette

```rust
fn app_theme() -> Theme {
    Theme::custom("Navy Dark", Palette {
        background: BG, text: TEXT, primary: ACCENT,
        success: ACCENT2, ..Palette::DARK
    })
}
```

## Iced: text_editor for multiline JSON

```rust
text_editor(&state.payload_content)
    .on_action(Message::PayloadEditorAction)
    .height(180)
    .style(|theme, status| {
        let mut base = text_editor::default(theme, status);
        base.background = Background::Color(SURFACE);
        base.border = Border { radius: 6.0.into(), width: 1.0, color: BORDER };
        base
    })
```

## Iced: Container + Button custom styles

```rust
container(sidebar).padding(16).width(Length::FillPortion(1)).style(|_| container::Style {
    background: Some(Background::Color(PANEL)),
    border: Border { radius: 10.0.into(), width: 1.0, color: BORDER },
    shadow: Shadow::default(), snap: false, text_color: Some(TEXT),
});

button(content).on_press(Message::SelectPlugin(id.clone()))
    .style(move |_theme, status| button::Style {
        background: Some(Background::Color(match status {
            button::Status::Hovered => lighten(card_bg, 0.06), _ => card_bg,
        })),
        text_color: TEXT,
        border: Border { radius: 8.0.into(), width: 1.0, color: border_color },
        shadow: Shadow::default(), snap: false,
    })
```

## Iced: Message-based state updates

```rust
#[derive(Debug, Clone)]
enum Message { SelectPlugin(String), SelectAction(String), InvokeSelected }

fn update(state: &mut IcedHostApp, message: Message) {
    match message {
        Message::SelectPlugin(id) => { state.selected_plugin_id = Some(id); }
        Message::InvokeSelected   => { state.invoke_selected(); }
        _ => {}
    }
}
```

---

## Dioxus: CSS style system as functions

```rust
mod styles {
    pub const BG: &str = "#0b1020";
    pub const PANEL: &str = "#121933";
    pub const ACCENT: &str = "#70a5ff";
}
fn card_style() -> String {
    format!("border: 1px solid {}; border-radius: 10px; background: {};",
        styles::BORDER, styles::PANEL)
}
```

## Dioxus: Reactive signals for selection state

```rust
let mut selected_plugin_id = use_signal(|| None::<String>);
let mut selected_action_id = use_signal(|| None::<String>);
let mut payload_input = use_signal(|| "{}".to_owned());
```

## Dioxus: Extracted component with EventHandler

```rust
#[component]
fn PluginCard(manifest: PluginManifest, selected: bool, on_select: EventHandler<()>) -> Element {
    rsx! {
        button { style: "{plugin_button_style(selected)}", onclick: move |_| on_select.call(()),
            div { "{manifest.name}" }
            div { style: "color: {styles::MUTED};", "{manifest.description}" }
        }
    }
}
```

## Dioxus: Collapsible section with details/summary

```rust
details { open: true,
    summary { style: "{details_summary_style()}", "📋 Manifest Details" }
    div {
        div { style: "{detail_row_style()}", span { "Version" } span { "{manifest.version}" } }
    }
}
```
