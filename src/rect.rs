use crate::Color;
use crate::app::{AppContext, AppState, DrawItem};
use crate::shape::{Shape, ShapeType};
use crate::view::{View, ViewType};
use backer::{Area, Layout};

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub(crate) id: u64,
    pub(crate) shape: Shape,
    // pub(crate) box_shadow: Option<(Color, f32)>,
}

impl Rect {
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
        self.shape
            .draw(&mut app.scene, area, app.scale_factor, visible_amount);
    }
}

pub fn rect(id: u64) -> Rect {
    Rect {
        id,
        shape: Shape {
            shape: ShapeType::Rect {
                corner_rounding: (0., 0., 0., 0.),
            },
            fill: None,
            stroke: None,
        },
        // box_shadow: None,
    }
}

impl Rect {
    pub fn fill(mut self, color: Color) -> Self {
        self.shape.fill = Some(color);
        self
    }
    pub fn corner_rounding(mut self, radius: f32) -> Self {
        self.shape.shape = ShapeType::Rect {
            corner_rounding: (radius, radius, radius, radius),
        };
        self
    }
    pub fn corner_rounding_individual(
        mut self,
        top_left: f32,
        top_right: f32,
        bottom_right: f32,
        bottom_left: f32,
    ) -> Self {
        self.shape.shape = ShapeType::Rect {
            corner_rounding: (top_left, top_right, bottom_right, bottom_left),
        };
        self
    }
    pub fn stroke(mut self, color: Color, line_width: f32) -> Self {
        self.shape.stroke = Some((color, line_width));
        self
    }
    // pub fn box_shadow(mut self, color: Color, radius: f32) -> Self {
    //     self.box_shadow = Some((color, radius));
    //     self
    // }
    pub fn view<State>(self) -> View<State> {
        View {
            view_type: ViewType::Rect(self),
            z_index: 0,
            gesture_handlers: Vec::new(),
        }
    }
    pub fn finish<State: 'static>(
        self,
        app: &mut AppState<State>,
    ) -> Layout<DrawItem<State>, AppContext> {
        self.view().finish(app)
    }
}
