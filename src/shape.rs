use std::rc::Rc;

use backer::Area;
use vello_svg::vello::Scene;
use vello_svg::vello::kurbo::{Affine, BezPath, Point, RoundedRect, Shape as _, Stroke};
use vello_svg::vello::peniko::{Brush, Fill};
use vello_svg::vello::peniko::Color;

pub(crate) type PathBuilder = Rc<dyn Fn(Area) -> BezPath>;

pub(crate) enum Paint {
    Static(Brush),
    Dynamic(Rc<dyn Fn(Area) -> Brush>),
}

impl Clone for Paint {
    fn clone(&self) -> Self {
        match self {
            Paint::Static(b) => Paint::Static(b.clone()),
            Paint::Dynamic(f) => Paint::Dynamic(f.clone()),
        }
    }
}

impl Paint {
    pub(crate) fn resolve(&self, area: Area) -> Brush {
        match self {
            Paint::Static(b) => b.clone(),
            Paint::Dynamic(f) => f(area),
        }
    }

    pub(crate) fn from_brush(brush: impl Into<Brush>) -> Self {
        Paint::Static(brush.into())
    }

    pub(crate) fn from_fn(f: impl Fn(Area) -> Brush + 'static) -> Self {
        Paint::Dynamic(Rc::new(f))
    }
}

#[derive(Clone)]
pub(crate) struct PathData {
    pub(crate) id: u64,
    pub(crate) builder: PathBuilder,
    pub(crate) fill: Option<Paint>,
    pub(crate) stroke: Option<(Paint, Stroke)>,
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
            if let Some(paint) = &self.fill {
                let brush = paint.resolve(area).multiply_alpha(visible_amount);
                scene.fill(
                    Fill::EvenOdd,
                    scale,
                    &brush,
                    None,
                    &user_path,
                )
            }
            if let Some((paint, stroke_style)) = &self.stroke {
                let brush = paint.resolve(area).multiply_alpha(visible_amount);
                let mut scaled = stroke_style.clone();
                scaled.width *= scale_factor;
                scene.stroke(
                    &scaled,
                    scale,
                    &brush,
                    None,
                    &user_path,
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
