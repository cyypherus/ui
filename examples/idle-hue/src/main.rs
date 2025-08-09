use std::sync::Arc;

use arboard::Clipboard;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use ui::*;
use vello_svg::vello::{
    kurbo::Point,
    peniko::color::{AlphaColor, ColorSpaceTag, Oklch, Srgb, parse_color},
};

const GRAY_30: Color = Color::from_rgb8(30, 30, 30);
const GRAY_50: Color = Color::from_rgb8(60, 60, 60);

struct State {
    hex: String,
    error: Arc<Mutex<Option<String>>>,
    copy_button: ButtonState,
    paste_button: ButtonState,
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
    fn update_text(&mut self) {
        match self.color {
            CurrentColor::Srgb(color) => {
                self.hex = format!(
                    "#{:02x}{:02x}{:02x}",
                    (color.components[0] * 255.0) as u8,
                    (color.components[1] * 255.0) as u8,
                    (color.components[2] * 255.0) as u8,
                );
            }
            CurrentColor::Oklch(color) => {
                let color_text = format!(
                    "oklch({:.2} {:.2} {:.0})",
                    color.components[0], color.components[1], color.components[2],
                );
                self.hex = color_text;
            }
        }
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
            if let Err(e) = clipboard.set_text(self.hex.clone()) {
                eprintln!("Failed to copy to clipboard: {}", e);
            }
        }
    }

    fn paste(&mut self, app: &mut AppState<State>) {
        fn delay_clear_error(error: Arc<Mutex<Option<String>>>, app: &mut AppState<State>) {
            app.spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                let mut current_error = error.lock().await;
                *current_error = None;
            });
        }
        if let Ok(mut clipboard) = Clipboard::new() {
            if let Ok(text) = clipboard.get_text() {
                let trimmed = text.trim();
                let Some(parsed) = parse_color(&trimmed).ok() else {
                    self.error = Arc::new(Mutex::new(Some("Color parsing failed".to_string())));
                    delay_clear_error(self.error.clone(), app);
                    return;
                };
                match parsed.cs {
                    ColorSpaceTag::Srgb => {
                        self.color = CurrentColor::Srgb(parsed.to_alpha_color::<Srgb>())
                    }
                    ColorSpaceTag::Oklch => {
                        self.color = CurrentColor::Oklch(parsed.to_alpha_color::<Oklch>())
                    }
                    _ => {
                        self.error =
                            Arc::new(Mutex::new(Some("Unsupported color space".to_string())));
                        delay_clear_error(self.error.clone(), app);
                    }
                }
            }
        }
        self.update_text();
        self.sync_component_fields();
    }

    fn contrast_color(&self) -> Color {
        let rl =
            self.color.display().discard_alpha().relative_luminance() * self.color.components()[3];
        if rl > 0.5 { Color::BLACK } else { Color::WHITE }
    }

    fn default() -> Self {
        let mut s = State {
            hex: "ffffff".to_string(),
            error: Arc::new(Mutex::new(None)),
            copy_button: ButtonState::default(),
            paste_button: ButtonState::default(),
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
        s.update_text();
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
                            .corner_rounding(15.)
                            // .stroke(Color::WHITE, 3.)
                            .view()
                            .finish(),
                        text(id!(), s.hex.clone())
                            .font_size(20)
                            .fill(s.contrast_color())
                            .view()
                            .transition_duration(0.)
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
                            row_spaced(
                                10.,
                                vec![
                                    paste_button(),
                                    space().height(0.),
                                    if let Some(error) = s.error.blocking_lock().clone() {
                                        text(id!(), error).fill(s.contrast_color()).finish()
                                    } else {
                                        empty()
                                    },
                                    space().height(0.),
                                    copy_button(),
                                ],
                            ),
                        ])
                        .pad(10.),
                    ]),
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

fn copy_button<'n>() -> Node<'n, State, AppState<State>> {
    let color = Color::WHITE;
    button(id!(), binding!(State, copy_button))
        .corner_rounding(10.)
        .fill(GRAY_30)
        .label(move |button| {
            svg(id!(), include_str!("assets/copy.svg"))
                .fill({
                    match (button.depressed, button.hovered) {
                        (true, _) => color.map_lightness(|l| l - 0.2),
                        (false, true) => color.map_lightness(|l| l + 0.2),
                        (false, false) => color,
                    }
                })
                .finish()
                .pad(8.)
        })
        .on_click(|s, _app| {
            s.copy_to_clipboard();
        })
        .finish()
        .height(30.)
        .width(30.)
}

fn paste_button<'n>() -> Node<'n, State, AppState<State>> {
    let color = Color::WHITE;
    button(id!(), binding!(State, paste_button))
        .corner_rounding(10.)
        .fill(GRAY_30)
        .label(move |button| {
            svg(id!(), include_str!("assets/paste.svg"))
                .fill({
                    match (button.depressed, button.hovered) {
                        (true, _) => color.map_lightness(|l| l - 0.2),
                        (false, true) => color.map_lightness(|l| l + 0.2),
                        (false, false) => color,
                    }
                })
                .finish()
                .pad(6.)
        })
        .on_click(|s, app| {
            s.paste(app);
        })
        .finish()
        .height(30.)
        .width(30.)
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
            s.update_text();
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
                        .cursor_fill(s.color.display())
                        .highlight_fill({
                            let luminance = s.color.display().discard_alpha().relative_luminance();
                            if luminance < 0.3 || luminance > 0.7 {
                                Color::from_rgb8(100, 100, 100)
                            } else {
                                s.color.display().with_alpha(0.3)
                            }
                        })
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
                                        s.update_text();
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
