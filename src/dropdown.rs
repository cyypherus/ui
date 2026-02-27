use std::rc::Rc;

use crate::app::{AppContext, DrawItem};
use crate::{Binding, ClickState, DEFAULT_CORNER_ROUNDING, DEFAULT_PURP, app::AppState, rect};
use crate::{ButtonState, Color, DEFAULT_DARK_GRAY, DEFAULT_FG_COLOR, TRANSPARENT, Text, svg};
use backer::{Align, Layout, nodes::*};
use vello_svg::vello::kurbo::Stroke;

#[derive(Debug, Clone, Default)]
pub struct DropdownState {
    pub selected: usize,
    pub hovered: Option<usize>,
    pub expanded: bool,
    pub button: ButtonState,
}

impl DropdownState {
    pub fn new(
        selected: usize,
        hovered: Option<usize>,
        expanded: bool,
        button: ButtonState,
    ) -> Self {
        Self {
            selected,
            hovered,
            expanded,
            button,
        }
    }
}

pub struct DropDown<State> {
    id: u64,
    state: DropdownState,
    binding: Binding<State, DropdownState>,
    corner_rounding: Option<f32>,
    fill: Option<Color>,
    stroke: Option<(Color, Stroke)>,
    text_fill: Option<Color>,
    highlight_fill: Option<Color>,
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
        corner_rounding: None,
        fill: None,
        stroke: None,
        text_fill: None,
        highlight_fill: None,
        options,
        on_select: None,
    }
}

impl<State> DropDown<State> {
    pub fn corner_rounding(mut self, corner_rounding: f32) -> Self {
        self.corner_rounding = Some(corner_rounding);
        self
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    pub fn stroke(mut self, color: Color, style: Stroke) -> Self {
        self.stroke = Some((color, style));
        self
    }

    pub fn text_fill(mut self, color: Color) -> Self {
        self.text_fill = Some(color);
        self
    }

    pub fn highlight_fill(mut self, color: Color) -> Self {
        self.highlight_fill = Some(color);
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
        let state = self.state;
        let binding = self.binding.clone();
        let expanded = state.expanded;
        let hovered = state.hovered;
        let selected = state.selected;
        let on_select = self.on_select.clone();
        let id = self.id;
        let highlight_fill = self.highlight_fill;
        let corner_rounding = self.corner_rounding;
        let text_fill = self.text_fill;
        let fill = self.fill;
        let stroke = self.stroke;

        let mut option_views = Vec::new();
        for (index, option) in self.options.clone().into_iter().enumerate() {
            let binding_clone = binding.clone();
            let binding_clone2 = binding.clone();
            let on_select_clone = on_select.clone();

            let view = stack_aligned(
                Align::Leading,
                vec![
                    rect(crate::id!(index as u64, id))
                        .fill(
                            if let Some(h) = hovered
                                && h == index
                                && expanded
                            {
                                highlight_fill.unwrap_or(DEFAULT_PURP)
                            } else {
                                TRANSPARENT
                            },
                        )
                        .corner_rounding(corner_rounding.unwrap_or(DEFAULT_CORNER_ROUNDING))
                        .view()
                        .on_hover({
                            let binding = binding_clone.clone();
                            move |state: &mut State, _app, h| {
                                if h && expanded {
                                    binding.update(state, move |state| {
                                        state.hovered = Some(index);
                                    });
                                }
                            }
                        })
                        .finish(ctx)
                        .layer(1),
                    row_spaced(
                        5.,
                        vec![
                            if (index == selected && !expanded) || (index == 0 && expanded) {
                                svg(
                                    crate::id!(id),
                                    if expanded {
                                        include_str!("../assets/arrow-down.svg")
                                    } else {
                                        include_str!("../assets/arrow-right.svg")
                                    },
                                )
                                .fill(text_fill.unwrap_or(DEFAULT_FG_COLOR))
                                .view()
                                .z_index(1)
                                .finish(ctx)
                                .width(12.)
                                .height(if expanded {
                                    12.
                                } else {
                                    10.
                                })
                            } else {
                                empty()
                            },
                            {
                                let opt = option
                                    .fill(if index == selected || expanded {
                                        text_fill.unwrap_or(DEFAULT_FG_COLOR)
                                    } else {
                                        TRANSPARENT
                                    })
                                    .view()
                                    .z_index(1)
                                    .finish(ctx);
                                if index == selected || expanded {
                                    opt
                                } else {
                                    opt.width(0.).height(0.)
                                }
                            },
                        ],
                    )
                    .pad(5.)
                    .layer(1),
                ],
            )
            .attach_over(
                rect(crate::id!(index as u64, id))
                    .fill(TRANSPARENT)
                    .view()
                    .on_click({
                        let binding = binding_clone2.clone();
                        let on_select = on_select_clone.clone();
                        move |state: &mut State, app, click, _pos| {
                            if matches!(click, ClickState::Completed) {
                                if expanded {
                                    if let Some(ref on_select) = on_select {
                                        on_select(state, app, index);
                                    }
                                    binding.update(state, move |state| {
                                        state.selected = index;
                                        state.expanded = false;
                                    });
                                } else {
                                    binding.update(state, move |state| {
                                        state.hovered = Some(0);
                                        state.expanded = true;
                                    });
                                }
                            }
                        }
                    })
                    .finish(ctx)
                    .layer(1),
            );

            option_views.push(view);
        }

        if expanded {
            column(option_views).align(Align::Top)
        } else {
            stack(option_views).align(Align::Top)
        }
        .attach_under(
            rect(crate::id!(id))
                .fill(fill.unwrap_or(DEFAULT_DARK_GRAY))
                .stroke(
                    stroke.as_ref().map(|s| s.0).unwrap_or(TRANSPARENT),
                    stroke.unwrap_or((TRANSPARENT, Stroke::new(0.))).1,
                )
                .corner_rounding(corner_rounding.unwrap_or(DEFAULT_CORNER_ROUNDING))
                .view()
                .on_click_outside({
                    let binding = binding.clone();
                    move |state: &mut State, _app, _click, _pos| {
                        if expanded {
                            binding.update(state, move |state| {
                                state.expanded = false;
                            });
                        }
                    }
                })
                .finish(ctx)
                .layer(1),
        )
    }
}
