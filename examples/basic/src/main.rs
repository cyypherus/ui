use std::{
    borrow::Borrow,
    cell::{Ref, RefCell},
};

use haven::{
    column, dynamic, id, rect, row, scope, scoper, space, stack, text, view, App, ClickState,
    Color, Node, RcUi, Ui,
};

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
        dynamic(|ui: &mut RcUi<UserState>| {
            column(vec![
                view(|_| {
                    rect(id!())
                        .stroke(Color::rgb(1., 0., 0.), 5.)
                        .fill(Color::rgb(1., 0., 0.))
                        .finish()
                })
                .width(100.)
                .height(100.),
                scoper(
                    move |state: &mut UserState| state.button[0].clone(),
                    move |state: &mut UserState, substate: ButtonState<UserState>| {
                        state.button[0] = substate
                    },
                    move |_ui: &mut RcUi<ButtonState<UserState>>| button(id!()),
                )
                .height(100.)
                .width(100.),
                // view(
                //     ui.clone(),
                //     text(id!(), c).fill(Color::WHITE).finish().on_hover(
                //         |state: &mut UserState, _| {
                //             state.count += 1;
                //         },
                //     ),
                // )
                // .width(100.)
                // .height(100.),
            ])

            // column(vec![
            // view(
            //     ui,
            //     text(id!(), format!("{}", ui.state.count))
            //         .fill(Color::WHITE)
            //         .finish(),
            // )
            // .pad(100.)
            // scoper(
            //     move |f: &mut UserState| {
            //         let child_cx = self.cx.take();
            //         Ui {
            //             state: state.button,
            //             gesture_handlers: Vec::new(),
            //             cx: child_cx,
            //         }
            // f.button[0].clone()
            // },
            // move |e: &mut UserState, mut f: ButtonState<UserState>| {
            //     if f.clicked {
            //         (f.action)(e);
            //         f.clicked = false;
            //     }
            //     e.button[0] = f;
            // },
            //     move |ui| button(id!(), ui),
            // )
            // .width(100.)
            // .height(100.),
            // ])
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
        }),
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

fn button<'n, T: Clone + 'n>(id: String) -> Node<'n, RcUi<ButtonState<T>>> {
    let id_a = id.clone();
    stack(vec![
        view(move |ui: &mut RcUi<ButtonState<T>>| {
            rect(id_a.clone())
                .stroke(
                    {
                        match (ui.borrow_state().depressed, ui.borrow_state().hovered) {
                            (_, true) => Color::rgb(0.9, 0.9, 0.9),
                            (true, false) => Color::rgb(0.8, 0.3, 0.3),
                            (false, false) => Color::rgb(0.1, 0.1, 0.1),
                        }
                    },
                    4.,
                )
                .fill(
                    match (ui.borrow_state().depressed, ui.borrow_state().hovered) {
                        (_, true) => Color::rgb(0.9, 0.2, 0.2),
                        (true, false) => Color::rgb(0.3, 0.3, 0.3),
                        (false, false) => Color::rgb(0.1, 0.1, 0.1),
                    },
                )
                .corner_rounding(if ui.borrow_state().hovered { 25. } else { 20. })
                .finish()
                .on_hover(|state: &mut ButtonState<T>, hovered| {
                    dbg!("??");
                    state.hovered = hovered
                })
                .on_click(|state: &mut ButtonState<T>, click_state| {
                    dbg!("??");
                    match click_state {
                        ClickState::Started => state.depressed = true,
                        ClickState::Cancelled => state.depressed = false,
                        ClickState::Completed => {
                            state.clicked = true;
                            state.depressed = false;
                        }
                    }
                })
        }),
        view(move |_| {
            text(id!() + id.as_str(), "hello!")
                .fill(Color::WHITE)
                .finish()
        }),
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
