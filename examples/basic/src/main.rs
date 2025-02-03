use haven::*;
// use haven::{
//     dynamic, dynamic_node, dynamic_view, id, rect, row_spaced, scoper, stack, svg, text, view, App,
//     ClickState, Color, Node, RcUi,
// };

#[derive(Clone, Default)]
struct AppState {
    hovered: bool,
}

fn main() {
    App::start(
        AppState::default(),
        dynamic_node(|s: &mut AppState| {
            view(|| {
                svg(id!(), || std::fs::read("assets/tiger.svg").unwrap())
                    .finish()
                    .on_hover(|s: &mut AppState, hovered| s.hovered = hovered)
            })
            .width(if s.hovered { 550. } else { 500. })
            .height(if s.hovered { 550. } else { 500. })
            .attach_under(dynamic_view(|s: &mut AppState| {
                rect(id!())
                    .stroke(
                        if s.hovered {
                            Color::from_rgb8(200, 200, 200)
                        } else {
                            Color::from_rgb8(100, 100, 100)
                        },
                        if s.hovered { 9. } else { 3. },
                    )
                    .corner_rounding(30.)
                    .finish()
            }))
        }),
    )
}

// scoper(
//     move |st: &mut UserState| st.button[i].clone(),
//     move |st: &mut UserState, b: ButtonState<i32>| st.button[i] = b,
//     button(id!(i as u64)),
// ),

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
