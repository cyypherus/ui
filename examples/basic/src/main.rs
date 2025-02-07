use ui::*;

#[derive(Clone, Default)]
struct AppState {
    text: String,
    button: ButtonState,
    scroller: ScrollerState,
}

fn main() {
    App::start(
        AppState {
            text: "The scale factor is calculated differently on different platforms:".to_string(),
            button: ButtonState::default(),
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
                        30.,
                        Binding::<AppState, _> {
                            get: |s| s.scroller,
                            set: |s, sc| s.scroller = sc,
                        },
                        |_state, index, _id| {
                            let id = id!(index as u64);
                            Some(
                                text(id, index.to_string())
                                    .fill(Color::WHITE)
                                    .view()
                                    .transition_duration(0.)
                                    .finish(),
                            )
                        },
                    ),
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
                                .corner_rounding(30.)
                                .stroke(Color::WHITE, 1.)
                                .finish(),
                        ),
                    button::<AppState>(
                        id!(),
                        "Clear text".to_string(),
                        Binding {
                            get: |s| s.button.depressed,
                            set: |s, d| s.button.depressed = d,
                        },
                        Binding {
                            get: |s| s.button.hovered,
                            set: |s, h| s.button.hovered = h,
                        },
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

#[derive(Debug, Clone, Default, Copy)]
struct ScrollerState {
    offset: f32,
}

impl ScrollerState {}

fn scroller<'n, State: 'static, CellFn>(
    id: u64,
    cell_height: f32,
    scroller: Binding<State, ScrollerState>,
    cell: CellFn,
) -> Node<'n, RcUi<State>>
where
    CellFn: for<'x> Fn(&'x mut State, usize, u64) -> Option<Node<'n, RcUi<State>>> + 'static + Copy,
{
    stack(vec![
        rect(id!())
            .corner_rounding(30.)
            .stroke(Color::WHITE, 1.)
            .view()
            .on_scroll(move |s, dt| {
                let mut sc = scroller.get(s);
                sc.offset += dt.y;
                sc.offset = sc.offset.min(0.);
                scroller.set(s, sc);
            })
            .finish(),
        area_reader::<RcUi<State>>(move |area, state| {
            let index = (-scroller.get(&state.ui.state).offset / cell_height) as i32;
            let window = (area.height / cell_height) as i32;
            let mut cells = Vec::new();
            for i in 0..=(window) {
                let window_index = index + i;
                if window_index >= 0 {
                    if let Some(cell) = cell(&mut state.ui.state, window_index as usize, id) {
                        cells.push(cell.height(cell_height));
                    }
                }
            }
            column(cells)
                .height(1.)
                .offset_y(scroller.get(&state.ui.state).offset % cell_height)
        })
        .expand(),
    ])
}

#[derive(Debug, Clone, Default)]
struct ButtonState {
    hovered: bool,
    depressed: bool,
}

fn button<'n, State: 'static>(
    id: u64,
    label: String,
    depressed: Binding<State, bool>,
    hovered: Binding<State, bool>,
    on_click: fn(&mut State),
) -> Node<'n, RcUi<State>> {
    dynamic_node(move |s: &mut State| {
        stack(vec![
            rect(id!(id))
                .fill(match (depressed.get(s), hovered.get(s)) {
                    (true, _) => Color::from_rgb8(50, 30, 55),
                    (false, true) => Color::from_rgb8(180, 150, 255),
                    (false, false) => Color::from_rgb8(110, 80, 255),
                })
                .corner_rounding(30.)
                .view()
                .on_hover(move |s: &mut State, h| hovered.set(s, h))
                .on_click(move |s: &mut State, click_state| match click_state {
                    ClickState::Started => depressed.set(s, true),
                    ClickState::Cancelled => depressed.set(s, false),
                    ClickState::Completed => {
                        on_click(s);
                        depressed.set(s, false);
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
