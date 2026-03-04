use std::rc::Rc;

use crate::app::{AppContext, DrawItem};
use crate::background_style::BrushSource;
use crate::{Binding, ClickState, DEFAULT_CORNER_ROUNDING, DEFAULT_PURP, app::AppState, rect};
use crate::{Color, DEFAULT_DARK_GRAY, DEFAULT_FG_COLOR, DEFAULT_PADDING, TRANSPARENT, Text, svg};
use backer::{Align, Layout, nodes::*};
use vello_svg::vello::kurbo::Stroke;
use vello_svg::vello::peniko::Brush;

#[derive(Debug, Clone, Default)]
pub struct DropdownState {
    pub selected: usize,
    pub hovered: Option<usize>,
    pub expanded: bool,
}

pub struct DropDown<State> {
    id: u64,
    state: DropdownState,
    binding: Binding<State, DropdownState>,
    background_corner_rounding: f32,
    background_fill: BrushSource<DropdownState>,
    background_stroke: (BrushSource<DropdownState>, Stroke),
    text_fill: Brush,
    highlight_fill: Brush,
    background_padding: f32,
    options: Vec<Text>,
    on_select: Option<Rc<dyn Fn(&mut State, &mut AppState<State>, usize)>>,
}

pub fn dropdown<State>(
    id: u64,
    state: (DropdownState, Binding<State, DropdownState>),
    options: Vec<Text>,
) -> DropDown<State> {
    DropDown {
        id,
        state: state.0,
        binding: state.1,
        background_corner_rounding: DEFAULT_CORNER_ROUNDING,
        background_fill: Color::from_rgb8(50, 50, 50).into(),
        background_stroke: (Color::from_rgb8(60, 60, 60).into(), Stroke::new(1.)),
        text_fill: Brush::Solid(DEFAULT_FG_COLOR),
        highlight_fill: Brush::Solid(DEFAULT_PURP),
        background_padding: DEFAULT_PADDING,
        options,
        on_select: None,
    }
}

impl<State> DropDown<State> {
    pub fn background_corner_rounding(mut self, corner_rounding: f32) -> Self {
        self.background_corner_rounding = corner_rounding;
        self
    }
    pub fn background_fill(mut self, fill: impl Into<BrushSource<DropdownState>>) -> Self {
        self.background_fill = fill.into();
        self
    }
    pub fn background_stroke(mut self, brush: impl Into<BrushSource<DropdownState>>, style: Stroke) -> Self {
        self.background_stroke = (brush.into(), style);
        self
    }
    pub fn text_fill(mut self, fill: impl Into<Brush>) -> Self {
        self.text_fill = fill.into();
        self
    }
    pub fn highlight_fill(mut self, fill: impl Into<Brush>) -> Self {
        self.highlight_fill = fill.into();
        self
    }
    pub fn background_padding(mut self, padding: f32) -> Self {
        self.background_padding = padding;
        self
    }
    pub fn on_select(
        mut self,
        on_select: impl Fn(&mut State, &mut AppState<State>, usize) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(on_select));
        self
    }

    pub fn build(self, ctx: &mut AppContext) -> Layout<DrawItem<State>, AppContext>
    where
        State: 'static,
    {
        let expanded = self.state.expanded;
        let selected = self.state.selected;
        let hovered = self.state.hovered;
        let id = self.id;
        let binding = self.binding.clone();
        let on_select = self.on_select.clone();
        let fill = self.background_fill;
        let stroke = self.background_stroke;
        let corner_rounding = self.background_corner_rounding;
        let text_fill = self.text_fill;
        let highlight_fill = self.highlight_fill;

        let arrow_svg = if expanded {
            include_str!("../assets/arrow-down.svg")
        } else {
            include_str!("../assets/arrow-right.svg")
        };

        let row = |index: usize,
                   option: Text,
                   ctx: &mut AppContext|
         -> Layout<DrawItem<State>, AppContext> {
            let row_fill: Brush = if expanded && selected == index {
                highlight_fill.clone()
            } else if let Some(hovered) = hovered
                && hovered == index
            {
                DEFAULT_DARK_GRAY.into()
            } else {
                TRANSPARENT.into()
            };

            row_spaced(
                5.,
                vec![
                    if index == 0 {
                        svg(crate::id!(id), arrow_svg)
                            .fill(text_fill.clone())
                            .finish(ctx)
                            .width(10.)
                            .height(10.)
                    } else {
                        empty()
                    },
                    option.fill(text_fill.clone()).build(ctx),
                ],
            )
            .expand_x()
            .pad(self.background_padding)
            .attach_under(
                rect(crate::id!(index as u64, id))
                    .fill(row_fill)
                    .corner_rounding(corner_rounding)
                    .view()
                    .on_click({
                        let binding = binding.clone();
                        let on_select = on_select.clone();
                        move |state: &mut State, app, click, _pos| {
                            let ClickState::Completed = click else { return };
                            if expanded {
                                if let Some(ref on_select) = on_select {
                                    on_select(state, app, index);
                                }
                                binding.update(state, move |s| {
                                    s.selected = index;
                                    s.expanded = false;
                                });
                            } else {
                                binding.update(state, |s| s.expanded = true);
                            }
                        }
                    })
                    .on_hover({
                        let binding = binding.clone();
                        move |state: &mut State, _app, hovered| {
                            binding.update(state, move |s| {
                                if expanded && hovered {
                                    s.hovered = Some(index)
                                }
                            });
                        }
                    })
                    .finish(ctx)
                    .expand_x(),
            )
        };

        let dd_state = self.state.clone();
        let rows: Vec<_> = if expanded {
            self.options
        } else {
            vec![self.options[selected].clone()]
        }
        .into_iter()
        .enumerate()
        .map(|(index, option)| row(index, option, ctx))
        .collect();

        column(rows).align(Align::Top).attach_under(
            area_reader(move |area, ctx: &mut AppContext| {
                rect(crate::id!(id))
                    .fill(fill.resolve(area, &dd_state))
                    .stroke(stroke.0.resolve(area, &dd_state), stroke.1.clone())
                    .corner_rounding(corner_rounding)
                    .view()
                    .on_click_outside({
                        let binding = binding.clone();
                        move |state: &mut State, _app, click, _pos| {
                            let ClickState::Completed = click else { return };
                            binding.update(state, |s| s.expanded = false);
                        }
                    })
                    .on_hover({
                        let binding = binding.clone();
                        move |state: &mut State, _app, hovered| {
                            binding.update(state, move |s| {
                                if !hovered {
                                    s.hovered = None
                                }
                            });
                        }
                    })
                    .finish(ctx)
            }),
        )
    }
}
