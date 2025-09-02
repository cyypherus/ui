use crate::animated_color::{AnimatedColor, AnimatedU8};
use crate::{DEFAULT_DURATION, DEFAULT_EASING};
use backer::models::Area;
use lilt::{Animated, Easing};
use std::time::Instant;
use vello_svg::vello::Scene;
use vello_svg::vello::kurbo::{Point, RoundedRect, Shape as KurboShape, Stroke};
use vello_svg::vello::peniko::{Brush, Fill};
use vello_svg::vello::{kurbo::Affine, peniko::Color};

#[derive(Debug, Clone, Copy)]
pub struct Shape {
    pub(crate) shape: ShapeType,
    pub(crate) fill: Option<Color>,
    pub(crate) stroke: Option<(Color, f32)>,
    pub(crate) easing: Option<Easing>,
    pub(crate) duration: Option<f32>,
    pub(crate) delay: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ShapeType {
    Circle,
    Rect {
        corner_rounding: (f32, f32, f32, f32),
    },
}

#[derive(Debug, Clone)]
pub(crate) struct AnimatedShape {
    pub(crate) shape: AnimatedShapeType,
    pub(crate) fill: Option<AnimatedColor>,
    pub(crate) stroke: Option<(AnimatedColor, Animated<f32, Instant>)>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub(crate) enum AnimatedShapeType {
    Circle,
    Rect {
        corner_rounding: (
            Animated<f32, Instant>,
            Animated<f32, Instant>,
            Animated<f32, Instant>,
            Animated<f32, Instant>,
        ),
    },
}

impl AnimatedShape {
    pub(crate) fn in_progress(&self, now: Instant) -> bool {
        self.fill
            .as_ref()
            .map(|f| f.in_progress(now))
            .unwrap_or(false)
            || self
                .stroke
                .as_ref()
                .map(|f| f.0.in_progress(now) || f.1.in_progress(now))
                .unwrap_or(false)
    }
    pub(crate) fn update(now: Instant, from: &Shape, existing: &mut AnimatedShape) {
        if let (Some(existing_fill), Some(new_fill)) = (&mut existing.fill, from.fill) {
            existing_fill.transition(new_fill, now);
        }
        if let (Some((existing_stroke, existing_width)), Some((new_stroke, new_width))) =
            (&mut existing.stroke, from.stroke)
        {
            existing_stroke.transition(new_stroke, now);
            existing_width.transition(new_width, now);
        }
        match (&mut existing.shape, from.shape) {
            (AnimatedShapeType::Circle, ShapeType::Circle) => (),
            (
                AnimatedShapeType::Rect {
                    corner_rounding:
                        (
                            existing_top_left,
                            existing_top_right,
                            existing_bottom_left,
                            existing_bottom_right,
                        ),
                },
                ShapeType::Rect {
                    corner_rounding: (top_left, top_right, bottom_left, bottom_right),
                },
            ) => {
                existing_top_left.transition(top_left, now);
                existing_top_right.transition(top_right, now);
                existing_bottom_left.transition(bottom_left, now);
                existing_bottom_right.transition(bottom_right, now);
            }
            _ => debug_assert!(false, "Mismatched shape types"),
        }
    }
    pub(crate) fn new_from(from: &Shape) -> Self {
        AnimatedShape {
            fill: from.fill.map(|fill| AnimatedColor {
                r: Animated::new(AnimatedU8(fill.to_rgba8().r))
                    .easing(from.easing.unwrap_or(DEFAULT_EASING))
                    .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                    .delay(from.delay),
                g: Animated::new(AnimatedU8(fill.to_rgba8().g))
                    .easing(from.easing.unwrap_or(DEFAULT_EASING))
                    .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                    .delay(from.delay),
                b: Animated::new(AnimatedU8(fill.to_rgba8().b))
                    .easing(from.easing.unwrap_or(DEFAULT_EASING))
                    .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                    .delay(from.delay),
                a: Animated::new(AnimatedU8(fill.to_rgba8().a))
                    .easing(from.easing.unwrap_or(DEFAULT_EASING))
                    .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                    .delay(from.delay),
            }),
            stroke: from.stroke.map(|(color, width)| {
                (
                    AnimatedColor {
                        r: Animated::new(AnimatedU8(color.to_rgba8().r))
                            .easing(from.easing.unwrap_or(DEFAULT_EASING))
                            .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                            .delay(from.delay),
                        g: Animated::new(AnimatedU8(color.to_rgba8().g))
                            .easing(from.easing.unwrap_or(DEFAULT_EASING))
                            .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                            .delay(from.delay),
                        b: Animated::new(AnimatedU8(color.to_rgba8().b))
                            .easing(from.easing.unwrap_or(DEFAULT_EASING))
                            .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                            .delay(from.delay),
                        a: Animated::new(AnimatedU8(color.to_rgba8().a))
                            .easing(from.easing.unwrap_or(DEFAULT_EASING))
                            .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                            .delay(from.delay),
                    },
                    Animated::new(width)
                        .easing(from.easing.unwrap_or(DEFAULT_EASING))
                        .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                        .delay(from.delay),
                )
            }),
            shape: match from.shape {
                ShapeType::Circle => AnimatedShapeType::Circle,
                ShapeType::Rect {
                    corner_rounding: (top_left, top_right, bottom_left, bottom_right),
                } => AnimatedShapeType::Rect {
                    corner_rounding: (
                        Animated::new(top_left)
                            .easing(from.easing.unwrap_or(DEFAULT_EASING))
                            .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                            .delay(from.delay),
                        Animated::new(top_right)
                            .easing(from.easing.unwrap_or(DEFAULT_EASING))
                            .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                            .delay(from.delay),
                        Animated::new(bottom_left)
                            .easing(from.easing.unwrap_or(DEFAULT_EASING))
                            .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                            .delay(from.delay),
                        Animated::new(bottom_right)
                            .easing(from.easing.unwrap_or(DEFAULT_EASING))
                            .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                            .delay(from.delay),
                    ),
                },
            },
        }
    }
}

impl AnimatedShape {
    pub(crate) fn draw(
        &mut self,
        scene: &mut Scene,
        area: Area,
        scale_factor: f64,
        now: Instant,
        visible_amount: f32,
    ) {
        let path = match &self.shape {
            AnimatedShapeType::Circle => {
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
            AnimatedShapeType::Rect {
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
                    top_left.animate_wrapped(now) as f64 * scale_factor,
                    top_right.animate_wrapped(now) as f64 * scale_factor,
                    bottom_left.animate_wrapped(now) as f64 * scale_factor,
                    bottom_right.animate_wrapped(now) as f64 * scale_factor,
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
            if let Some(fill) = &self.fill {
                scene.fill(
                    Fill::EvenOdd,
                    Affine::IDENTITY,
                    Color::from_rgba8(
                        fill.r.animate_wrapped(now).0,
                        fill.g.animate_wrapped(now).0,
                        fill.b.animate_wrapped(now).0,
                        fill.a.animate_wrapped(now).0,
                    )
                    .multiply_alpha(visible_amount),
                    None,
                    &path,
                )
            }
            if let Some((stroke, width)) = &self.stroke {
                scene.stroke(
                    &Stroke::new(width.animate_wrapped(now) as f64 * scale_factor),
                    Affine::IDENTITY,
                    &Brush::Solid(
                        Color::from_rgba8(
                            stroke.r.animate_wrapped(now).0,
                            stroke.g.animate_wrapped(now).0,
                            stroke.b.animate_wrapped(now).0,
                            stroke.a.animate_wrapped(now).0,
                        )
                        .multiply_alpha(visible_amount),
                    ),
                    None,
                    &path,
                );
            }
        }
    }
}
