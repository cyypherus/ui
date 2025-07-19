use arboard::Clipboard;
use color::{parse_color, Srgb};
use ui::*;

#[derive(Clone, Default)]
struct State {
    text: TextState,
    copy_button: ButtonState,
}

impl State {
    fn current_color(&self) -> Option<Color> {
        let hex_color = format!("#{}", self.text.text);
        let parsed = parse_color(&hex_color).ok()?;
        let srgb = parsed.to_alpha_color::<Srgb>();
        let components = srgb.components;
        Some(Color::from_rgb8(
            (components[0] * 255.0) as u8,
            (components[1] * 255.0) as u8,
            (components[2] * 255.0) as u8,
        ))
    }
}

fn main() {
    App::builder(
        State {
            text: TextState {
                text: "000000".to_string(),
                editing: false,
            },
            copy_button: ButtonState::default(),
        },
        || {
            dynamic(|s: &mut State, _: &mut AppState<State>| {
                column_spaced(
                    20.,
                    vec![
                        space().height(10.),
                        rect(id!())
                            .fill(s.current_color().unwrap_or(Color::TRANSPARENT))
                            .corner_rounding(5.)
                            .stroke(Color::WHITE, 3.)
                            .finish(),
                        row(vec![
                            text(id!(), "#").font_size(40).finish().width(20.),
                            space().width(10.),
                            text_field(id!(), binding!(State, text))
                                .font_size(40)
                                .background_fill(None)
                                .no_background_stroke()
                                .finish(),
                            button(id!(), binding!(State, copy_button))
                                .corner_rounding(10.)
                                .label(|button| {
                                    svg(id!(), include_str!("assets/copy.svg"))
                                        .fill(match (button.depressed, button.hovered) {
                                            (true, _) => Color::from_rgb8(190, 190, 190),
                                            (false, true) => Color::from_rgb8(250, 250, 250),
                                            (false, false) => Color::from_rgb8(240, 240, 240),
                                        })
                                        .finish()
                                        .pad(10.)
                                })
                                .on_click(|s, app| {
                                    if let Ok(mut clipboard) = Clipboard::new() {
                                        let hex_code = format!("#{}", s.text.text);
                                        if let Err(e) = clipboard.set_text(hex_code) {
                                            eprintln!("Failed to copy to clipboard: {}", e);
                                        }
                                    }
                                })
                                .finish()
                                .height(40.)
                                .width(40.),
                        ])
                        .align_contents(Align::CenterY)
                        .height(30.),
                    ],
                )
                .pad(20.)
            })
        },
    )
    .inner_size(400, 300)
    // .resizable(false)
    .start()
}
