use ui::*;
use vello_svg::vello::peniko::color::AlphaColor;

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
            ..Default::default()
        },
        dynamic_node(|_: &mut AppState| {
            column_spaced(
                20.,
                vec![
                    text_field(id!(), binding!(AppState, text))
                        .fill(AlphaColor::WHITE)
                        .font_size(40)
                        .finish()
                        .pad(100.),
                    toggle(id!(), binding!(AppState, toggle)).finish(),
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
        }),
    )
}
