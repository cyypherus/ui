use std::rc::Rc;

use crate::app::{AppContext, DrawItem};
use crate::{Binding, ClickState, DEFAULT_CORNER_ROUNDING, DEFAULT_PURP, app::AppState, rect};
use crate::{Color, DEFAULT_DARK_GRAY, DEFAULT_FG_COLOR, TRANSPARENT, Text, svg};
use backer::{Align, Layout, nodes::*};
use vello_svg::vello::kurbo::Stroke;

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
    corner_rounding: f32,
    fill: Color,
    stroke: (Color, Stroke),
    text_fill: Color,
    highlight_fill: Color,
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
        corner_rounding: DEFAULT_CORNER_ROUNDING,
        fill: Color::from_rgb8(50, 50, 50),
        stroke: (Color::from_rgb8(60, 60, 60), Stroke::new(1.)),
        text_fill: DEFAULT_FG_COLOR,
        highlight_fill: DEFAULT_PURP,
        options,
        on_select: None,
    }
}

impl<State> DropDown<State> {
    pub fn corner_rounding(mut self, corner_rounding: f32) -> Self {
        self.corner_rounding = corner_rounding;
        self
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill = color;
        self
    }

    pub fn stroke(mut self, color: Color, style: Stroke) -> Self {
        self.stroke = (color, style);
        self
    }

    pub fn text_fill(mut self, color: Color) -> Self {
        self.text_fill = color;
        self
    }

    pub fn highlight_fill(mut self, color: Color) -> Self {
        self.highlight_fill = color;
        self
    }

    pub fn on_select(
        mut self,
        on_select: impl Fn(&mut State, &mut AppState<State>, usize) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(on_select));
        self
    }

    pub fn finish(self, ctx: &mut AppContext) -> Layout<DrawItem<State>, AppContext>
    where
        State: 'static,
    {
        let expanded = self.state.expanded;
        let selected = self.state.selected;
        let hovered = self.state.hovered;
        let id = self.id;
        let binding = self.binding.clone();
        let on_select = self.on_select.clone();
        let fill = self.fill;
        let stroke = self.stroke;
        let corner_rounding = self.corner_rounding;
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
            let row_fill = if expanded && selected == index {
                highlight_fill
            } else if let Some(hovered) = hovered
                && hovered == index
            {
                DEFAULT_DARK_GRAY
            } else {
                TRANSPARENT
            };

            row_spaced(
                5.,
                vec![
                    if index == 0 {
                        svg(crate::id!(id), arrow_svg)
                            .fill(text_fill)
                            .finish(ctx)
                            .width(10.)
                            .height(10.)
                    } else {
                        empty()
                    },
                    option.fill(text_fill).finish(ctx),
                ],
            )
            .expand_x()
            .pad(8.)
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
            rect(crate::id!(id))
                .fill(fill)
                .stroke(stroke.0, stroke.1)
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
                .finish(ctx),
        )
    }
}
