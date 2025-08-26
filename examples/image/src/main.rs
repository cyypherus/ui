use arboard::Clipboard;
use parley::FontWeight;
use std::sync::Arc;
use tokio::sync::Mutex;
use ui::*;

#[derive(Clone)]
enum DownloadState {
    Idle,
    Downloading,
    Success(String, Arc<Vec<u8>>),
    Error(String),
}

struct State {
    input: String,
    load_button: ButtonState,
    paste_button: ButtonState,
    download_state: Arc<Mutex<DownloadState>>,
}

impl State {
    fn new() -> Self {
        Self {
            input: "".to_string(),
            load_button: ButtonState::default(),
            paste_button: ButtonState::default(),
            download_state: Arc::new(Mutex::new(DownloadState::Idle)),
        }
    }

    fn load_image(&mut self, app: &mut AppState<State>) {
        let input = self.input.trim().to_string();
        if input.is_empty() {
            return;
        }

        let download_state = self.download_state.clone();

        let redraw = app.redraw_trigger();
        if input.starts_with("http://") || input.starts_with("https://") {
            app.spawn(async move {
                {
                    let mut state = download_state.lock().await;
                    *state = DownloadState::Downloading;
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                let result = reqwest::get(&input).await;

                match result {
                    Ok(response) => {
                        if response.status().is_success() {
                            match response.bytes().await {
                                Ok(bytes) => {
                                    let mut state = download_state.lock().await;
                                    *state = DownloadState::Success(input, Arc::new(bytes.into()));
                                }
                                Err(e) => {
                                    let mut state = download_state.lock().await;
                                    *state = DownloadState::Error(format!(
                                        "Failed to read response: {e}"
                                    ));
                                }
                            }
                        } else {
                            let mut state = download_state.lock().await;
                            *state = DownloadState::Error(format!(
                                "HTTP {}: {}",
                                response.status().as_u16(),
                                response
                                    .status()
                                    .canonical_reason()
                                    .unwrap_or("Unknown error")
                            ));
                        }
                    }
                    Err(e) => {
                        let mut state = download_state.lock().await;
                        *state = DownloadState::Error(format!("Request failed: {e}"));
                    }
                }
                redraw.trigger().await;
            });
        } else {
            app.spawn(async move {
                {
                    let mut state = download_state.lock().await;
                    *state = DownloadState::Downloading;
                }

                match std::fs::read(&input) {
                    Ok(bytes) => {
                        let mut state = download_state.lock().await;
                        *state = DownloadState::Success(input, Arc::new(bytes));
                    }
                    Err(e) => {
                        let mut state = download_state.lock().await;
                        *state = DownloadState::Error(format!("Failed to read file: {e}"));
                    }
                }
                redraw.trigger().await;
            });
        }
    }

    fn paste_from_clipboard(&mut self) {
        if let Ok(mut clipboard) = Clipboard::new()
            && let Ok(text) = clipboard.get_text()
        {
            self.input = text.trim().to_string();
        }
    }
}

fn main() {
    App::builder(State::new(), || {
        dynamic(|s: &mut State, _app: &mut AppState<State>| {
            let download_state = s.download_state.blocking_lock().clone();
            let download_state_for_button = download_state.clone();

            column_spaced(
                20.,
                vec![
                    text(id!(), "Image Loader")
                        .font_size(32)
                        .font_weight(FontWeight::BOLD)
                        .finish()
                        .pad(10.),
                    row_spaced(
                        10.,
                        vec![
                            text(id!(), {
                                let input = s.input.clone();
                                if input.len() > 20 {
                                    format!("{}...", &input[..20])
                                } else {
                                    input
                                }
                            })
                            .font_size(16)
                            .view()
                            .finish()
                            .pad(10.)
                            .width_range(..200.),
                            button(id!(), binding!(State, paste_button))
                                .label(|_, _| text(id!(), "Paste").finish())
                                .on_click(|s, _| s.paste_from_clipboard())
                                .finish()
                                .height(40.)
                                .width(80.),
                        ],
                    ),
                    button(id!(), binding!(State, load_button))
                        .label(move |_, _| {
                            text(
                                id!(),
                                match download_state_for_button {
                                    DownloadState::Downloading => "Loading...",
                                    _ => "Load Image",
                                },
                            )
                            .finish()
                        })
                        .on_click(|s, app| {
                            if matches!(
                                s.download_state.blocking_lock().clone(),
                                DownloadState::Downloading
                            ) {
                                return;
                            }
                            s.load_image(app);
                        })
                        .finish()
                        .height(50.)
                        .width(200.),
                    match download_state {
                        DownloadState::Idle => {
                            text(id!(), "Enter a URL or file path and click Load")
                                .font_size(14)
                                .finish()
                        }
                        DownloadState::Downloading => {
                            text(id!(), "Loading image...").font_size(14).finish()
                        }
                        DownloadState::Success(ref image_id, ref bytes) => column_spaced(
                            10.,
                            vec![
                                text(id!(), format!("Loaded {} bytes", bytes.len()))
                                    .font_size(14)
                                    .finish(),
                                image_from_bytes(id!(), bytes.clone())
                                    .image_id(image_id)
                                    .view()
                                    .finish()
                                    .height_range(100.0..)
                                    .width_range(100.0..),
                            ],
                        ),
                        DownloadState::Error(ref error) => text(id!(), format!("Error: {error}"))
                            .font_size(14)
                            .fill(Color::from_rgb8(255, 0, 0))
                            .finish(),
                    },
                ],
            )
            .pad(20.)
            .pad_top(20.)
        })
    })
    .inner_size(600, 700)
    .resizable(true)
    .title("Image Download")
    .start()
}
