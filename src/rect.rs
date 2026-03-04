use crate::app::{AppContext, DrawItem};
use crate::shape::{PathData, rect_path};
use crate::view::{View, ViewType};
use backer::Layout;
use vello_svg::vello::kurbo::Stroke;
use vello_svg::vello::peniko::Brush;

pub struct Rect {
    id: u64,
    fill: Option<Brush>,
    stroke: Option<(Brush, Stroke)>,
    corner_rounding: (f32, f32, f32, f32),
}

pub fn rect(id: u64) -> Rect {
    Rect {
        id,
        fill: None,
        stroke: None,
        corner_rounding: (0., 0., 0., 0.),
    }
}

impl Rect {
    pub fn fill(mut self, brush: impl Into<Brush>) -> Self {
        self.fill = Some(brush.into());
        self
    }
    pub fn corner_rounding(mut self, radius: f32) -> Self {
        self.corner_rounding = (radius, radius, radius, radius);
        self
    }
    pub fn corner_rounding_individual(
        mut self,
        top_left: f32,
        top_right: f32,
        bottom_right: f32,
        bottom_left: f32,
    ) -> Self {
        self.corner_rounding = (top_left, top_right, bottom_right, bottom_left);
        self
    }
    pub fn stroke(mut self, brush: impl Into<Brush>, style: impl Into<Stroke>) -> Self {
        self.stroke = Some((brush.into(), style.into()));
        self
    }
    pub(crate) fn into_path_data(self) -> PathData {
        PathData {
            id: self.id,
            builder: rect_path(self.corner_rounding),
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
    pub fn build<State: 'static>(
        self,
        ctx: &mut AppContext,
    ) -> Layout<DrawItem<State>, AppContext> {
        self.view().finish(ctx)
    }
}
