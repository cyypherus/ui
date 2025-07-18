use ui::*;

#[derive(Clone, Default)]
struct State {
    text: TextState,
    toggle: ToggleState,
}

fn main() {
    App::builder(
        State {
            text: TextState {
                text: "#000000".to_string(),
                editing: false,
            },
            toggle: ToggleState::default(),
        },
        || {
            dynamic(|_, _: &mut AppState<State>| {
                column_spaced(
                    20.,
                    vec![
                        space(),
                        text_field(id!(), binding!(State, text))
                            .font_size(40)
                            .finish(),
                        // toggle(id!(), binding!(State, toggle)).finish(),
                    ],
                )
                .pad(20.)
            })
        },
    )
    .inner_size(400, 300)
    .resizable(false)
    .start()
}
