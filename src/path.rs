use std::rc::Rc;

use crate::app::{AppContext, DrawItem};
use crate::shape::{Paint, PathData};
use crate::view::{View, ViewType};
use backer::{Area, Layout};
use vello_svg::vello::kurbo::{BezPath, Stroke};
use vello_svg::vello::peniko::Brush;

pub struct Path {
    id: u64,
    builder: Rc<dyn Fn(Area) -> BezPath>,
    fill: Option<Paint>,
    stroke: Option<(Paint, Stroke)>,
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
    pub fn fill(mut self, brush: impl Into<Brush>) -> Self {
        self.fill = Some(Paint::from_brush(brush));
        self
    }
    pub fn fill_with(mut self, f: impl Fn(Area) -> Brush + 'static) -> Self {
        self.fill = Some(Paint::from_fn(f));
        self
    }
    pub fn stroke(mut self, brush: impl Into<Brush>, style: Stroke) -> Self {
        self.stroke = Some((Paint::from_brush(brush), style));
        self
    }
    pub fn stroke_with(mut self, f: impl Fn(Area) -> Brush + 'static, style: Stroke) -> Self {
        self.stroke = Some((Paint::from_fn(f), style));
        self
    }
    pub fn view<State>(self) -> View<State> {
        View {
            view_type: ViewType::Path(Box::new(PathData {
                id: self.id,
                builder: self.builder,
                fill: self.fill,
                stroke: self.stroke,
            })),
            gesture_handlers: Vec::new(),
        }
    }
    pub fn build<State: 'static>(
        self,
        ctx: &mut AppContext,
    ) -> Layout<DrawItem<State>, AppContext> {
        self.view().finish(ctx)
    }
}
