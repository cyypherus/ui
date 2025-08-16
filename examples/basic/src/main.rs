use ui::*;

#[derive(Clone, Default)]
struct State {
    text: TextState,
    toggle: ToggleState,
    slider: SliderState,
}

fn main() {
    App::builder(
        State {
            text: TextState {
                text: "The scale factor is calculated differently on different platforms:"
                    .to_string(),
                editing: false,
            },
            toggle: ToggleState::default(),
            slider: SliderState::default(),
        },
        || {
            dynamic(|_, _: &mut AppState<State>| {
                column_spaced(
                    20.,
                    vec![
                        text_field(id!(), binding!(State, text))
                            .wrap()
                            .font_size(40)
                            .finish(),
                        toggle(id!(), binding!(State, toggle)).finish().height(50.),
                        slider(id!(), binding!(State, slider)).finish().height(50.),
                    ],
                )
                .pad(20.)
            })
        },
    )
    .inner_size(800, 600)
    // .resizable(false)
    .start()
}
