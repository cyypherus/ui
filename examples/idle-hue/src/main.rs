use std::fmt::Display;

use arboard::Clipboard;
use serde::{Deserialize, Serialize};
use ui::*;
use vello_svg::vello::{
    kurbo::Point,
    peniko::color::{AlphaColor, Oklch, Srgb, parse_color},
};

const GRAY_30: Color = Color::from_rgb8(30, 30, 30);
const GRAY_50: Color = Color::from_rgb8(60, 60, 60);

#[derive(Clone)]
struct State {
    text: TextState,
    copy_button: ButtonState,
    toggle_text_button: ButtonState,
    show_text: bool,
    loaded: bool,
    color: CurrentColor,
    oklch_mode: bool,
    mode_picker: SegmentPickerState<ColorMode>,
}

#[derive(Clone, Debug)]
enum CurrentColor {
    Srgb(AlphaColor<Srgb>),
    Oklch(AlphaColor<Oklch>),
}

impl CurrentColor {
    fn components(&self) -> [f32; 4] {
        match self {
            CurrentColor::Srgb(color) => color.components,
            CurrentColor::Oklch(color) => color.components,
        }
    }
    fn components_mut(&mut self) -> &mut [f32; 4] {
        match self {
            CurrentColor::Srgb(color) => &mut color.components,
            CurrentColor::Oklch(color) => &mut color.components,
        }
    }
    fn display(&self) -> AlphaColor<Srgb> {
        match self {
            CurrentColor::Srgb(color) => color.convert::<Srgb>(),
            CurrentColor::Oklch(color) => color.convert::<Srgb>(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SavedState {
    text: String,
}

impl State {
    fn update_current_color(&mut self) {
        let hex_color = format!("#{}", self.text.text);
        let Some(parsed) = parse_color(&hex_color).ok() else {
            self.color = CurrentColor::Srgb(Color::TRANSPARENT);
            return;
        };
        let srgb = parsed.to_alpha_color::<Srgb>();
        let components = srgb.components;
        self.color = CurrentColor::Srgb(Color::from_rgba8(
            (components[0] * 255.0) as u8,
            (components[1] * 255.0) as u8,
            (components[2] * 255.0) as u8,
            (components[3] * 255.0) as u8,
        ));
    }

    fn update_text(&mut self) {
        match self.color {
            CurrentColor::Srgb(color) => {
                self.text.text = format!(
                    "{:02x}{:02x}{:02x}",
                    (color.components[0] * 255.0) as u8,
                    (color.components[1] * 255.0) as u8,
                    (color.components[2] * 255.0) as u8,
                );
            }
            CurrentColor::Oklch(color) => {
                let color_text = format!(
                    "{:.2}, {:.2}, {:.2}",
                    color.components[0], color.components[1], color.components[2],
                );
                self.text.text = color_text;
            }
        }
        self.text.editing = false;
    }

    fn rgb_to_oklch(&mut self) {
        match self.color {
            CurrentColor::Srgb(color) => {
                self.color = CurrentColor::Oklch(color.convert::<Oklch>());
            }
            _ => {}
        }
    }

    fn oklch_to_rgb(&mut self) {
        match self.color {
            CurrentColor::Oklch(color) => {
                let mut converted = color.convert::<Srgb>();
                converted.components[0] = converted.components[0].clamp(0.0, 1.0);
                converted.components[1] = converted.components[1].clamp(0.0, 1.0);
                converted.components[2] = converted.components[2].clamp(0.0, 1.0);
                self.color = CurrentColor::Srgb(converted);
            }
            _ => {}
        }
    }

    fn update_component(color: &mut CurrentColor, component_index: usize, drag: DragState) {
        match drag {
            DragState::Began(_) => (),
            DragState::Updated {
                delta: Point { y, .. },
                ..
            }
            | DragState::Completed {
                delta: Point { y, .. },
                ..
            } => {
                let y = y as f32;
                match color {
                    CurrentColor::Oklch(color) => match component_index {
                        0 => {
                            color.components[0] = (color.components[0] - y * 0.001).clamp(0.0, 1.0)
                        }
                        1 => {
                            color.components[1] = (color.components[1] - y * 0.0005).clamp(0.0, 0.5)
                        }
                        2 => {
                            color.components[2] -= y * 0.5;
                            if color.components[2] < 0.0 {
                                color.components[2] += 360.0
                            }
                            if color.components[2] >= 360.0 {
                                color.components[2] -= 360.0
                            }
                        }
                        _ => return,
                    },
                    CurrentColor::Srgb(color) => {
                        color.components[component_index] =
                            (color.components[component_index] - y * 0.001).clamp(0.0, 1.0);
                    }
                }
            }
        }
        // if matches!(drag, DragState::Completed { .. }) {
        //     let state = SavedState {
        //         text: self.text.text.clone(),
        //     };
        //     app.spawn(async move { _ = save_state_to_file(&state).await });
        // }
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
            toggle_text_button: ButtonState::default(),
            show_text: false,
            loaded: false,
            color: CurrentColor::Oklch(AlphaColor::<Oklch>::new([0.0, 0.0, 0.0, 1.0])),
            oklch_mode: false,
            mode_picker: SegmentPickerState::new(ColorMode::Rgb),
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
                            .fill(s.color.display())
                            .corner_rounding(5.)
                            .stroke(Color::WHITE, 3.)
                            .view()
                            .finish(),
                        button(id!(), binding!(State, toggle_text_button))
                            // .label("Toggle Mode")
                            .on_click(|s, _a| {
                                s.show_text = !s.show_text;
                            })
                            .finish(),
                        if s.show_text { hex_row() } else { empty() },
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

fn hex_text_field<'n>() -> Node<'n, State, AppState<State>> {
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
        .finish()
}

fn copy_button<'n>() -> Node<'n, State, AppState<State>> {
    dynamic(|s: &mut State, _app| {
        let rl = s.color.display().discard_alpha().relative_luminance() * s.color.components()[3];
        button(id!(), binding!(State, copy_button))
            .corner_rounding(10.)
            .fill(s.color.display())
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
            .width(40.)
    })
}

fn hex_row<'n>() -> Node<'n, State, AppState<State>> {
    row(vec![
        text(id!(), "#").font_size(40).finish().width(20.),
        hex_text_field(),
        copy_button(),
    ])
    .align_contents(Align::CenterY)
    .height(30.)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColorMode {
    Oklch,
    Rgb,
}

impl Display for ColorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorMode::Oklch => write!(f, "OKLCH"),
            ColorMode::Rgb => write!(f, "RGB"),
        }
    }
}

fn mode_toggle_button<'n>() -> Node<'n, State, AppState<State>> {
    segment_picker(
        id!(),
        vec![ColorMode::Rgb, ColorMode::Oklch],
        binding!(State, mode_picker),
    )
    .on_select(|s, _a, sel| match sel {
        ColorMode::Oklch => {
            s.oklch_mode = true;
            s.rgb_to_oklch();
        }
        ColorMode::Rgb => {
            s.oklch_mode = false;
            s.oklch_to_rgb();
        }
    })
    .finish()
    .height(40.)
}

fn color_component_sliders<'n>() -> Node<'n, State, AppState<State>> {
    dynamic(|s: &mut State, _app| {
        let color = s.color.clone();
        row_spaced(
            10.,
            (0usize..3)
                .map(|i| {
                    text(id!(i as u64), format!("{:.2}", s.color.components()[i]))
                        .font_size(24)
                        .view()
                        .finish()
                        .pad(5.)
                        .pad_y(10.)
                        .attach_under(stack(vec![
                            rect(id!(i as u64))
                                .fill(Color::TRANSPARENT)
                                .view()
                                .on_drag(move |s: &mut State, _a, drag| {
                                    State::update_component(&mut s.color, i, drag);
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
                                        let mut color = color.clone();
                                        let drag = -150.;
                                        State::update_component(
                                            &mut color,
                                            i,
                                            DragState::Updated {
                                                start: Point::new(0.0, 0.0),
                                                current: Point::new(0.0, drag),
                                                delta: Point::new(0.0, drag),
                                                distance: drag as f32,
                                            },
                                        );
                                        color.display()
                                    })
                                    .corner_rounding(5.)
                                    .finish()
                                    .height(5.)
                                    .pad(5.),
                                space(),
                                rect(id!(i as u64))
                                    .fill({
                                        let mut color = color.clone();
                                        let drag = 150.;
                                        State::update_component(
                                            &mut color,
                                            i,
                                            DragState::Updated {
                                                start: Point::new(0.0, 0.0),
                                                current: Point::new(0.0, drag),
                                                delta: Point::new(0.0, drag),
                                                distance: drag as f32,
                                            },
                                        );
                                        color.display()
                                    })
                                    .corner_rounding(5.)
                                    .finish()
                                    .height(5.)
                                    .pad(5.),
                            ]),
                        ]))
                })
                .collect(),
        )
    })
}

fn rgb_row<'n>() -> Node<'n, State, AppState<State>> {
    column_spaced(10., vec![mode_toggle_button(), color_component_sliders()])
}
