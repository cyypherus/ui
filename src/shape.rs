use std::rc::Rc;

use backer::Area;
use vello_svg::vello::Scene;
use vello_svg::vello::kurbo::{Affine, BezPath, Point, RoundedRect, Shape as _, Stroke};
use vello_svg::vello::peniko::{Brush, Color, Fill};

pub(crate) type PathBuilder = Rc<dyn Fn(Area) -> BezPath>;

#[derive(Clone)]
pub(crate) struct PathData {
    pub(crate) id: u64,
    pub(crate) builder: PathBuilder,
    pub(crate) fill: Option<Brush>,
    pub(crate) stroke: Option<(Brush, Stroke)>,
}

impl PathData {
    pub(crate) fn draw(
        &self,
        scene: &mut Scene,
        area: Area,
        scale_factor: f64,
        visible_amount: f32,
    ) {
        let user_path = (self.builder)(area);
        let scale = Affine::scale(scale_factor);
        let path = scale * &user_path;

        if self.fill.is_none() && self.stroke.is_none() {
            scene.fill(
                Fill::EvenOdd,
                Affine::IDENTITY,
                Color::BLACK.multiply_alpha(visible_amount),
                None,
                &path,
            )
        } else {
            if let Some(brush) = &self.fill {
                let brush = brush.clone().multiply_alpha(visible_amount);
                scene.fill(Fill::EvenOdd, scale, &brush, None, &user_path)
            }
            if let Some((brush, stroke_style)) = &self.stroke {
                let brush = brush.clone().multiply_alpha(visible_amount);
                let mut scaled = stroke_style.clone();
                scaled.width *= scale_factor;
                scene.stroke(&scaled, scale, &brush, None, &user_path);
            }
        }
    }
}

pub(crate) fn rect_path(corner_rounding: (f32, f32, f32, f32)) -> PathBuilder {
    Rc::new(move |area| {
        let (top_left, top_right, bottom_left, bottom_right) = corner_rounding;
        RoundedRect::from_rect(
            vello_svg::vello::kurbo::Rect::from_origin_size(
                Point::new(area.x as f64, area.y as f64),
                vello_svg::vello::kurbo::Size::new(area.width as f64, area.height as f64),
            ),
            (
                top_left as f64,
                top_right as f64,
                bottom_left as f64,
                bottom_right as f64,
            ),
        )
        .to_path(0.01)
    })
}

pub(crate) fn circle_path() -> PathBuilder {
    Rc::new(|area| {
        let radius = f32::min(area.width, area.height) * 0.5;
        vello_svg::vello::kurbo::Circle::new(
            Point::new(
                (area.x + (area.width * 0.5)) as f64,
                (area.y + (area.height * 0.5)) as f64,
            ),
            radius as f64,
        )
        .to_path(0.01)
    })
}
