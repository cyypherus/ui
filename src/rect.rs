use crate::Color;
use crate::app::AppState;
use crate::shape::{AnimatedShape, Shape, ShapeType};
use crate::view::{AnimatedView, View, ViewType};
use backer::Node;
use backer::models::Area;
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub(crate) id: u64,
    pub(crate) shape: Shape,
    // pub(crate) box_shadow: Option<(Color, f32)>,
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
                corner_rounding: (0., 0., 0., 0.),
            },
            fill: None,
            stroke: None,
            easing: None,
            duration: None,
            delay: 0.,
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
            gesture_handlers: Vec::new(),
        }
    }
    pub fn finish<'n, State: 'static>(self) -> Node<'n, State, AppState<State>> {
        self.view().finish()
    }
}

impl Rect {
    pub(crate) fn draw<State>(
        &mut self,
        area: Area,
        _state: &mut State,
        app: &mut AppState<State>,
        visible: bool,
        visible_amount: f32,
    ) {
        if !visible && visible_amount == 0. {
            return;
        }
        let AnimatedView::Rect(mut animated) = app
            .view_state
            .remove(&self.id)
            .unwrap_or(AnimatedView::Rect(Box::new(AnimatedRect::new_from(self))))
        else {
            return;
        };
        AnimatedRect::update(app.now, self, &mut animated);
        // TODO: Fix box shadow drawing with scale factor
        // if let Some((color, radius)) = self.box_shadow {
        //     app.scene.draw_blurred_rounded_rect(
        //         Affine::IDENTITY,
        //         kurbo::Rect::new(
        //             area.x as f64 * app.scale_factor,
        //             area.y as f64 * app.scale_factor,
        //             area.x as f64 + area.width as f64 * app.scale_factor,
        //             area.y as f64 + area.height as f64 * app.scale_factor,
        //         ),
        //         color,
        //         {
        //             if let ShapeType::Rect { corner_rounding } = self.shape.shape {
        //                 corner_rounding as f64 * app.scale_factor
        //             } else {
        //                 0.0
        //             }
        //         },
        //         radius as f64 * app.scale_factor,
        //     );
        // }
        let now = app.now;
        animated
            .shape
            .draw(&mut app.scene, area, app.scale_factor, now, visible_amount);
        app.view_state.insert(self.id, AnimatedView::Rect(animated));
    }
}
