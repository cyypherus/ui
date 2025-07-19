use ui::*;

#[derive(Clone, Default)]
struct State {
    text: TextState,
}

fn main() {
    App::builder(
        State {
            text: TextState {
                text: "000000".to_string(),
                editing: false,
            },
        },
        || {
            dynamic(|_, _: &mut AppState<State>| {
                column_spaced(
                    20.,
                    vec![
                        space(),
                        row(vec![
                            text(id!(), "#").font_size(40).finish().width(20.),
                            space().width(10.),
                            text_field(id!(), binding!(State, text))
                                .font_size(40)
                                // .background_fill(None)
                                // .no_background_stroke()
                                .finish(),
                        ])
                        .align_contents(Align::CenterY),
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
