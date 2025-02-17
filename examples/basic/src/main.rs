use ui::*;

#[derive(Clone, Default)]
struct AppState {
    text: TextState,
    toggle: ToggleState,
}

fn main() {
    App::start(
        AppState {
            text: TextState {
                text: "The scale factor is calculated differently on different platforms:"
                    .to_string(),
                editing: false,
            },
            toggle: ToggleState::default(),
        },
        dynamic_node(|_: &mut AppState| {
            column_spaced(
                20.,
                vec![
                    text_field(id!(), binding!(AppState, text))
                        .font_size(40)
                        .finish(),
                    toggle(id!(), binding!(AppState, toggle)).finish(),
                ],
            )
            .pad(20.)
        }),
    )
}
