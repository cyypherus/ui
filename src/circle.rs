use crate::animated_color::{AnimatedColor, AnimatedU8};
use crate::ui::RcUi;
use crate::view::{AnimatedView, View, ViewTrait, ViewType};
use crate::{GestureHandler, DEFAULT_DURATION, DEFAULT_EASING};
use backer::{models::Area, Node};
use lilt::{Animated, Easing};
use std::time::Instant;
use vello_svg::vello::kurbo::{Point, Stroke};
use vello_svg::vello::peniko::{Brush, Fill};
use vello_svg::vello::{kurbo::Affine, peniko::Color};

#[derive(Debug, Clone)]
pub struct Circle {
    pub(crate) id: u64,
    pub(crate) fill: Option<Color>,
    pub(crate) stroke: Option<(Color, f32)>,
    pub(crate) easing: Option<Easing>,
    pub(crate) duration: Option<f32>,
    pub(crate) delay: f32,
}

#[derive(Debug, Clone)]
pub(crate) struct AnimatedCircle {
    pub(crate) fill: Option<AnimatedColor>,
    pub(crate) stroke: Option<(AnimatedColor, Animated<f32, Instant>)>,
}

impl AnimatedCircle {
    pub(crate) fn update(now: Instant, from: &Circle, existing: &mut AnimatedCircle) {
        if let (Some(existing_fill), Some(new_fill)) = (&mut existing.fill, from.fill) {
            existing_fill.transition(now, new_fill);
        }
        if let (Some((existing_stroke, existing_width)), Some((new_stroke, new_width))) =
            (&mut existing.stroke, from.stroke)
        {
            existing_stroke.transition(now, new_stroke);
            existing_width.transition(new_width, now);
        }
    }
    pub(crate) fn new_from(from: &Circle) -> Self {
        AnimatedCircle {
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

pub fn circle(id: u64) -> Circle {
    Circle {
        id,
        fill: None,
        stroke: None,
        easing: None,
        duration: None,
        delay: 0.,
    }
}

impl Circle {
    pub fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }
    pub fn stroke(mut self, color: Color, line_width: f32) -> Self {
        self.stroke = Some((color, line_width));
        self
    }
    pub fn view<State>(self) -> View<State> {
        View {
            view_type: ViewType::Circle(self),
            gesture_handler: GestureHandler {
                on_click: None,
                on_drag: None,
                on_hover: None,
                on_key: None,
            },
        }
    }
}

impl Circle {
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
        let AnimatedView::Circle(mut animated) = state
            .ui
            .cx()
            .view_state
            .remove(&self.id)
            .unwrap_or(AnimatedView::Circle(Box::new(AnimatedCircle::new_from(
                self,
            ))))
        else {
            return;
        };
        AnimatedCircle::update(state.ui.now, self, &mut animated);
        let radius = f32::min(area.width, area.height) * 0.5;
        let path = vello_svg::vello::kurbo::Circle::new(
            Point::new(
                (area.x + (area.width * 0.5)) as f64,
                (area.y + (area.height * 0.5)) as f64,
            ),
            radius as f64,
        );
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
                    Color::from_rgba8(
                        fill.r.animate_wrapped(state.ui.now).0,
                        fill.g.animate_wrapped(state.ui.now).0,
                        fill.b.animate_wrapped(state.ui.now).0,
                        255,
                    )
                    .multiply_alpha(visible_amount),
                    None,
                    &path,
                )
            }
            if let Some((stroke, width)) = &animated.stroke {
                let now = state.ui.now;
                state.ui.cx().scene.stroke(
                    &Stroke::new(width.animate_wrapped(now) as f64),
                    Affine::IDENTITY,
                    &Brush::Solid(
                        Color::from_rgba8(
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
            .insert(self.id, AnimatedView::Circle(animated));
    }
}

impl<'s, State> ViewTrait<'s, State> for Circle {
    fn create_node(
        self,
        _ui: &mut RcUi<State>,
        node: Node<'s, RcUi<State>>,
    ) -> Node<'s, RcUi<State>> {
        node
    }
}
