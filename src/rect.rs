use crate::shape::{AnimatedShape, Shape, ShapeType};
use crate::ui::RcUi;
use crate::view::{AnimatedView, View, ViewType};
use crate::GestureHandler;
use backer::models::Area;
use backer::Node;
use std::time::Instant;
use vello_svg::vello::peniko::Color;

#[derive(Debug, Clone)]
pub struct Rect {
    pub(crate) id: u64,
    pub(crate) shape: Shape,
}

#[derive(Debug, Clone)]
pub(crate) struct AnimatedRect {
    pub(crate) shape: AnimatedShape,
}

impl AnimatedRect {
    pub(crate) fn update(now: Instant, from: &Rect, existing: &mut AnimatedRect) {
        AnimatedShape::update(now, &from.shape, &mut existing.shape);
    }
    pub(crate) fn new_from(from: &Rect) -> Self {
        AnimatedRect {
            shape: AnimatedShape::new_from(&from.shape),
        }
    }
}

pub fn rect(id: u64) -> Rect {
    Rect {
        id,
        shape: Shape {
            shape: ShapeType::Rect {
                corner_rounding: 0.,
            },
            fill: None,
            stroke: None,
            easing: None,
            duration: None,
            delay: 0.,
        },
    }
}

impl Rect {
    pub fn fill(mut self, color: Color) -> Self {
        self.shape.fill = Some(color);
        self
    }
    pub fn corner_rounding(mut self, radius: f32) -> Self {
        self.shape.shape = ShapeType::Rect {
            corner_rounding: radius,
        };
        self
    }
    pub fn stroke(mut self, color: Color, line_width: f32) -> Self {
        self.shape.stroke = Some((color, line_width));
        self
    }
    pub fn view<State>(self) -> View<State, ()> {
        View {
            view_type: ViewType::Rect(self),
            gesture_handler: GestureHandler {
                on_click: None,
                on_drag: None,
                on_hover: None,
                on_key: None,
            },
        }
    }
    pub fn finish<'n, State: 'n>(self) -> Node<'n, RcUi<State>> {
        self.view().finish()
    }
}

impl Rect {
    pub(crate) fn draw<State>(
        &mut self,
        area: Area,
        state: &mut RcUi<State>,
        visible: bool,
        visible_amount: f32,
    ) {
        if !visible && visible_amount == 0. {
            return;
        }
        let AnimatedView::Rect(mut animated) = state
            .ui
            .cx()
            .view_state
            .remove(&self.id)
            .unwrap_or(AnimatedView::Rect(Box::new(AnimatedRect::new_from(self))))
        else {
            return;
        };
        AnimatedRect::update(state.ui.now, self, &mut animated);
        let now = state.ui.now;
        animated
            .shape
            .draw(&mut state.ui.cx().scene, area, now, visible_amount);
        state
            .ui
            .cx()
            .view_state
            .insert(self.id, AnimatedView::Rect(animated));
    }
}
