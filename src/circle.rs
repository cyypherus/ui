use crate::Color;
use crate::app::{AppContext, AppState, DrawItem};
use crate::shape::{Shape, ShapeType};
use crate::view::{View, ViewType};

use backer::{Area, Layout};

#[derive(Debug, Clone)]
pub struct Circle {
    pub(crate) id: u64,
    pub(crate) shape: Shape,
}

impl Circle {
    pub(crate) fn draw<State>(
        &self,
        area: Area,
        _state: &mut State,
        app: &mut AppState<State>,
        visible: bool,
        visible_amount: f32,
    ) {
        if !visible && visible_amount == 0. {
            return;
        }
        self.shape.draw(
            &mut app.scene,
            area,
            app.app_context.scale_factor,
            visible_amount,
        );
    }
}

pub fn circle(id: u64) -> Circle {
    Circle {
        id,
        shape: Shape {
            shape: ShapeType::Circle,
            fill: None,
            stroke: None,
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
    pub fn view<State>(self) -> View<State> {
        View {
            view_type: ViewType::Circle(self),
            z_index: 0,
            gesture_handlers: Vec::new(),
        }
    }
    pub fn finish<State: 'static>(
        self,
        ctx: &mut AppContext,
    ) -> Layout<DrawItem<State>, AppContext> {
        self.view().finish(ctx)
    }
}
