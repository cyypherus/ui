use crate::shape::{AnimatedShape, Shape, ShapeType};
use crate::ui::RcUi;
use crate::view::{AnimatedView, View, ViewType};
use backer::models::Area;
use backer::Node;
use std::time::Instant;
use vello_svg::vello::peniko::Color;

#[derive(Debug, Clone)]
pub struct Circle {
    pub(crate) id: u64,
    pub(crate) shape: Shape,
}

#[derive(Debug, Clone)]
pub(crate) struct AnimatedCircle {
    pub(crate) shape: AnimatedShape,
}

impl AnimatedCircle {
    pub(crate) fn update(now: Instant, from: &Circle, existing: &mut AnimatedCircle) {
        AnimatedShape::update(now, &from.shape, &mut existing.shape);
    }
    pub(crate) fn new_from(from: &Circle) -> Self {
        AnimatedCircle {
            shape: AnimatedShape::new_from(&from.shape),
        }
    }
}

pub fn circle(id: u64) -> Circle {
    Circle {
        id,
        shape: Shape {
            shape: ShapeType::Circle,
            fill: None,
            stroke: None,
            easing: None,
            duration: None,
            delay: 0.,
        },
    }
}

impl Circle {
    pub fn fill(mut self, color: Color) -> Self {
        self.shape.fill = Some(color);
        self
    }
    pub fn stroke(mut self, color: Color, line_width: f32) -> Self {
        self.shape.stroke = Some((color, line_width));
        self
    }
    pub fn view<State>(self) -> View<State, ()> {
        View {
            view_type: ViewType::Circle(self),
            gesture_handlers: Vec::new(),
        }
    }
    pub fn finish<'n, State: 'static>(self) -> Node<'n, RcUi<State>> {
        self.view().finish()
    }
}

impl Circle {
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
        let AnimatedView::Circle(mut animated) = state
            .ui
            .cx()
            .view_state
            .remove(&self.id)
            .unwrap_or(AnimatedView::Circle(Box::new(AnimatedCircle::new_from(
                self,
            ))))
        else {
            return;
        };
        AnimatedCircle::update(state.ui.now, self, &mut animated);
        let now = state.ui.now;
        animated
            .shape
            .draw(&mut state.ui.cx().scene, area, now, visible_amount);
        state
            .ui
            .cx()
            .view_state
            .insert(self.id, AnimatedView::Circle(animated));
    }
}
