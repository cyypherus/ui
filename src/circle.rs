use crate::app::{AppContext, DrawItem};
use crate::shape::{PathData, circle_path};
use crate::view::{View, ViewType};

use backer::Layout;
use vello_svg::vello::kurbo::Stroke;
use vello_svg::vello::peniko::Brush;

pub struct Circle {
    id: u64,
    fill: Option<Brush>,
    stroke: Option<(Brush, Stroke)>,
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
        self.fill = Some(brush.into());
        self
    }
    pub fn stroke(mut self, brush: impl Into<Brush>, style: Stroke) -> Self {
        self.stroke = Some((brush.into(), style));
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
