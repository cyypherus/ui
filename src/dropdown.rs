use std::iter::repeat;

use crate::Color;
use crate::{
    Binding, ClickState, DEFAULT_CORNER_ROUNDING, DEFAULT_FONT_SIZE, DEFAULT_PURP, app::AppState,
    rect, text,
};
use backer::models::Align;
use backer::{Node, nodes::*};

#[derive(Debug, Clone)]
pub struct DropdownState {
    pub selected: usize,
}

impl DropdownState {
    pub fn new(selected: usize) -> Self {
        Self { selected }
    }
}

pub struct DropDown<State> {
    id: u64,
    corner_rounding: Option<f32>,
    state: Binding<State, DropdownState>,
    fill: Option<Color>,
    text_fill: Option<Color>,
}

pub fn dropdown<'n, State, T, ElementFn>(
    id: u64,
    binding: Binding<State, DropdownState>,
) -> DropDown<State> {
    DropDown {
        id,
        corner_rounding: None,
        state: binding,
        fill: None,
        text_fill: None,
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

    pub fn finish(
        self,
        items: Vec<Node<'n, State, AppState<State>>>,
    ) -> Node<'n, State, AppState<State>>
    where
        State: 'static,
    {
        column(items).align_contents(Align::Top).attach_under(
            rect(crate::id!(self.id))
                .fill(color)
                .corner_rounding(self.corner_rounding.unwrap_or(DEFAULT_CORNER_ROUNDING))
                .finish(),
        )
    }
}
