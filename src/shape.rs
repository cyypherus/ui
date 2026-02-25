use backer::Area;
use vello_svg::vello::Scene;
use vello_svg::vello::kurbo::{Point, RoundedRect, Shape as KurboShape, Stroke};
use vello_svg::vello::peniko::{Brush, Fill};
use vello_svg::vello::{kurbo::Affine, peniko::Color};

#[derive(Debug, Clone, Copy)]
pub struct Shape {
    pub(crate) shape: ShapeType,
    pub(crate) fill: Option<Color>,
    pub(crate) stroke: Option<(Color, f32)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ShapeType {
    Circle,
    Rect {
        corner_rounding: (f32, f32, f32, f32),
    },
}

impl Shape {
    pub(crate) fn draw(
        &self,
        scene: &mut Scene,
        area: Area,
        scale_factor: f64,
        visible_amount: f32,
    ) {
        let path = match self.shape {
            ShapeType::Circle => {
                let radius = f32::min(area.width, area.height) * 0.5;
                vello_svg::vello::kurbo::Circle::new(
                    Point::new(
                        (area.x + (area.width * 0.5)) as f64 * scale_factor,
                        (area.y + (area.height * 0.5)) as f64 * scale_factor,
                    ),
                    radius as f64 * scale_factor,
                )
                .to_path(0.01)
            }
            ShapeType::Rect {
                corner_rounding: (top_left, top_right, bottom_left, bottom_right),
            } => RoundedRect::from_rect(
                vello_svg::vello::kurbo::Rect::from_origin_size(
                    vello_svg::vello::kurbo::Point::new(
                        area.x as f64 * scale_factor,
                        area.y as f64 * scale_factor,
                    ),
                    vello_svg::vello::kurbo::Size::new(
                        area.width as f64 * scale_factor,
                        area.height as f64 * scale_factor,
                    ),
                ),
                (
                    top_left as f64 * scale_factor,
                    top_right as f64 * scale_factor,
                    bottom_left as f64 * scale_factor,
                    bottom_right as f64 * scale_factor,
                ),
            )
            .to_path(0.01),
        };

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
            if let Some((stroke, width)) = self.stroke {
                scene.stroke(
                    &Stroke::new(width as f64 * scale_factor),
                    Affine::IDENTITY,
                    &Brush::Solid(stroke.multiply_alpha(visible_amount)),
                    None,
                    &path,
                );
            }
        }
    }
}
