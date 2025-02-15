use crate::{clipping, rect, Binding, RcUi, DEFAULT_CORNER_ROUNDING};
use backer::{
    models::Area,
    nodes::{area_reader, column, stack},
    Node,
};
use vello_svg::vello::{
    kurbo::{Point, RoundedRect, Shape, Size},
    peniko::Color,
};

#[derive(Debug, Clone, Default)]
pub struct ScrollerState {
    visible_window: Vec<Element>,
    dt: f32,
    compensated: f32,
    offset: f32,
    _limit_offset: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Element {
    height: f32,
    index: usize,
}

impl ScrollerState {
    fn fill_forwards<State, CellFn>(
        &mut self,
        state: &mut State,
        available_area: Area,
        id: u64,
        cell: CellFn,
    ) where
        CellFn: Fn(&mut State, usize, u64, Area) -> Option<f32> + Copy,
    {
        let mut current_height = self.visible_window.iter().fold(0., |acc, e| acc + e.height);
        let mut index = self.visible_window.last().map(|l| l.index).unwrap_or(0) + {
            if self.visible_window.is_empty() {
                0
            } else {
                1
            }
        };
        while current_height + self.compensated < available_area.height {
            if let Some(added_height) = cell(state, index, id, available_area) {
                current_height += added_height;
                self.visible_window.push(Element {
                    height: added_height,
                    index,
                });
                index += 1;
            } else {
                break;
            }
        }
    }
    pub fn update<State, CellFn>(
        &mut self,
        available_area: Area,
        state: &mut State,
        id: u64,
        cell: CellFn,
    ) where
        CellFn: Fn(&mut State, usize, u64, Area) -> Option<f32> + Copy,
    {
        if self.visible_window.is_empty() {
            self.fill_forwards(state, available_area, id, cell);
        }
        if self.dt != 0. {
            if self.dt.is_sign_negative() {
                self.compensated += self.dt;
                self.dt = 0.;
                if self
                    .visible_window
                    .last()
                    .and_then(|l| cell(state, l.index + 1, id, available_area))
                    .is_none()
                {
                    self.compensated = self.compensated.max(
                        available_area.height
                            - self.visible_window.iter().fold(0., |acc, e| acc + e.height),
                    );
                } else {
                    while let Some(true) = self
                        .visible_window
                        .first()
                        .map(|first| first.height < -self.compensated && -self.compensated > 0.)
                    {
                        let removed = self.visible_window.remove(0);
                        self.compensated += removed.height;
                    }
                    self.fill_forwards(state, available_area, id, cell)
                }
            } else if self.dt.is_sign_positive() {
                self.compensated += self.dt;
                self.dt = 0.;
                while let Some((cell_height, true, index)) =
                    self.visible_window.first().and_then(|f| {
                        if f.index > 0 {
                            cell(state, f.index - 1, id, available_area).map(|cell_height| {
                                (cell_height, self.compensated >= 0., f.index - 1)
                            })
                        } else {
                            // if self.compensated > 0. {
                            //     self.limit_offset = self.compensated.min(self.compensated * 0.2);
                            // }
                            self.compensated = self.compensated.min(0.);
                            None
                        }
                    })
                {
                    self.visible_window.insert(
                        0,
                        Element {
                            height: cell_height,
                            index,
                        },
                    );
                    self.compensated -= cell_height;
                }
                while self.visible_window.len() > 1
                    && self.visible_window.iter().fold(0., |acc, e| acc + e.height)
                        - self.visible_window.last().map(|l| l.height).unwrap_or(0.)
                        + self.compensated
                        > available_area.height
                {
                    self.visible_window.pop();
                }
            }
        }
        self.offset = -(available_area.height
            - self.visible_window.iter().fold(0., |acc, e| acc + e.height))
            * 0.5;
    }
}

pub fn scroller<'n, State: 'static, CellFn>(
    id: u64,
    backing: Node<'n, RcUi<State>>,
    scroller: Binding<State, ScrollerState>,
    cell: CellFn,
) -> Node<'n, RcUi<State>>
where
    CellFn: for<'x> Fn(&'x mut State, usize, u64) -> Option<Node<'n, RcUi<State>>> + Copy + 'static,
{
    stack(vec![
        backing,
        clipping(
            |area| {
                RoundedRect::from_origin_size(
                    Point::new(area.x.into(), area.y.into()),
                    Size::new(area.width.into(), area.height.into()),
                    DEFAULT_CORNER_ROUNDING,
                )
                .to_path(0.001)
            },
            area_reader::<RcUi<State>>({
                let scroller = scroller.clone();
                move |area, state| {
                    let mut scroller_state = scroller.get(&state.ui.state);
                    scroller_state.update(area, state, id, |state, index, id, area| {
                        cell(&mut state.ui.state, index, id)?.min_height(area, state)
                    });
                    let window = &scroller_state.visible_window;
                    let mut cells = Vec::new();
                    for element in window {
                        if let Some(cell) = cell(&mut state.ui.state, element.index, id) {
                            cells.push(cell);
                        }
                    }
                    let offset = scroller_state.offset;
                    let comp = scroller_state.compensated;
                    // let limit_offset = scroller_state.limit_offset;

                    scroller.set(&mut state.ui.state, scroller_state);
                    column(cells)
                        //
                        .offset_y(offset + comp)
                        .height(1.)
                }
            })
            .expand(),
        ),
        rect(crate::id!())
            .corner_rounding(DEFAULT_CORNER_ROUNDING as f32)
            .fill(Color::TRANSPARENT)
            .view()
            .on_scroll({
                let scroller = scroller.clone();
                move |s, dt| {
                    let mut sc = scroller.get(s);
                    sc.dt += dt.y;
                    scroller.set(s, sc);
                }
            })
            .finish(),
    ])
}
