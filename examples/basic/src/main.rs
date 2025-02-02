use haven::{
    column, column_spaced, dynamic, dynamic_node, dynamic_view, id, rect, scope, scoper, space,
    stack, text, view, App, ClickState, Color, Node, RcUi, Ui,
};
use std::borrow::BorrowMut;

#[derive(Clone)]
struct UserState {
    count: i32,
    button: Vec<ButtonState<UserState>>,
}

fn main() {
    App::start(
        UserState {
            count: 0,
            button: vec![ButtonState::new(|state| state.borrow_mut().count += 1); 100],
        },
        scoper(
            |st: &mut UserState| st.button[0].clone(),
            |st: &mut UserState, b: ButtonState<UserState>| st.button[0] = b,
            button(id!()),
        )
        .width(100.)
        .aspect(1.),
        // column_spaced(
        //     10.,
        //     vec![
        //         dynamic_view(|st: &mut UserState| {
        //             text(id!(), "hiiiiii")
        //                 .fill(if st.button[0].depressed {
        //                     Color::WHITE
        //                 } else {
        //                     Color::BLACK
        //                 })
        //                 .finish()
        //         }),
        //         dynamic_node(|st: &UserState| {
        //             scoper(
        //                 |ui: &mut UserState| ui.button.get_mut(0).unwrap(),
        //                 // |ctx: backer::ScopeCtx<RcUi<ButtonState<UserState>>>,
        //                 //  ui: &mut RcUi<UserState>| {
        //                 //     let mut scoped = ui.scope_ui(|state| state.button[0].clone());
        //                 //     let result = ctx.with_scoped(&mut scoped);
        //                 //     if ui.ui.state.button[0].clicked {
        //                 //         (ui.ui.state.button[0].action)(&mut ui.ui.state);
        //                 //     }
        //                 //     ui.embed_ui(|state| &mut state.button[0], scoped);
        //                 //     result
        //                 // },
        //                 button(id!()),
        //             )
        //             .height(if st.button[0].depressed { 100. } else { 200. })
        //             .width(100.)
        //         }),
        //     ],
        // ),
    )
}

#[derive(Debug, Clone)]
struct ButtonState<F> {
    hovered: bool,
    depressed: bool,
    clicked: bool,
    action: fn(&mut F),
}

impl<F> ButtonState<F> {
    fn new(action: fn(&mut F)) -> Self {
        Self {
            hovered: false,
            depressed: false,
            clicked: false,
            action,
        }
    }
}

fn button<'n, F: 'n>(id: u64) -> Node<'n, RcUi<ButtonState<F>>> {
    dynamic_node(move |ui: &ButtonState<F>| {
        stack(vec![dynamic_view(move |ui: &mut ButtonState<F>| {
            rect(id!(id))
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
                .on_hover(|state: &mut ButtonState<F>, hovered| state.hovered = hovered)
                .on_click(
                    move |state: &mut ButtonState<F>, click_state| match click_state {
                        ClickState::Started => state.depressed = true,
                        ClickState::Cancelled => state.depressed = false,
                        ClickState::Completed => {
                            state.clicked = true;
                            state.depressed = false;
                        }
                    },
                )
        })])
        .pad({
            let mut p = 5.;
            if ui.hovered {
                p -= 10.
            }
            if ui.depressed {
                p += 10.
            }
            p
        })
    })
}
