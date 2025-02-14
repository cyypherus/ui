use ui::*;

#[derive(Clone, Default)]
struct AppState {
    text: String,
    button: Vec<ButtonState>,
    scroller: ScrollerState,
}

fn main() {
    App::start(
        AppState {
            text: "The scale factor is calculated differently on different platforms:".to_string(),
            button: vec![ButtonState::default(); 40],
            scroller: ScrollerState::default(),
        },
        dynamic_node(|s: &mut AppState| {
            column_spaced(
                20.,
                vec![
                    svg(id!(), || std::fs::read("assets/tiger.svg").unwrap())
                        .finish()
                        .aspect(1.),
                    scroller(
                        id!(),
                        Binding::<AppState, ScrollerState>::new(
                            |s: &AppState| s.scroller.clone(),
                            |s: &mut AppState, sc| s.scroller = sc,
                        ),
                        |state, index, _id| {
                            if state.button.get(index).is_some() {
                                let id = id!(index as u64);
                                Some(
                                    button::<AppState>(
                                        id,
                                        "Clear text".to_string(),
                                        Binding::new(
                                            move |s: &AppState| s.button[index],
                                            move |s: &mut AppState, d| s.button[index] = d,
                                        ),
                                        |s| s.text = "".to_string(),
                                    )
                                    .height(if index % 3 == 0 { 80. } else { 50. })
                                    .pad(5.),
                                )
                            } else {
                                None
                            }
                        },
                    )
                    .height(300.),
                    text(id!(), s.text.clone())
                        .fill(Color::WHITE)
                        .font_size(30)
                        .align(TextAlign::Leading)
                        .view()
                        .on_key(|s: &mut AppState, key| match key {
                            Key::Named(NamedKey::Space) => s.text.push(' '),
                            Key::Named(NamedKey::Enter) => s.text.push('\n'),
                            Key::Named(NamedKey::Backspace) => {
                                s.text.pop();
                            }
                            Key::Character(c) => s.text.push_str(c.as_str()),
                            Key::Named(_) => (),
                        })
                        .finish()
                        .pad_x(30.)
                        .pad_y(10.)
                        .attach_under(
                            rect(id!())
                                .corner_rounding(10.)
                                .stroke(Color::WHITE, 1.)
                                .finish(),
                        ),
                ],
            )
            .width(400.)
        }),
    )
}

#[derive(Debug, Clone, Copy, Default)]
struct ButtonState {
    hovered: bool,
    depressed: bool,
}

fn button<'n, State: 'static>(
    id: u64,
    label: String,
    state: Binding<State, ButtonState>,
    on_click: fn(&mut State),
) -> Node<'n, RcUi<State>> {
    dynamic_node(move |s: &mut State| {
        stack(vec![
            rect(id!(id))
                .fill(match (state.get(s).depressed, state.get(s).hovered) {
                    (true, _) => Color::from_rgb8(50, 30, 55),
                    (false, true) => Color::from_rgb8(180, 150, 255),
                    (false, false) => Color::from_rgb8(110, 80, 255),
                })
                .corner_rounding(10.)
                .view()
                .on_hover({
                    let binding = state.clone();
                    move |s: &mut State, h| binding.update(s, |s| s.hovered = h)
                })
                .on_click({
                    let binding = state.clone();
                    move |s: &mut State, click_state| match click_state {
                        ClickState::Started => binding.update(s, |s| s.depressed = true),
                        ClickState::Cancelled => binding.update(s, |s| s.depressed = false),
                        ClickState::Completed => {
                            on_click(s);
                            binding.update(s, |s| s.depressed = false)
                        }
                    }
                })
                .transition_duration(0.)
                .finish(),
            text(id!(id), label.clone())
                .fill(Color::WHITE)
                .font_size(30)
                .view()
                .transition_duration(0.)
                .finish(),
        ])
    })
}
