use ui::*;

#[derive(Clone)]
struct AppState {
    text: TextState,
}

fn main() {
    App::start(
        AppState {
            text: TextState {
                text: "The scale factor is calculated differently on different platforms:"
                    .to_string(),
                editing: false,
            },
        },
        dynamic_node(|_: &mut AppState| {
            column_spaced(
                20.,
                vec![
                    text_field(id!(), binding!(AppState, text))
                        .font_size(40)
                        .fill(Color::from_rgb8(0, 200, 200))
                        .finish()
                        .width(200.)
                        .attach_under(rect(id!()).stroke(Color::WHITE, 2.).finish()),
                    // svg(id!(), "assets/tiger.svg").finish().aspect(1.),
                    // text(id!(), s.text.clone())
                    //     .fill(Color::WHITE)
                    //     .font_size(30)
                    //     .align(TextAlign::Leading)
                    //     .view()
                    //     .on_key(|s: &mut AppState, key| match key {
                    //         Key::Named(NamedKey::Space) => s.text.push(' '),
                    //         Key::Named(NamedKey::Enter) => s.text.push('\n'),
                    //         Key::Named(NamedKey::Backspace) => {
                    //             s.text.pop();
                    //         }
                    //         Key::Character(c) => s.text.push_str(c.as_str()),
                    //         Key::Named(_) => (),
                    //     })
                    //     .finish()
                    //     .pad_x(30.)
                    //     .pad_y(10.)
                    //     .attach_under(
                    //         rect(id!())
                    //             .corner_rounding(10.)
                    //             .stroke(Color::WHITE, 1.)
                    //             .finish(),
                    //     ),
                ],
            )
            .width(600.)
        }),
    )
}
