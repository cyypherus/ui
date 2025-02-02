use crate::ui::RcUi;
use crate::view::{AnimatedView, View, ViewTrait, ViewType};
use crate::{GestureHandler, DEFAULT_DURATION, DEFAULT_EASING};
use backer::{models::Area, Node};
use lilt::{Animated, Easing, FloatRepresentable, Interpolable};
use std::time::Instant;
use vello::kurbo::{RoundedRect, Shape, Stroke};
use vello::peniko::{Brush, Fill};
use vello::{kurbo::Affine, peniko::Color};

#[derive(Debug, Clone)]
pub struct Rect {
    pub(crate) id: u64,
    pub(crate) fill: Option<Color>,
    pub(crate) radius: f32,
    pub(crate) stroke: Option<(Color, f32)>,
    pub(crate) easing: Option<Easing>,
    pub(crate) duration: Option<f32>,
    pub(crate) delay: f32,
}

#[derive(Debug, Clone)]
pub(crate) struct AnimatedRect {
    pub(crate) fill: Option<AnimatedColor>,
    pub(crate) radius: Animated<f32, Instant>,
    pub(crate) stroke: Option<(AnimatedColor, Animated<f32, Instant>)>,
}

impl AnimatedRect {
    pub(crate) fn update(from: &Rect, existing: &mut AnimatedRect) {
        let now = Instant::now();
        if let (Some(existing_fill), Some(new_fill)) = (&mut existing.fill, from.fill) {
            existing_fill.r.transition(AnimatedU8(new_fill.r), now);
            existing_fill.g.transition(AnimatedU8(new_fill.g), now);
            existing_fill.b.transition(AnimatedU8(new_fill.b), now);
        }
        existing.radius.transition(from.radius, now);
        if let (Some((existing_stroke, existing_width)), Some((new_stroke, new_width))) =
            (&mut existing.stroke, from.stroke)
        {
            existing_stroke.r.transition(AnimatedU8(new_stroke.r), now);
            existing_stroke.g.transition(AnimatedU8(new_stroke.g), now);
            existing_stroke.b.transition(AnimatedU8(new_stroke.b), now);
            existing_width.transition(new_width, now);
        }
    }
    pub(crate) fn new_from(from: &Rect) -> Self {
        AnimatedRect {
            fill: from.fill.map(|fill| AnimatedColor {
                r: Animated::new(AnimatedU8(fill.r))
                    .easing(from.easing.unwrap_or(DEFAULT_EASING))
                    .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                    .delay(from.delay),
                g: Animated::new(AnimatedU8(fill.g))
                    .easing(from.easing.unwrap_or(DEFAULT_EASING))
                    .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                    .delay(from.delay),
                b: Animated::new(AnimatedU8(fill.b))
                    .easing(from.easing.unwrap_or(DEFAULT_EASING))
                    .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                    .delay(from.delay),
            }),
            radius: Animated::new(from.radius)
                .easing(from.easing.unwrap_or(DEFAULT_EASING))
                .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                .delay(from.delay),
            stroke: from.stroke.map(|(color, width)| {
                (
                    AnimatedColor {
                        r: Animated::new(AnimatedU8(color.r))
                            .easing(from.easing.unwrap_or(DEFAULT_EASING))
                            .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                            .delay(from.delay),
                        g: Animated::new(AnimatedU8(color.g))
                            .easing(from.easing.unwrap_or(DEFAULT_EASING))
                            .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                            .delay(from.delay),
                        b: Animated::new(AnimatedU8(color.b))
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
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AnimatedColor {
    pub(crate) r: Animated<AnimatedU8, Instant>,
    pub(crate) g: Animated<AnimatedU8, Instant>,
    pub(crate) b: Animated<AnimatedU8, Instant>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct AnimatedU8(pub(crate) u8);

impl FloatRepresentable for AnimatedU8 {
    fn float_value(&self) -> f32 {
        self.0 as f32
    }
}

impl Interpolable for AnimatedU8 {
    fn interpolated(&self, other: Self, ratio: f32) -> Self {
        let start = self.0 as f32;
        let end = other.0 as f32;
        let result = start + (end - start) * ratio;
        AnimatedU8(result.round().clamp(0.0, 255.0) as u8)
    }
}

pub fn rect(id: u64) -> Rect {
    Rect {
        id,
        fill: None,
        radius: 0.,
        stroke: None,
        easing: None,
        duration: None,
        delay: 0.,
    }
}

impl Rect {
    pub fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }
    pub fn corner_rounding(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }
    pub fn stroke(mut self, color: Color, line_width: f32) -> Self {
        self.stroke = Some((color, line_width));
        self
    }
    pub fn finish<State>(self) -> View<State> {
        View {
            view_type: ViewType::Rect(self),
            gesture_handler: GestureHandler {
                on_click: None,
                on_drag: None,
                on_hover: None,
            },
        }
    }
}

impl Rect {
    pub(crate) fn draw<State>(
        &mut self,
        area: Area,
        state: &mut RcUi<State>,
        visible: bool,
        visible_amount: f32,
    ) {
        if !visible && visible_amount == 0. {
            return;
        }
        let AnimatedView::Rect(mut animated) = state
            .ui
            .cx()
            .view_state
            .remove(&self.id)
            .unwrap_or(AnimatedView::Rect(Box::new(AnimatedRect::new_from(self))))
        else {
            return;
        };
        AnimatedRect::update(self, &mut animated);
        let now = Instant::now();
        let path = RoundedRect::from_rect(
            vello::kurbo::Rect::from_origin_size(
                vello::kurbo::Point::new(area.x as f64, area.y as f64),
                vello::kurbo::Size::new(area.width as f64, area.height as f64),
            ),
            animated.radius.animate_wrapped(now) as f64,
        )
        .to_path(0.01);
        if animated.fill.is_none() && animated.stroke.is_none() {
            state.ui.cx().scene.fill(
                Fill::EvenOdd,
                Affine::IDENTITY,
                Color::BLACK.multiply_alpha(visible_amount),
                None,
                &path,
            )
        } else {
            if let Some(fill) = &animated.fill {
                state.ui.cx.as_mut().unwrap().scene.fill(
                    Fill::EvenOdd,
                    Affine::IDENTITY,
                    Color::rgba8(
                        fill.r.animate_wrapped(now).0,
                        fill.g.animate_wrapped(now).0,
                        fill.b.animate_wrapped(now).0,
                        255,
                    )
                    .multiply_alpha(visible_amount),
                    None,
                    &path,
                )
            }
            if let Some((stroke, width)) = &animated.stroke {
                state.ui.cx().scene.stroke(
                    &Stroke::new(width.animate_wrapped(now) as f64),
                    Affine::IDENTITY,
                    &Brush::Solid(
                        Color::rgba8(
                            stroke.r.animate_wrapped(now).0,
                            stroke.g.animate_wrapped(now).0,
                            stroke.b.animate_wrapped(now).0,
                            255,
                        )
                        .multiply_alpha(visible_amount),
                    ),
                    None,
                    &path,
                );
            }
        }
        state
            .ui
            .cx()
            .view_state
            .insert(self.id, AnimatedView::Rect(animated));
    }
    // fn id(&self) -> &u64 {
    //     &self.id
    // }
    // fn easing(&self) -> backer::Easing {
    //     self.easing.unwrap_or(backer::Easing::EaseOut)
    // }
    // fn duration(&self) -> f32 {
    //     self.duration.unwrap_or(200.)
    // }
    // fn delay(&self) -> f32 {
    //     self.delay
    // }
}

impl<'s, State> ViewTrait<'s, State> for Rect {
    fn view(self, _ui: &mut RcUi<State>, node: Node<'s, RcUi<State>>) -> Node<'s, RcUi<State>> {
        node
    }
}
