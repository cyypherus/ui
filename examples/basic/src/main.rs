use haven::{
    column_spaced, id, rect, scoper, space, stack, text, view, App, ClickState, Color, Node, Ui,
};

#[derive(Clone)]
struct UserState {
    button: ButtonState,
}

fn main() {
    App::start(
        UserState {
            button: ButtonState::new(),
        },
        move |ui| {
            column_spaced(
                10.,
                vec![
                    view(ui, text(id!(), "hello!").fill(Color::WHITE).finish()),
                    scoper(
                        |ctx, ui: &mut Ui<UserState>| ui.scope_ui(ctx, |f| &mut f.button),
                        |ui: &mut Ui<ButtonState>| button(id!(), ui),
                    ),
                ],
            )
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
            rect(id)
                .stroke(Color::rgb(0.4, 0.4, 0.4), 4.)
                .fill(match (ui.state.depressed, ui.state.hovered) {
                    (_, true) => Color::rgb(0.2, 0.2, 0.2),
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
                }),
        ),
        view(ui, text(id!(), "hello!").fill(Color::WHITE).finish()),
    ])
    .width({
        let mut w = 100.;
        if ui.state.hovered {
            w += 10.
        }
        if ui.state.depressed {
            w -= 5.
        }
        w
    })
    .height({
        let mut h = 50.;
        if ui.state.hovered {
            h += 10.
        }
        if ui.state.depressed {
            h -= 5.
        }
        h
    })
}
