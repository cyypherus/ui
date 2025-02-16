use ui::*;
use vello_svg::vello::peniko::color::AlphaColor;

#[derive(Clone, Default, Debug)]
struct AppState {
    count: i32,
    b1: ButtonState,
    b2: ButtonState,
    b3: ButtonState,
}

fn main() {
    App::start(
        AppState::default(),
        dynamic_node(|_: &mut AppState| {
            row_spaced(
                20.,
                vec![
                    column_spaced(
                        20.,
                        vec![
                            text(id!(), "Custom Label Text")
                                .fill(AlphaColor::WHITE)
                                .finish(),
                            dynamic_node(|s: &mut AppState| {
                                button(id!(), binding!(AppState, b1))
                                    .text_label(format!("Click count {}", s.count))
                                    .on_click(|s| s.count += 1)
                                    .finish()
                            }),
                        ],
                    )
                    .height(150.)
                    .width(200.),
                    column_spaced(
                        20.,
                        vec![
                            text(id!(), "Custom Body").fill(AlphaColor::WHITE).finish(),
                            button(id!(), binding!(AppState, b2))
                                .on_click(|s| s.count += 1)
                                .body(|button| {
                                    rect(id!())
                                        .fill({
                                            match (button.hovered, button.depressed) {
                                                (_, true) => AlphaColor::from_rgb8(100, 30, 30),
                                                (true, false) => AlphaColor::from_rgb8(230, 30, 30),
                                                (false, false) => {
                                                    AlphaColor::from_rgb8(200, 30, 30)
                                                }
                                            }
                                        })
                                        .corner_rounding(40.)
                                        .finish()
                                })
                                .finish(),
                        ],
                    )
                    .height(150.)
                    .width(200.),
                    column_spaced(
                        20.,
                        vec![
                            text(id!(), "Svg Label").fill(AlphaColor::WHITE).finish(),
                            button(id!(), binding!(AppState, b3))
                                .on_click(|s| s.count += 1)
                                .label(|button| {
                                    svg(id!(), "assets/download.svg")
                                        .fill(match (button.depressed, button.hovered) {
                                            (true, _) => AlphaColor::from_rgb8(190, 190, 190),
                                            (false, true) => AlphaColor::from_rgb8(250, 250, 250),
                                            (false, false) => AlphaColor::from_rgb8(240, 240, 240),
                                        })
                                        .finish()
                                        .pad(15.)
                                })
                                .finish(),
                        ],
                    )
                    .height(150.)
                    .width(200.),
                ],
            )
        }),
    )
}
