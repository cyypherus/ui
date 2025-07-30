use arboard::Clipboard;
use color::{Srgb, parse_color};
use serde::{Deserialize, Serialize};
use ui::*;

const GRAY_30: Color = Color::from_rgb8(30, 30, 30);
const GRAY_50: Color = Color::from_rgb8(60, 60, 60);

#[derive(Clone)]
struct State {
    text: TextState,
    copy_button: ButtonState,
    loaded: bool,
    color: Color,
    alpha_enabled: bool,
}

#[derive(Serialize, Deserialize)]
struct SavedState {
    text: String,
}

impl State {
    fn update_current_color(&mut self) {
        let hex_color = format!("#{}", self.text.text);
        let Some(parsed) = parse_color(&hex_color).ok() else {
            self.color = Color::TRANSPARENT;
            return;
        };
        let srgb = parsed.to_alpha_color::<Srgb>();
        let components = srgb.components;
        self.color = Color::from_rgba8(
            (components[0] * 255.0) as u8,
            (components[1] * 255.0) as u8,
            (components[2] * 255.0) as u8,
            (components[3] * 255.0) as u8,
        );
    }

    fn update_hex(&mut self) {
        let hex_color = if self.alpha_enabled {
            format!(
                "{:02x}{:02x}{:02x}{:02x}",
                (self.color.components[0] * 255.0) as u8,
                (self.color.components[1] * 255.0) as u8,
                (self.color.components[2] * 255.0) as u8,
                (self.color.components[3] * 255.0) as u8,
            )
        } else {
            format!(
                "{:02x}{:02x}{:02x}",
                (self.color.components[0] * 255.0) as u8,
                (self.color.components[1] * 255.0) as u8,
                (self.color.components[2] * 255.0) as u8,
            )
        };
        self.text.text = hex_color;
        self.text.editing = false;
    }

    fn update_rgb_component(
        &mut self,
        component_index: usize,
        drag: DragState,
        app: &mut AppState<State>,
    ) {
        match drag {
            DragState::Began(_) => (),
            DragState::Updated { delta, .. } | DragState::Completed { delta, .. } => {
                self.color.components[component_index] -= delta.y as f32 * 0.001;
                self.color.components[component_index] =
                    self.color.components[component_index].clamp(0., 1.);
                self.update_hex();
            }
        }
        app.end_editing();
        if matches!(drag, DragState::Completed { .. }) {
            let state = SavedState {
                text: self.text.text.clone(),
            };
            app.spawn(async move { _ = save_state_to_file(&state).await });
        }
    }

    fn copy_to_clipboard(&self) {
        if let Ok(mut clipboard) = Clipboard::new() {
            if let Err(e) = clipboard.set_text(self.text.text.clone()) {
                eprintln!("Failed to copy to clipboard: {}", e);
            }
        }
    }

    fn default() -> Self {
        State {
            text: TextState {
                text: "00000000".to_string(),
                editing: false,
            },
            copy_button: ButtonState::default(),
            loaded: false,
            color: Color::TRANSPARENT,
            alpha_enabled: false,
        }
    }
}

