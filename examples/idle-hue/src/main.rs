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
    mode_picker: ToggleState,
    component_fields: [TextState; 3],
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

    fn clamp_color_components(&mut self) {
        if self.oklch_mode {
            self.color.components_mut()[0] = self.color.components()[0].clamp(0.0, 1.0);
            self.color.components_mut()[1] = self.color.components()[1].clamp(0.0, 0.5);
            self.color.components_mut()[2] = self.color.components()[2].clamp(0.0, 360.0);
        } else {
            for i in 0..3 {
                self.color.components_mut()[i] = self.color.components()[i].clamp(0.0, 1.0);
            }
        }
    }

    fn sync_component_fields(&mut self) {
        if self.oklch_mode {
            self.component_fields[0].text = format!("{:.2}", self.color.components()[0])
                .trim_start_matches('0')
                .to_string();
            self.component_fields[1].text = format!("{:.2}", self.color.components()[1])
                .trim_start_matches('0')
                .to_string();
            self.component_fields[2].text = format!("{:.0}", self.color.components()[2]);
        } else {
            for i in 0..3 {
                self.component_fields[i].text =
                    format!("{}", (self.color.components()[i] * 255.) as u8);
            }
        }
    }

    fn copy_to_clipboard(&self) {
        if let Ok(mut clipboard) = Clipboard::new() {
            if let Err(e) = clipboard.set_text(self.text.text.clone()) {
                eprintln!("Failed to copy to clipboard: {}", e);
            }
        }
    }

    fn contrast_color(&self) -> Color {
        let rl =
            self.color.display().discard_alpha().relative_luminance() * self.color.components()[3];
        if rl > 0.5 { Color::BLACK } else { Color::WHITE }
    }

    fn default() -> Self {
        let mut s = State {
            text: TextState {
                text: "00000000".to_string(),
                editing: false,
            },
            copy_button: ButtonState::default(),
            toggle_text_button: ButtonState::default(),
            show_text: false,
            loaded: false,
            color: CurrentColor::Oklch(AlphaColor::<Oklch>::new([1.0, 0.0, 0.0, 1.0])),
            oklch_mode: true,
            mode_picker: ToggleState::on(),
            component_fields: [
                TextState::default(),
                TextState::default(),
                TextState::default(),
            ],
        };
        s.sync_component_fields();
        s
    }
}

fn main() {
    App::builder(State::default(), || {
        dynamic(|s: &mut State, _: &mut AppState<State>| {
            column_spaced(
                15.,
                vec![
                    stack(vec![
                        rect(id!())
                            .fill(s.color.display())
                            .corner_rounding(5.)
                            .stroke(Color::WHITE, 3.)
                            .view()
                            .finish(),
                        column(vec![
                            row_spaced(
                                10.,
                                vec![
                                    space().height(0.),
                                    text(id!(), "oklch").fill(s.contrast_color()).finish(),
                                    mode_toggle_button(),
                                ],
                            ),
                            space(),
                        ])
                        .pad(10.),
                    ]),
                    // button(id!(), binding!(State, toggle_text_button))
                    //     // .label("Toggle Mode")
                    //     .on_click(|s, _a| {
                    //         s.show_text = !s.show_text;
                    //     })
                    //     .finish(),
                    if s.show_text { hex_row() } else { empty() },
                    rgb_row(),
                ],
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
        .on_edit(|s, _, edit| {
            let EditInteraction::Update(text) = edit else {
                return;
            };
            s.text.text = text.clone();
            s.update_current_color();
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

fn mode_toggle_button<'n>() -> Node<'n, State, AppState<State>> {
    toggle(id!(), binding!(State, mode_picker))
        .on_fill(GRAY_50)
        .off_fill(GRAY_30)
        .on_toggle(|s, _, on| {
            if on {
                s.oklch_mode = true;
                s.rgb_to_oklch();
            } else {
                s.oklch_mode = false;
                s.oklch_to_rgb();
            }
            s.sync_component_fields();
        })
        .finish()
        .height(20.)
        .width(40.)
}

fn color_component_sliders<'n>() -> Node<'n, State, AppState<State>> {
    dynamic(|s: &mut State, _app| {
        let color = s.color.clone();
        let oklch_mode = s.oklch_mode;
        row_spaced(
            10.,
            (0usize..3)
                .map(|i| {
                    row(vec![
                        text_field(
                            id!(i as u64),
                            Binding::new(
                                move |s: &State| s.component_fields[i].clone(),
                                move |s, value| s.component_fields[i] = value,
                            ),
                        )
                        .background_stroke(GRAY_50, s.color.display(), 2.)
                        .on_edit(move |s, _, edit| match edit {
                            EditInteraction::Update(new) => {
                                if oklch_mode {
                                    if let Ok(value) = new.parse::<f32>() {
                                        s.color.components_mut()[i] = value;
                                    }
                                } else {
                                    if let Ok(value) = new.parse::<u8>() {
                                        s.color.components_mut()[i] = value as f32 / 255.;
                                    }
                                }
                            }
                            EditInteraction::End => {
                                s.clamp_color_components();
                                s.sync_component_fields();
                            }
                        })
                        .font_size(24)
                        .finish()
                        .pad(10.),
                        svg(id!(i as u64), include_str!("assets/arrow-up-down.svg"))
                            .fill(Color::WHITE)
                            .finish()
                            .height(30.)
                            .width(20.)
                            .pad(5.)
                            .pad_y(10.)
                            .attach_under(stack(vec![
                                rect(id!(i as u64))
                                    .fill(Color::TRANSPARENT)
                                    .view()
                                    .on_drag(move |s: &mut State, a, drag| {
                                        State::update_component(&mut s.color, i, drag);
                                        s.sync_component_fields();
                                        match drag {
                                            DragState::Began { .. } => {
                                                a.end_editing(s);
                                            }
                                            _ => (),
                                        }
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
                            ])),
                    ])
                    .attach_under(
                        rect(id!(i as u64))
                            .fill(GRAY_30)
                            .corner_rounding(10.)
                            .view()
                            .finish(),
                    )
                })
                .collect(),
        )
    })
}

fn rgb_row<'n>() -> Node<'n, State, AppState<State>> {
    column_spaced(
        10.,
        vec![
            // mode_toggle_button(),
            color_component_sliders(),
        ],
    )
}
