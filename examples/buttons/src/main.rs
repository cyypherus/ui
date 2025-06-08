use ui::*;
use vello_svg::vello::peniko::color::AlphaColor;

#[derive(Clone, Default, Debug)]
struct State {
    count: i32,
    b1: ButtonState,
    b2: ButtonState,
    b3: ButtonState,
    t1: ToggleState,
}

fn main() {
    App::start(State::default(), || {
        dynamic(|_: &mut AppState<State>| {
            row_spaced(
                20.,
                vec![
                    // toggle(id!(), binding!(State, t1)).finish(),
                    // column_spaced(
                    //     20.,
                    //     vec![
                    //         text(id!(), "Custom Label Text")
                    //             .fill(AlphaColor::WHITE)
                    //             .finish(),
                    //         dynamic(|s: &mut AppState<State>| {
                    //             button(id!(), binding!(State, b1))
                    //                 .text_label(format!("Click count {}", s.state.count))
                    //                 .on_click(|s| s.state.count += 1)
                    //                 .finish()
                    //         }),
                    //     ],
                    // )
                    // .height(150.)
                    // .width(200.),
                    column_spaced(
                        20.,
                        vec![
                            text(id!(), "Custom Body").fill(AlphaColor::WHITE).finish(),
                            button(id!(), binding!(State, b2))
                                .on_click(|s| s.state.count += 1)
                                .body(|button| {
                                    rect(id!())
                                        .fill({
                                            dbg!(button);
                                            let c = match (button.hovered, button.depressed) {
                                                (_, true) => AlphaColor::from_rgb8(100, 30, 30),
                                                (true, false) => AlphaColor::from_rgb8(230, 30, 30),
                                                (false, false) => {
                                                    AlphaColor::from_rgb8(200, 30, 30)
                                                }
                                            };
                                            dbg!(c);
                                            c
                                        })
                                        .corner_rounding(40.)
                                        .finish()
                                })
                                .finish(),
                        ],
                    )
                    .height(150.)
                    .width(200.),
                    // column_spaced(
                    //     20.,
                    //     vec![
                    //         text(id!(), "Svg Label").fill(AlphaColor::WHITE).finish(),
                    //         button(id!(), binding!(State, b3))
                    //             .on_click(|s| s.state.count += 1)
                    //             .label(|button| {
                    //                 svg(id!(), "assets/download.svg")
                    //                     .fill(match (button.depressed, button.hovered) {
                    //                         (true, _) => AlphaColor::from_rgb8(190, 190, 190),
                    //                         (false, true) => AlphaColor::from_rgb8(250, 250, 250),
                    //                         (false, false) => AlphaColor::from_rgb8(240, 240, 240),
                    //                     })
                    //                     .finish()
                    //                     .pad(15.)
                    //             })
                    //             .finish(),
                    //     ],
                    // )
                    // .height(150.)
                    // .width(200.),
                ],
            )
        })
    })
}
