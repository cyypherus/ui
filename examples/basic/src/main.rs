use ui::*;

#[derive(Clone, Default)]
struct State {
    text: TextState,
    toggle: ToggleState,
}

fn main() {
    App::start(
        State {
            text: TextState {
                text: "The scale factor is calculated differently on different platforms:"
                    .to_string(),
                editing: false,
            },
            toggle: ToggleState::default(),
        },
        || {
            dynamic(|_: &mut AppState<State>| {
                column_spaced(
                    20.,
                    vec![
                        text_field(id!(), binding!(State, text))
                            .font_size(40)
                            .finish(),
                        toggle(id!(), binding!(State, toggle)).finish(),
                    ],
                )
                .pad(20.)
            })
        },
    )
}
