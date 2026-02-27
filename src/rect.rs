use crate::Color;
use crate::app::{AppContext, DrawItem};
use crate::shape::{PathData, rect_path};
use crate::view::{View, ViewType};
use backer::Layout;
use vello_svg::vello::kurbo::Stroke;

pub struct Rect {
    id: u64,
    fill: Option<Color>,
    stroke: Option<(Color, Stroke)>,
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
    pub fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
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
    pub fn stroke(mut self, color: Color, style: Stroke) -> Self {
        self.stroke = Some((color, style));
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
            view_type: ViewType::Path(self.into_path_data()),
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
