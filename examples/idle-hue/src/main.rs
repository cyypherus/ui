use arboard::Clipboard;
use color::{parse_color, Srgb};
use serde::{Deserialize, Serialize};

use ui::*;

#[derive(Clone, Default)]
struct State {
    text: TextState,
    copy_button: ButtonState,
    save_button: ButtonState,
}

#[derive(Serialize, Deserialize)]
struct SavedState {
    text: String,
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

    fn copy_to_clipboard(&self) {
        if let Ok(mut clipboard) = Clipboard::new() {
            let hex_code = format!("#{}", self.text.text);
            if let Err(e) = clipboard.set_text(hex_code) {
                eprintln!("Failed to copy to clipboard: {}", e);
            }
        }
    }

    fn get_saved_state(&self) -> SavedState {
        SavedState {
            text: self.text.text.clone(),
        }
    }

    fn load_from_file() -> Self {
        match tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(load_state_from_file())
        {
            Ok(saved_state) => State {
                text: TextState {
                    text: saved_state.text,
                    editing: false,
                },
                copy_button: ButtonState::default(),
                save_button: ButtonState::default(),
            },
            Err(_) => State {
                text: TextState {
                    text: "000000".to_string(),
                    editing: false,
                },
                copy_button: ButtonState::default(),
                save_button: ButtonState::default(),
            },
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
    let content = tokio::fs::read_to_string("idle-hue-state.json")
        .await
        .map_err(|e| e.to_string())?;
    let state: SavedState = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(state)
}

fn main() {
    App::builder(State::load_from_file(), || {
        dynamic(|s: &mut State, _: &mut AppState<State>| {
            column_spaced(
                20.,
                vec![
                    rect(id!())
                        .fill(s.current_color().unwrap_or(Color::TRANSPARENT))
                        .corner_rounding(5.)
                        .stroke(Color::WHITE, 3.)
                        .finish(),
                    hex_row(),
                    row(vec![button(id!(), binding!(State, save_button))
                        .corner_rounding(5.)
                        .label(|_| text(id!(), "Save").finish().pad(8.))
                        .on_click(|s, app| {
                            let saved_state = s.get_saved_state();
                            app.spawn_with_result(
                                async move { save_state_to_file(&saved_state).await },
                                |_state, result| match result {
                                    Ok(_) => println!("State saved successfully!"),
                                    Err(e) => eprintln!("Failed to save state: {}", e),
                                },
                            );
                        })
                        .finish()
                        .height(30.)])
                    .pad_top(10.),
                ],
            )
            .pad(20.)
            .pad_top(15.)
        })
    })
    .inner_size(400, 300)
    .start()
}

fn hex_row<'n>() -> Node<'n, State, AppState<State>> {
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
            .on_click(|s, _app| {
                s.copy_to_clipboard();
            })
            .finish()
            .height(40.)
            .width(40.),
    ])
    .align_contents(Align::CenterY)
    .height(30.)
}
