use haven::{
    dynamic, dynamic_node, dynamic_view, id, rect, row_spaced, scoper, stack, text, App,
    ClickState, Color, Node, RcUi,
};

#[derive(Clone)]
struct UserState {
    button: Vec<ButtonState<i32>>,
}

fn main() {
    App::start(
        UserState {
            button: vec![ButtonState::new(0, |state| *state += 1); 100],
        },
        dynamic(|_| {
            row_spaced(
                10.,
                (0..30)
                    .map(|i| {
                        dynamic_node(move |st: &UserState| {
                            stack(vec![
                                scoper(
                                    move |st: &mut UserState| st.button[i].clone(),
                                    move |st: &mut UserState, b: ButtonState<i32>| st.button[i] = b,
                                    button(id!(i as u64)),
                                ),
                                dynamic_view(move |st: &mut UserState| {
                                    text(id!(i as u64), st.button[i].state.to_string())
                                        .fill(Color::WHITE)
                                        .finish()
                                }),
                            ])
                            .width((10 * (st.button[i].state + 3)) as f32)
                        })
                    })
                    .collect::<Vec<_>>(),
            )
            .height(100.)
        }),
    )
}

#[derive(Debug, Clone)]
struct ButtonState<F> {
    hovered: bool,
    depressed: bool,
    clicked: bool,
    state: F,
    action: fn(&mut F),
}

impl<F> ButtonState<F> {
    fn new(state: F, action: fn(&mut F)) -> Self {
        Self {
            hovered: false,
            depressed: false,
            clicked: false,
            state,
            action,
        }
    }
}

fn button<'n, F: 'n>(id: u64) -> Node<'n, RcUi<ButtonState<F>>> {
    stack(vec![dynamic_view(move |ui: &mut ButtonState<F>| {
        rect(id!(id))
            .stroke(
                {
                    match (ui.depressed, ui.hovered) {
                        (_, true) => Color::from_rgb8(90, 90, 90),
                        (true, false) => Color::from_rgb8(80, 30, 30),
                        (false, false) => Color::from_rgb8(10, 10, 10),
                    }
                },
                1.,
            )
            .fill(match (ui.depressed, ui.hovered) {
                (true, true) => Color::from_rgb8(60, 5, 5),
                (false, true) => Color::from_rgb8(90, 20, 20),
                (true, false) => Color::from_rgb8(30, 30, 30),
                (false, false) => Color::from_rgb8(10, 10, 10),
            })
            .corner_rounding(if ui.hovered { 10. } else { 5. })
            .finish()
            .on_hover(|state: &mut ButtonState<F>, hovered| state.hovered = hovered)
            .on_click(
                move |state: &mut ButtonState<F>, click_state| match click_state {
                    ClickState::Started => state.depressed = true,
                    ClickState::Cancelled => state.depressed = false,
                    ClickState::Completed => {
                        state.clicked = true;
                        (state.action)(&mut state.state);
                        state.depressed = false;
                    }
                },
            )
    })])
}
