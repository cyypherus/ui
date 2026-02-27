use std::rc::Rc;

use backer::Area;
use vello_svg::vello::Scene;
use vello_svg::vello::kurbo::{Affine, BezPath, Point, RoundedRect, Shape as _, Stroke};
use vello_svg::vello::peniko::{Brush, Fill};
use vello_svg::vello::peniko::Color;

pub(crate) type PathBuilder = Rc<dyn Fn(Area) -> BezPath>;

#[derive(Clone)]
pub(crate) struct PathData {
    pub(crate) id: u64,
    pub(crate) builder: PathBuilder,
    pub(crate) fill: Option<Color>,
    pub(crate) stroke: Option<(Color, Stroke)>,
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
        let path = Affine::scale(scale_factor) * &user_path;

        if self.fill.is_none() && self.stroke.is_none() {
            scene.fill(
                Fill::EvenOdd,
                Affine::IDENTITY,
                Color::BLACK.multiply_alpha(visible_amount),
                None,
                &path,
            )
        } else {
            if let Some(fill) = self.fill {
                scene.fill(
                    Fill::EvenOdd,
                    Affine::IDENTITY,
                    fill.multiply_alpha(visible_amount),
                    None,
                    &path,
                )
            }
            if let Some((stroke_color, stroke_style)) = &self.stroke {
                let mut scaled = stroke_style.clone();
                scaled.width *= scale_factor;
                scene.stroke(
                    &scaled,
                    Affine::IDENTITY,
                    &Brush::Solid(stroke_color.multiply_alpha(visible_amount)),
                    None,
                    &path,
                );
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
                vello_svg::vello::kurbo::Size::new(
                    area.width as f64,
                    area.height as f64,
                ),
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
