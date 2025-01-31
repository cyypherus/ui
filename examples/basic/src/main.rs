use haven::{
    column, column_spaced, dynamic, dynamic_node, dynamic_view, id, rect, scope, space, stack,
    text, view, App, ClickState, Color, Node, RcUi,
};
use std::cell::RefMut;

#[derive(Clone)]
struct UserState {
    count: i32,
    button: Vec<ButtonState<Self>>,
}

fn main() {
    let s = scope(
        |ctx: backer::ScopeCtx<RcUi<ButtonState<UserState>>>, ui: &mut RcUi<UserState>| {
            let mut scoped = ui.scope_ui(|state| state.button[0].clone());
            let result = ctx.with_scoped(&mut scoped);
            ui.embed_ui(|state| &mut state.button[0], scoped);
            result
        },
        button(id!()),
    );
    App::start(
        UserState {
            count: 0,
            button: vec![ButtonState::new(|state| state.count += 1); 100],
        },
        column_spaced(
            10.,
            vec![
                dynamic_view(|st: RefMut<'_, UserState>| {
                    text(id!(), "hiiiiii")
                        .fill(if st.button[0].depressed {
                            Color::WHITE
                        } else {
                            Color::BLACK
                        })
                        .finish()
                }),
                dynamic_node(|st: RefMut<'_, UserState>| {
                    scope(
                        |ctx: backer::ScopeCtx<RcUi<ButtonState<UserState>>>,
                         ui: &mut RcUi<UserState>| {
                            let mut scoped = ui.scope_ui(|state| state.button[0].clone());
                            let result = ctx.with_scoped(&mut scoped);
                            ui.embed_ui(|state| &mut state.button[0], scoped);
                            result
                        },
                        button(id!()),
                    )
                    .height(if st.button[0].depressed { 100. } else { 200. })
                    .width(100.)
                }),
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
    dynamic_node(move |ui: RefMut<ButtonState<T>>| {
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
            view(move || {
                rect(id!(id))
                    .fill(Color::rgb(0.1, 0.1, 0.1))
                    .corner_rounding(5.)
                    .finish()
            })
            .width(10.)
            .height(20.)
            .pad_top(20.)
            .offset_x(-10.)
            .align(haven::Align::Top),
            view(move || {
                rect(id!(id))
                    .fill(Color::rgb(0.1, 0.1, 0.1))
                    .corner_rounding(5.)
                    .finish()
            })
            .width(10.)
            .height(20.)
            .pad_top(20.)
            .offset_x(10.)
            .align(haven::Align::Top),
            view(move || {
                rect(id!(id))
                    .fill(Color::rgb(0.1, 0.1, 0.1))
                    .corner_rounding(5.)
                    .finish()
            })
            .width(10.)
            .height(10.)
            .pad_top(40.)
            .align(haven::Align::Top),
        ])
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
