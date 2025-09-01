use crate::{Binding, ClickState, DEFAULT_CORNER_ROUNDING, DEFAULT_PURP, app::AppState, rect};
use crate::{ButtonState, Color, DEFAULT_DARK_GRAY, DEFAULT_FG_COLOR, TRANSPARENT, Text, svg};
use backer::models::Align;
use backer::{Node, nodes::*};

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
    corner_rounding: Option<f32>,
    state: Binding<State, DropdownState>,
    fill: Option<Color>,
    text_fill: Option<Color>,
    highlight_fill: Option<Color>,
    options: Vec<Text>,
}

pub fn dropdown<State>(
    id: u64,
    binding: Binding<State, DropdownState>,
    options: Vec<Text>,
) -> DropDown<State> {
    DropDown {
        id,
        corner_rounding: None,
        state: binding,
        fill: None,
        text_fill: None,
        highlight_fill: None,
        options,
    }
}

impl<'n, State> DropDown<State> {
    pub fn corner_rounding(mut self, corner_rounding: f32) -> Self {
        self.corner_rounding = Some(corner_rounding);
        self
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
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

    pub fn finish(self) -> Node<'n, State, AppState<State>>
    where
        State: 'static,
    {
        dynamic(move |state, _app| {
            let binding = self.state.clone();
            let expanded = binding.get(state).expanded;
            let hovered = binding.get(state).hovered;
            let selected = binding.get(state).selected;
            let option_views = self
                .options
                .clone()
                .into_iter()
                .enumerate()
                .map(move |(index, option)| {
                    let binding = binding.clone();
                    row_spaced(
                        5.,
                        vec![
                            if (index == selected && !expanded) || (index == 0 && expanded) {
                                svg(
                                    crate::id!(self.id),
                                    if expanded {
                                        include_str!("../assets/arrow-down.svg")
                                    } else {
                                        include_str!("../assets/arrow-right.svg")
                                    },
                                )
                                .fill(self.text_fill.unwrap_or(DEFAULT_FG_COLOR))
                                .view()
                                .z_index(1)
                                .finish()
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
                                let option = option
                                    .fill(if index == selected || expanded {
                                        self.text_fill.unwrap_or(DEFAULT_FG_COLOR)
                                    } else {
                                        DEFAULT_PURP
                                    })
                                    .view()
                                    .z_index(1)
                                    .finish();
                                if index == selected || expanded {
                                    option
                                } else {
                                    option.width(0.).height(0.)
                                }
                            },
                        ],
                    )
                    .pad(5.)
                    .attach_over(
                        rect(crate::id!(index as u64, self.id))
                            .fill(TRANSPARENT)
                            .view()
                            .on_click({
                                let binding = binding.clone();
                                move |state, _app, click, _pos| {
                                    if matches!(click, ClickState::Completed) {
                                        if expanded {
                                            binding.update(state, move |state| {
                                                state.selected = index;
                                                state.expanded = false;
                                            });
                                        } else {
                                            binding.update(state, move |state| {
                                                state.expanded = true;
                                            });
                                        }
                                    }
                                }
                            })
                            .finish(),
                    )
                    .attach_under(
                        rect(crate::id!(index as u64, self.id))
                            .fill(
                                if let Some(hovered) = hovered
                                    && hovered == index
                                    && expanded
                                {
                                    self.highlight_fill.unwrap_or(DEFAULT_PURP)
                                } else {
                                    TRANSPARENT
                                },
                            )
                            .corner_rounding(
                                self.corner_rounding.unwrap_or(DEFAULT_CORNER_ROUNDING),
                            )
                            .view()
                            .z_index(1)
                            .transition_duration(0.)
                            .on_hover({
                                let binding = binding.clone();
                                move |state, _app, hovered| {
                                    if hovered && expanded {
                                        binding.update(state, move |state| {
                                            state.hovered = Some(index);
                                        });
                                    }
                                }
                            })
                            .finish(),
                    )
                })
                .collect();

            if expanded {
                column(option_views)
                    .align_contents(Align::TopLeading)
                    .align(Align::Top)
            } else {
                stack(option_views)
                    .align_contents(Align::TopLeading)
                    .align(Align::Top)
            }
            .attach_under(
                rect(crate::id!(self.id))
                    .fill(DEFAULT_DARK_GRAY)
                    .corner_rounding(self.corner_rounding.unwrap_or(DEFAULT_CORNER_ROUNDING))
                    .view()
                    .z_index(1)
                    .finish(),
            )
        })
    }
}
