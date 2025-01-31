use haven::{
    column, column_spaced, dynamic, dynamic_view, id, rect, stack, text, view, App, ClickState,
    Color, Node, RcUi,
};
use std::cell::RefMut;

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
        column_spaced(
            10.,
            vec![
                dynamic_view(|state: RefMut<'_, UserState>| {
                    text(id!(), state.count.to_string())
                        .fill(Color::WHITE)
                        .finish()
                }),
                view(|| {
                    rect(id!())
                        .stroke(Color::rgb(1., 0., 0.), 5.)
                        .fill(Color::rgb(1., 0., 0.))
                        .finish()
                        .on_hover(|s: &mut UserState, hovered| {
                            if hovered {
                                s.count += 1
                            }
                        })
                })
                .width(100.)
                .height(100.),
            ],
        ),
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

fn button<'n, T: Clone + 'n>(id: u64) -> Node<'n, RcUi<ButtonState<T>>> {
    let id_a = id.clone();
    stack(vec![
        dynamic_view(move |ui: RefMut<ButtonState<T>>| {
            rect(id_a.clone())
                .stroke(
                    {
                        match (ui.depressed, ui.hovered) {
                            (_, true) => Color::rgb(0.9, 0.9, 0.9),
                            (true, false) => Color::rgb(0.8, 0.3, 0.3),
                            (false, false) => Color::rgb(0.1, 0.1, 0.1),
                        }
                    },
                    4.,
                )
                .fill(match (ui.depressed, ui.hovered) {
                    (_, true) => Color::rgb(0.9, 0.2, 0.2),
                    (true, false) => Color::rgb(0.3, 0.3, 0.3),
                    (false, false) => Color::rgb(0.1, 0.1, 0.1),
                })
                .corner_rounding(if ui.hovered { 25. } else { 20. })
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
                )
        }),
        // dynamic(move |ui| view(ui, text(id!(id), "hello!").fill(Color::WHITE).finish())),
    ])
    // .pad({
    //     let mut p = 5.;
    //     if ui.borrow_state().hovered {
    //         p -= 10.
    //     }
    //     if ui.borrow_state().depressed {
    //         p += 10.
    //     }
    //     p
    // })
}
