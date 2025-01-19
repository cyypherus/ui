use haven::{
    column, column_spaced, id, rect, row, row_spaced, scope, scoper, stack, text, view, App,
    ClickState, Color, Node, Ui,
};

#[derive(Clone)]
struct UserState {
    button: Vec<ButtonState>,
}

fn main() {
    App::start(
        UserState {
            button: vec![ButtonState::new(); 100],
        },
        move |ui| {
            row((0..ui.state.button.len())
                .collect::<Vec<usize>>()
                .chunks(10)
                .enumerate()
                .map(|(c, chunk)| {
                    column(
                        chunk
                            .iter()
                            .map(|&i| {
                                let button_hovered = ui.state.button[i].hovered;
                                scoper(
                                    move |f: &mut UserState| &mut f.button[i],
                                    move |ui| {
                                        button(
                                            id!() + c.to_string().as_str() + i.to_string().as_str(),
                                            ui,
                                        )
                                    },
                                )
                                .height(if button_hovered { 200. } else { 100. })
                                .width(if button_hovered {
                                    200.
                                } else {
                                    100.
                                })
                            })
                            .collect(),
                    )
                })
                .collect())
        },
    )
}

#[derive(Debug, Clone)]
struct ButtonState {
    hovered: bool,
    depressed: bool,
}

impl ButtonState {
    fn new() -> Self {
        Self {
            hovered: false,
            depressed: false,
        }
    }
}

fn button(id: String, ui: &mut Ui<ButtonState>) -> Node<Ui<ButtonState>> {
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
                .on_hover(|state: &mut ButtonState, hovered| state.hovered = hovered)
                .on_click(|state: &mut ButtonState, click_state| match click_state {
                    ClickState::Started => state.depressed = true,
                    ClickState::Cancelled => state.depressed = false,
                    ClickState::Completed => {
                        state.depressed = false;
                    }
                }), // .transition_duration(900.)
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
