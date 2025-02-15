use ui::*;
use vello_svg::vello::peniko::color::AlphaColor;

#[derive(Clone, Default)]
struct AppState {
    count: i32,
    b1: ButtonState,
    b2: ButtonState,
    b3: ButtonState,
}

fn main() {
    App::start(
        AppState::default(),
        dynamic_node(|s: &mut AppState| {
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
                                .body(
                                    rect(id!())
                                        .fill({
                                            match (s.b2.hovered, s.b2.depressed) {
                                                (_, true) => AlphaColor::from_rgb8(100, 30, 30),
                                                (true, false) => AlphaColor::from_rgb8(230, 30, 30),
                                                (false, false) => {
                                                    AlphaColor::from_rgb8(200, 30, 30)
                                                }
                                            }
                                        })
                                        .corner_rounding(40.)
                                        .view(),
                                )
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
                                .label(svg(id!(), "assets/download.svg").view())
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
