use std::rc::Rc;

use crate::Color;
use crate::app::{AppContext, DrawItem};
use crate::shape::PathData;
use crate::view::{View, ViewType};
use backer::{Area, Layout};
use vello_svg::vello::kurbo::{BezPath, Stroke};

pub struct Path {
    id: u64,
    builder: Rc<dyn Fn(Area) -> BezPath>,
    fill: Option<Color>,
    stroke: Option<(Color, Stroke)>,
}

pub fn path(id: u64, builder: impl Fn(Area) -> BezPath + 'static) -> Path {
    Path {
        id,
        builder: Rc::new(builder),
        fill: None,
        stroke: None,
    }
}

impl Path {
    pub fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }
    pub fn stroke(mut self, color: Color, style: Stroke) -> Self {
        self.stroke = Some((color, style));
        self
    }
    pub fn view<State>(self) -> View<State> {
        View {
            view_type: ViewType::Path(PathData {
                id: self.id,
                builder: self.builder,
                fill: self.fill,
                stroke: self.stroke,
            }),
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
