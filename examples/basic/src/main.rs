use haven::*;

#[derive(Clone, Default)]
struct AppState {
    text: String,
}

fn main() {
    App::start(
        AppState {
            text: "The scale factor is calculated differently on different platforms:".to_string(),
        },
        dynamic_node(|s: &mut AppState| {
            row(vec![
                space(),
                text(id!(), s.text.clone())
                    .fill(Color::WHITE)
                    .align(TextAlign::Leading)
                    .view()
                    .on_key(|s: &mut AppState, key| match key {
                        Key::Named(NamedKey::Space) => s.text.push(' '),
                        Key::Named(NamedKey::Enter) => s.text.push('\n'),
                        Key::Named(NamedKey::Backspace) => {
                            s.text.pop();
                        }
                        Key::Character(c) => s.text.extend(c.as_str().chars()),
                        Key::Named(_) => (),
                    })
                    .finish()
                    .attach_under(rect(id!()).stroke(Color::WHITE, 1.).view().finish()),
                space(),
            ])
        }),
    )
}

// svg(id!(), || std::fs::read("assets/tiger.svg").unwrap())
//     .view()
//     .on_hover(|s: &mut AppState, hovered| s.hovered = hovered)
//     .finish()
//     .width(if s.hovered { 550. } else { 500. })
//     .height(if s.hovered { 550. } else { 500. })
// scoper(
//     move |st: &mut UserState| st.button[i].clone(),
//     move |st: &mut UserState, b: ButtonState<i32>| st.button[i] = b,
//     button(id!(i as u64)),
// ),

// #[derive(Debug, Clone)]
// struct ButtonState<F> {
//     hovered: bool,
//     depressed: bool,
//     clicked: bool,
//     state: F,
//     action: fn(&mut F),
// }

// impl<F> ButtonState<F> {
//     fn new(state: F, action: fn(&mut F)) -> Self {
//         Self {
//             hovered: false,
//             depressed: false,
//             clicked: false,
//             state,
//             action,
//         }
//     }
// }

// fn button<'n, F: 'n>(id: u64) -> Node<'n, RcUi<ButtonState<F>>> {
//     dynamic_node(move |ui: &mut ButtonState<F>| {
//         stack(vec![rect(id!(id))
//             .stroke(
//                 {
//                     match (ui.depressed, ui.hovered) {
//                         (_, true) => Color::from_rgb8(90, 90, 90),
//                         (true, false) => Color::from_rgb8(80, 30, 30),
//                         (false, false) => Color::from_rgb8(10, 10, 10),
//                     }
//                 },
//                 1.,
//             )
//             .fill(match (ui.depressed, ui.hovered) {
//                 (true, true) => Color::from_rgb8(60, 5, 5),
//                 (false, true) => Color::from_rgb8(90, 20, 20),
//                 (true, false) => Color::from_rgb8(30, 30, 30),
//                 (false, false) => Color::from_rgb8(10, 10, 10),
//             })
//             .corner_rounding(if ui.hovered { 10. } else { 5. })
//             .view()
//             .on_hover(|state: &mut ButtonState<F>, hovered| state.hovered = hovered)
//             .on_click(
//                 move |state: &mut ButtonState<F>, click_state| match click_state {
//                     ClickState::Started => state.depressed = true,
//                     ClickState::Cancelled => state.depressed = false,
//                     ClickState::Completed => {
//                         state.clicked = true;
//                         (state.action)(&mut state.state);
//                         state.depressed = false;
//                     }
//                 },
//             )
//             .finish()])
//     })
// }
