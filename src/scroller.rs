use crate::{
    DEFAULT_CORNER_ROUNDING, TRANSPARENT,
    app::{AppCtx, AppState, View},
    rect,
    view::clipping,
};
use backer::{
    Area, Layout,
    nodes::{area_reader, column, empty, stack},
};
use std::{cell::RefCell, rc::Rc};
use vello_svg::vello::kurbo::{RoundedRect, Shape as _};

#[derive(Debug, Clone, Default)]
pub struct ScrollerState {
    visible_window: Vec<Element>,
    dt: f32,
    compensated: f32,
    offset: f32,
    area: Area,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Element {
    height: f32,
    index: usize,
}

impl ScrollerState {
    fn fill_forwards<State>(
        &mut self,
        ctx: &mut AppCtx,
        available_area: Area,
        id: u64,
        cell: &dyn Fn(usize, u64, &mut AppCtx) -> Option<Layout<View<State>, AppCtx>>,
    ) {
        let mut current_height = self.visible_window.iter().fold(0., |acc, e| acc + e.height);
        let mut index = self.visible_window.last().map(|l| l.index).unwrap_or(0)
            + if self.visible_window.is_empty() { 0 } else { 1 };
        while current_height + self.compensated < available_area.height {
            if let Some(added_height) = cell_height::<State>(ctx, index, id, available_area, cell) {
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

    fn update<State>(
        &mut self,
        available_area: Area,
        ctx: &mut AppCtx,
        id: u64,
        cell: &dyn Fn(usize, u64, &mut AppCtx) -> Option<Layout<View<State>, AppCtx>>,
    ) {
        if self.area != available_area && self.visible_window.len() > 1 {
            self.visible_window.drain(1..);
            let index = self.visible_window[0].index;
            self.visible_window[0].height =
                cell_height::<State>(ctx, index, id, available_area, cell).unwrap_or(0.);
            self.fill_forwards::<State>(ctx, available_area, id, cell);
        }
        if self.visible_window.is_empty() {
            self.fill_forwards::<State>(ctx, available_area, id, cell);
        }
        if self.dt != 0. {
            if self.dt.is_sign_negative() {
                self.compensated += self.dt;
                self.dt = 0.;
                if self
                    .visible_window
                    .last()
                    .and_then(|l| cell_height::<State>(ctx, l.index + 1, id, available_area, cell))
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
                    self.fill_forwards::<State>(ctx, available_area, id, cell);
                }
            } else if self.dt.is_sign_positive() {
                self.compensated += self.dt;
                self.dt = 0.;
                while let Some((ch, true, idx)) = self.visible_window.first().and_then(|f| {
                    if f.index > 0 {
                        cell_height::<State>(ctx, f.index - 1, id, available_area, cell)
                            .map(|ch| (ch, self.compensated >= 0., f.index - 1))
                    } else {
                        self.compensated = self.compensated.min(0.);
                        None
                    }
                }) {
                    self.visible_window.insert(
                        0,
                        Element {
                            height: ch,
                            index: idx,
                        },
                    );
                    self.compensated -= ch;
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
        self.area = available_area;
    }
}

fn cell_height<State>(
    ctx: &mut AppCtx,
    index: usize,
    id: u64,
    available_area: Area,
    cell: &dyn Fn(usize, u64, &mut AppCtx) -> Option<Layout<View<State>, AppCtx>>,
) -> Option<f32> {
    cell(index, id, ctx).and_then(|mut layout| layout.min_height(available_area, ctx))
}

pub fn scroller<State: 'static>(
    id: u64,
    backing: Option<Layout<View<State>, AppCtx>>,
    state: Rc<RefCell<ScrollerState>>,
    cell: impl Fn(usize, u64, &mut AppCtx) -> Option<Layout<View<State>, AppCtx>> + 'static,
    ctx: &mut AppCtx,
) -> Layout<View<State>, AppCtx> {
    let scroll_state = state.clone();
    stack(vec![
        backing.unwrap_or(empty()),
        clipping(
            |area| {
                RoundedRect::from_rect(
                    vello_svg::vello::kurbo::Rect::new(
                        area.x as f64,
                        area.y as f64,
                        (area.x + area.width) as f64,
                        (area.y + area.height) as f64,
                    ),
                    DEFAULT_CORNER_ROUNDING as f64,
                )
                .to_path(0.1)
            },
            area_reader({
                let state = state.clone();
                move |area, ctx: &mut AppCtx| {
                    let mut s = state.borrow_mut();
                    s.update::<State>(area, ctx, id, &cell);
                    let mut cells = Vec::new();
                    for element in &s.visible_window {
                        if let Some(c) = cell(element.index, id, ctx) {
                            cells.push(c.height(element.height));
                        }
                    }
                    let offset = s.offset;
                    let comp = s.compensated;
                    column(cells).offset_y(offset + comp)
                }
            })
            .expand(),
        ),
        rect(crate::id!(id))
            .corner_rounding(DEFAULT_CORNER_ROUNDING)
            .fill(TRANSPARENT)
            .view()
            .on_scroll({
                move |_s: &mut State, _app: &mut AppState, dt| {
                    let mut state = scroll_state.borrow_mut();
                    state.dt += dt.y;
                }
            })
            .finish(ctx),
    ])
}
