use haven::*;

#[derive(Clone, Default)]
struct AppState {
    text: String,
    button: ButtonState,
}

fn main() {
    App::start(
        AppState {
            text: "The scale factor is calculated differently on different platforms:".to_string(),
            button: ButtonState::default(),
        },
        dynamic_node(|s: &mut AppState| {
            column_spaced(
                20.,
                vec![
                    svg(id!(), || std::fs::read("assets/tiger.svg").unwrap())
                        .finish()
                        .aspect(1.),
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
                        .attach_under(
                            rect(id!())
                                .corner_rounding(30.)
                                .stroke(Color::WHITE, 1.)
                                .finish(),
                        ),
                    button::<AppState>(
                        id!(),
                        "Clear text".to_string(),
                        |s| s.button.depressed,
                        |s, d| s.button.depressed = d,
                        |s| s.button.hovered,
                        |s, h| s.button.hovered = h,
                        |s| s.text = "".to_string(),
                    )
                    .width(200.)
                    .height(100.),
                ],
            )
            .width(400.)
        }),
    )
}

#[derive(Debug, Clone, Default)]
struct ButtonState {
    hovered: bool,
    depressed: bool,
}

fn button<'n, State: 'static>(
    id: u64,
    label: String,
    depressed: fn(&State) -> bool,
    set_depressed: fn(&mut State, bool),
    hovered: fn(&State) -> bool,
    set_hovered: fn(&mut State, bool),
    on_click: fn(&mut State),
) -> Node<'n, RcUi<State>> {
    dynamic_node(move |s: &mut State| {
        stack(vec![
            rect(id!(id))
                .fill(match (depressed(s), hovered(s)) {
                    (true, _) => Color::from_rgb8(50, 30, 55),
                    (false, true) => Color::from_rgb8(180, 150, 255),
                    (false, false) => Color::from_rgb8(110, 80, 255),
                })
                .corner_rounding(30.)
                .view()
                .on_hover(move |s: &mut State, hovered| set_hovered(s, hovered))
                .on_click(move |s: &mut State, click_state| match click_state {
                    ClickState::Started => set_depressed(s, true),
                    ClickState::Cancelled => set_depressed(s, false),
                    ClickState::Completed => {
                        on_click(s);
                        set_depressed(s, false);
                    }
                })
                .finish(),
            text(id!(id), label.clone())
                .fill(Color::WHITE)
                .font_size(30)
                .finish(),
        ])
    })
}
