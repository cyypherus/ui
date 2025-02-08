use ui::*;
use vello_svg::vello::peniko::color::AlphaColor;

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
                    // svg(id!(), || std::fs::read("assets/tiger.svg").unwrap())
                    //     .finish()
                    //     .aspect(1.),
                    scroller(
                        id!(),
                        Binding::<AppState, _> {
                            get: |s| s.scroller.clone(),
                            set: |s, sc| s.scroller = sc,
                        },
                        |_state, index, _id| {
                            // if index < 10 {
                            let id = id!(index as u64);
                            Some(
                                text(id!(id), index.to_string())
                                    .fill(Color::WHITE)
                                    .view()
                                    .transition_duration(0.)
                                    .finish()
                                    .height(if index % 2 == 0 { 20. } else { 35. })
                                    .attach_under(
                                        rect(id!(id))
                                            .stroke(Color::WHITE, 1.)
                                            .view()
                                            .transition_duration(0.)
                                            .finish(),
                                    ),
                            )
                            // } else {
                            //     None
                            // }
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

#[derive(Debug, Clone, Default)]
struct ScrollerState {
    visible_window: Vec<Element>,
    dt: f32,
}

#[derive(Debug, Clone, Copy, Default)]
struct Element {
    height: f32,
    index: usize,
}

impl ScrollerState {
    fn update_visible_window<'n, State, CellFn>(
        &mut self,
        available_area: Area,
        state: &mut RcUi<State>,
        id: u64,
        cell: CellFn,
    ) where
        CellFn:
            for<'x> Fn(&'x mut State, usize, u64) -> Option<Node<'n, RcUi<State>>> + 'static + Copy,
    {
        let mut current_height = self.visible_window.iter().fold(0., |acc, e| acc + e.height);

        let overflow_amount = available_area.height * 0.25;

        while current_height < available_area.height + overflow_amount {
            let index = self.visible_window.last().map(|l| l.index).unwrap_or(0) + 1;
            if let Some(mut element) = cell(&mut state.ui.state, index, id) {
                let added_height = element.min_height(available_area, state).unwrap_or(0.);
                current_height += added_height;
                self.visible_window.push(Element {
                    height: added_height,
                    index,
                });
            } else {
                break;
            }
        }

        if self.dt.is_sign_negative() {
            let mut compensated_dt = 0.;
            while compensated_dt > self.dt {
                let index = self.visible_window.last().map(|l| l.index).unwrap_or(0) + 1;
                if let Some(mut element) = cell(&mut state.ui.state, index, id) {
                    let added_height = element.min_height(available_area, state).unwrap_or(0.);
                    self.visible_window.push(Element {
                        height: added_height,
                        index,
                    });
                    compensated_dt -= added_height * 0.5;
                } else {
                    compensated_dt = self.dt;
                    break;
                }
                if let Some(first) = self.visible_window.first() {
                    let height = first.height;
                    if -(self.dt - compensated_dt) > height {
                        let removed = self.visible_window.remove(0);
                        compensated_dt -= removed.height * 0.5;
                    } else {
                        break;
                    }
                }
            }
            self.dt -= compensated_dt;
        } else if self.dt.is_sign_positive() {
            let mut compensated_dt = 0.;
            while compensated_dt < self.dt {
                if let Some(last) = self.visible_window.last() {
                    let height = last.height;
                    if (self.dt + compensated_dt) > height {
                        if let Some(removed) = self.visible_window.pop() {
                            compensated_dt += removed.height * 0.5;
                        }
                    } else {
                        break;
                    }
                }
                let first_index = self.visible_window.first().map(|l| l.index).unwrap_or(0);
                if first_index > 0 {
                    let index = first_index - 1;
                    if let Some(mut element) = cell(&mut state.ui.state, index, id) {
                        let inserted_height =
                            element.min_height(available_area, state).unwrap_or(0.);
                        self.visible_window.insert(
                            0,
                            Element {
                                height: inserted_height,
                                index,
                            },
                        );
                        compensated_dt += inserted_height * 0.5;
                    }
                } else {
                    compensated_dt = self.dt;
                    break;
                }
            }
            self.dt -= compensated_dt;
        }
    }
}

fn scroller<'n, State: 'static, CellFn>(
    id: u64,
    scroller: Binding<State, ScrollerState>,
    cell: CellFn,
) -> Node<'n, RcUi<State>>
where
    CellFn: for<'x> Fn(&'x mut State, usize, u64) -> Option<Node<'n, RcUi<State>>> + 'static + Copy,
{
    stack(vec![
        rect(id!())
            .corner_rounding(30.)
            .fill(AlphaColor::from_rgb8(30, 30, 30))
            .stroke(Color::WHITE, 1.)
            .view()
            .on_scroll(move |s, dt| {
                let mut sc = scroller.get(s);
                sc.dt += dt.y;
                scroller.set(s, sc);
            })
            .finish(),
        clipping(
            |area| {
                ui::RoundedRect::from_origin_size(
                    Point::new(area.x.into(), area.y.into()),
                    Size::new(area.width.into(), area.height.into()),
                    30.,
                )
                .to_path(0.001)
            },
            area_reader::<RcUi<State>>(move |area, state| {
                let mut scroller_state = scroller.get(&state.ui.state);
                scroller_state.update_visible_window(area, state, id, cell);
                let window = &scroller_state.visible_window;
                let mut cells = Vec::new();
                for element in window {
                    if let Some(cell) = cell(&mut state.ui.state, element.index, id) {
                        cells.push(cell);
                    }
                }
                let dt = scroller_state.dt;
                scroller.set(&mut state.ui.state, scroller_state);
                column(cells).offset_y(dt).height(1.)
            })
            .expand(),
        ),
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
