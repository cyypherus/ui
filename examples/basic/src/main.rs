use haven::{column, id, rect, row, scoper, stack, text, view, App, ClickState, Color, Node, Ui};

#[derive(Clone)]
struct UserState {
    count: i32,
    button: Vec<ButtonState<Self>>,
}

fn main() {
    App::start(
        UserState {
            count: 0,
            button: vec![ButtonState::new(|state| state.count += 1); 100],
        },
        move |ui| {
            column(vec![
                view(
                    ui,
                    text(id!(), format!("{}", ui.state.count))
                        .fill(Color::WHITE)
                        .finish(),
                )
                .pad(100.),
                scoper(
                    move |f: &mut UserState| f.button[0].clone(),
                    move |e: &mut UserState, mut f: ButtonState<UserState>| {
                        if f.clicked {
                            (f.action)(e);
                            f.clicked = false;
                        }
                        e.button[0] = f;
                    },
                    move |ui| button(id!(), ui),
                )
                .width(100.)
                .height(100.),
            ])
            // row((0..ui.state.button.len())
            //     .collect::<Vec<usize>>()
            //     .chunks(10)
            //     .enumerate()
            //     .map(|(c, chunk)| {
            //         column(
            //             chunk
            //                 .iter()
            //                 .map(|&i| {
            //                     let button_hovered = ui.state.button[i].hovered;
            //                     scoper(
            //                         move |f: &mut UserState| f.button[i].clone(),
            //                         move |e: &mut UserState, mut f: ButtonState<UserState>| {
            //                             if f.clicked {
            //                                 (f.action)(e);
            //                                 f.clicked = false;
            //                             }
            //                             e.button[i] = f;
            //                         },
            //                         move |ui| {
            //                             button(
            //                                 id!() + c.to_string().as_str() + i.to_string().as_str(),
            //                                 ui,
            //                             )
            //                         },
            //                     )
            //                     .height(if button_hovered { 200. } else { 100. })
            //                     .width(if button_hovered {
            //                         200.
            //                     } else {
            //                         100.
            //                     })
            //                 })
            //                 .collect(),
            //         )
            //     })
            //     .collect())
        },
    )
}

#[derive(Debug, Clone)]
struct ButtonState<T> {
    hovered: bool,
    depressed: bool,
    clicked: bool,
    action: fn(&mut T),
}

impl<T> ButtonState<T> {
    fn new(action: fn(&mut T)) -> Self {
        Self {
            hovered: false,
            depressed: false,
            clicked: false,
            action,
        }
    }
}

fn button<T: 'static>(id: String, ui: &mut Ui<ButtonState<T>>) -> Node<Ui<ButtonState<T>>> {
    stack(vec![
        view(
            ui,
            rect(id.clone())
                .stroke(
                    match (ui.state.depressed, ui.state.hovered) {
                        (_, true) => Color::rgb(0.9, 0.9, 0.9),
                        (true, false) => Color::rgb(0.8, 0.3, 0.3),
                        (false, false) => Color::rgb(0.1, 0.1, 0.1),
                    },
                    4.,
                )
                .fill(match (ui.state.depressed, ui.state.hovered) {
                    (_, true) => Color::rgb(0.9, 0.2, 0.2),
                    (true, false) => Color::rgb(0.3, 0.3, 0.3),
                    (false, false) => Color::rgb(0.1, 0.1, 0.1),
                })
                .corner_rounding(if ui.state.hovered { 25. } else { 20. })
                .finish()
                .on_hover(|state: &mut ButtonState<T>, hovered| state.hovered = hovered)
                .on_click(
                    |state: &mut ButtonState<T>, click_state| match click_state {
                        ClickState::Started => state.depressed = true,
                        ClickState::Cancelled => state.depressed = false,
                        ClickState::Completed => {
                            state.clicked = true;
                            state.depressed = false;
                        }
                    },
                ),
            // .transition_duration(900.)
            // .easing(lilt::Easing::EaseOutCirc),
        ),
        view(
            ui,
            text(id!() + id.as_str(), "hello!")
                .fill(Color::WHITE)
                .finish(),
        ),
    ])
    .pad({
        let mut p = 5.;
        if ui.state.hovered {
            p -= 10.
        }
        if ui.state.depressed {
            p += 10.
        }
        p
    })
}
