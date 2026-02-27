use crate::app::{AppContext, DrawItem};
use crate::shape::{Paint, PathData, circle_path};
use crate::view::{View, ViewType};

use backer::{Area, Layout};
use vello_svg::vello::kurbo::Stroke;
use vello_svg::vello::peniko::Brush;

pub struct Circle {
    id: u64,
    fill: Option<Paint>,
    stroke: Option<(Paint, Stroke)>,
}

pub fn circle(id: u64) -> Circle {
    Circle {
        id,
        fill: None,
        stroke: None,
    }
}

impl Circle {
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
    pub(crate) fn into_path_data(self) -> PathData {
        PathData {
            id: self.id,
            builder: circle_path(),
            fill: self.fill,
            stroke: self.stroke,
        }
    }
    pub fn view<State>(self) -> View<State> {
        View {
            view_type: ViewType::Path(Box::new(self.into_path_data())),
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
