use ui::*;

#[derive(Clone)]
struct AppState {
    text: String,
    editor: Editor,
}

fn main() {
    App::start(
        AppState {
            text: "The scale factor is calculated differently on different platforms:".to_string(),
            editor: Editor::new("2025-02-15 20:49:37.749 basic[41538:11281138] +[IMKClient subclass]: chose IMKClient_Legacy
            2025-02-15 20:49:37.749 basic[41538:11281138] +[IMKInputSession subclass]: chose IMKInputSession_Legacy"),
        },
        dynamic_node(|s: &mut AppState| {
            column_spaced(
                20.,
                vec![
                    text_field(id!(), binding!(AppState, editor)).finish().width(200.),
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