async fn save_state_to_file(state: &SavedState) -> Result<(), String> {
    let json = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    tokio::fs::write("idle-hue-state.json", json)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

async fn load_state_from_file() -> Result<SavedState, String> {
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    let content = tokio::fs::read_to_string("idle-hue-state.json")
        .await
        .map_err(|e| e.to_string())?;
    let state: SavedState = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(state)
}

fn main() {
    App::builder(State::default(), || {
        dynamic(|s: &mut State, _: &mut AppState<State>| {
            column_spaced(
                15.,
                if !s.loaded {
                    vec![
                        circle(id!())
                            .stroke(Color::WHITE, 5.)
                            .view()
                            .on_appear(|_state: &mut State, app| {
                                println!("Loading saved state...");
                                app.spawn_with_result(
                                    async move { load_state_from_file().await },
                                    |state, result| match result {
                                        Ok(saved_state) => {
                                            state.text.text = saved_state.text;
                                            state.loaded = true;
                                            state.update_current_color();
                                            if state.text.text.is_empty() {
                                                state.text.text = "000000".to_string();
                                            }
                                        }
                                        Err(_) => {
                                            state.text.text = "000000".to_string();
                                            state.loaded = true;
                                            state.update_current_color();
                                        }
                                    },
                                );
                            })
                            .finish()
                            .width(10.)
                            .height(10.),
                    ]
                } else {
                    vec![
                        rect(id!())
                            .fill(s.color)
                            .corner_rounding(5.)
                            .stroke(Color::WHITE, 3.)
                            .view()
                            .finish(),
                        hex_row(),
                        rgb_row(),
                    ]
                },
            )
            .pad(20.)
            .pad_top(15.)
        })
    })
    .inner_size(400, 300)
    .start()
}

fn hex_row<'n>() -> Node<'n, State, AppState<State>> {
    dynamic(|s: &mut State, _app| {
        let rl = s.color.discard_alpha().relative_luminance() * s.color.components[3];
        row(vec![
            text(id!(), "#").font_size(40).finish().width(20.),
            text_field(id!(), binding!(State, text))
                .font_size(40)
                .background_fill(None)
                .no_background_stroke()
                .on_edit(|s, a, edit| {
                    let EditInteraction::Update(text) = edit else {
                        return;
                    };
                    s.text.text = text.clone();
                    s.update_current_color();
                    let state = SavedState {
                        text: text.to_string(),
                    };
                    a.spawn(async move { _ = save_state_to_file(&state).await });
                })
                .finish(),
            button(id!(), binding!(State, copy_button))
                .corner_rounding(10.)
                .fill(s.color)
                .label(move |button| {
                    svg(id!(), include_str!("assets/copy.svg"))
                        .fill({
                            let color = if rl > 0.5 { Color::BLACK } else { Color::WHITE };
                            match (button.depressed, button.hovered) {
                                (true, _) => color.map_lightness(|l| l - 0.2),
                                (false, true) => color.map_lightness(|l| l + 0.2),
                                (false, false) => color,
                            }
                        })
                        .finish()
                        .pad(10.)
                })
                .on_click(|s, _app| {
                    s.copy_to_clipboard();
                })
                .finish()
                .height(40.)
                .width(40.),
        ])
        .align_contents(Align::CenterY)
        .height(30.)
    })
}

fn rgb_row<'n>() -> Node<'n, State, AppState<State>> {
    dynamic(|s: &mut State, _app| {
        row_spaced(
            10.,
            vec![
                svg(id!(), include_str!("assets/arrow-up-down.svg"))
                    .fill(Color::WHITE)
                    .view()
                    .finish()
                    .height(30.)
                    .width(20.)
                    .pad(5.)
                    .pad_y(10.)
                    .attach_under(stack(vec![
                        rect(id!())
                            .fill(Color::TRANSPARENT)
                            .view()
                            .on_drag(move |s: &mut State, a, drag| {
                                s.update_rgb_component(0, drag, a);
                                s.update_rgb_component(1, drag, a);
                                s.update_rgb_component(2, drag, a);
                            })
                            .finish(),
                        rect(id!())
                            .fill(GRAY_30)
                            .corner_rounding(5.)
                            .view()
                            .finish(),
                        column(vec![
                            rect(id!())
                                .fill({
                                    let mut color = s.color.clone();
                                    color.components[0] += 0.3;
                                    color.components[1] += 0.3;
                                    color.components[2] += 0.3;
                                    color
                                })
                                .corner_rounding(5.)
                                .finish()
                                .height(5.)
                                .pad(5.),
                            space(),
                            rect(id!())
                                .fill({
                                    let mut color = s.color.clone();
                                    color.components[0] -= 0.3;
                                    color.components[1] -= 0.3;
                                    color.components[2] -= 0.3;
                                    color
                                })
                                .corner_rounding(5.)
                                .finish()
                                .height(5.)
                                .pad(5.),
                        ]),
                    ])),
                group(
                    (0usize..=2)
                        .map(|i| {
                            text(
                                id!(i as u64),
                                format!("{:.0}", s.color.components[i] * 255.),
                            )
                            .font_size(30)
                            .view()
                            .finish()
                            .pad(5.)
                            .pad_y(10.)
                            .attach_under(stack(vec![
                                rect(id!(i as u64))
                                    .fill(Color::TRANSPARENT)
                                    .view()
                                    .on_drag(move |s: &mut State, a, drag| {
                                        s.update_rgb_component(i, drag, a);
                                    })
                                    .finish(),
                                rect(id!(i as u64))
                                    .fill(GRAY_50)
                                    .corner_rounding(5.)
                                    .view()
                                    .finish(),
                                column(vec![
                                    rect(id!(i as u64))
                                        .fill({
                                            let mut color = s.color.clone();
                                            color.components[i] += 0.5;
                                            color
                                        })
                                        .corner_rounding(5.)
                                        .finish()
                                        .height(5.)
                                        .pad(5.),
                                    space(),
                                    rect(id!(i as u64))
                                        .fill({
                                            let mut color = s.color.clone();
                                            color.components[i] -= 0.5;
                                            color
                                        })
                                        .corner_rounding(5.)
                                        .finish()
                                        .height(5.)
                                        .pad(5.),
                                ]),
                            ]))
                        })
                        .collect(),
                ),
                if s.alpha_enabled {
                    text(id!(), format!("{:.0}", s.color.components[3] * 255.))
                        .font_size(30)
                        .view()
                        .on_drag(move |s: &mut State, a, drag| {
                            s.update_rgb_component(3, drag, a);
                        })
                        .on_click(|s: &mut State, a, c, _| {
                            if matches!(c, ClickState::Completed) {
                                s.color.components[3] = 1.0;
                                s.alpha_enabled = false;
                                s.update_hex();
                                a.end_editing();
                            }
                        })
                        .finish()
                        .pad(5.)
                        .pad_y(10.)
                        .attach_under(stack(vec![
                            rect(id!())
                                .fill(Color::TRANSPARENT)
                                .view()
                                .on_drag(move |s: &mut State, a, drag| {
                                    s.update_rgb_component(3, drag, a);
                                })
                                .finish(),
                            rect(id!())
                                .fill(GRAY_50)
                                .corner_rounding(5.)
                                .view()
                                .finish(),
                        ]))
                } else {
                    text(id!(), "A")
                        .font_size(30)
                        .view()
                        .on_click(|s: &mut State, a, c, _| {
                            if matches!(c, ClickState::Completed) {
                                s.color.components[3] = 1.0;
                                s.alpha_enabled = true;
                                s.update_hex();
                                a.end_editing();
                            }
                        })
                        .finish()
                        .pad(5.)
                        .pad_y(10.)
                        .attach_under(rect(id!()).fill(GRAY_30).corner_rounding(5.).finish())
                },
            ],
        )
    })
}
