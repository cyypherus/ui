use ui::*;
use vello_svg::vello::peniko::color::{AlphaColor, palette::css::RED};

#[derive(Clone, Default, Debug)]
struct State {
    count: i32,
    b1: ButtonState,
    b2: ButtonState,
    b3: ButtonState,
    // t1: ToggleState,
}

fn main() {
    App::start(State::default(), |state, app| {
        row_spaced(
            20.,
            vec![
                column_spaced(
                    20.,
                    vec![
                        text(id!(), "Custom Label Text")
                            .fill(Color::WHITE)
                            .finish(app),
                        // toggle(id!(), binding!(State, t1)).finish().height(80.),
                    ],
                )
                .height(150.)
                .width(200.),
                column_spaced(
                    20.,
                    vec![
                        text(id!(), "Custom Label Text")
                            .fill(Color::WHITE)
                            .finish(app),
                        button(id!(), binding!(State, b1))
                            .text_label(format!("Click count {}", state.count))
                            .on_click(|s, _| s.count += 1)
                            .finish(state, app),
                    ],
                )
                .height(150.)
                .width(200.),
                column_spaced(
                    20.,
                    vec![
                        text(id!(), "Custom Body").fill(Color::WHITE).finish(app),
                        button(id!(), binding!(State, b2))
                            .on_click(|s, _| s.count += 1)
                            .surface(
                                rect(id!())
                                    .fill({
                                        match (state.b2.hovered, state.b2.depressed) {
                                            (_, true) => AlphaColor::from_rgb8(100, 30, 30),
                                            (true, false) => AlphaColor::from_rgb8(230, 30, 30),
                                            (false, false) => AlphaColor::from_rgb8(200, 30, 30),
                                        }
                                    })
                                    .corner_rounding(40.)
                                    .finish(app),
                            )
                            .finish(state, app),
                    ],
                )
                .height(150.)
                .width(200.),
                column_spaced(
                    20.,
                    vec![
                        text(id!(), "Svg Label").fill(Color::WHITE).finish(app),
                        button(id!(), binding!(State, b3))
                            .on_click(|s, _| s.count += 1)
                            .label(
                                svg(id!(), include_str!("../../../assets/download.svg"))
                                    .fill(match (state.b3.depressed, state.b3.hovered) {
                                        (true, _) => AlphaColor::from_rgb8(190, 190, 190),
                                        (false, true) => AlphaColor::from_rgb8(250, 250, 250),
                                        (false, false) => AlphaColor::from_rgb8(240, 240, 240),
                                    })
                                    .finish(app)
                                    .pad(15.),
                            )
                            .finish(state, app),
                    ],
                )
                .height(150.)
                .width(200.),
            ],
        )
    })
}
