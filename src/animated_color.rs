use lilt::{Animated, FloatRepresentable, Interpolable};
use std::time::Instant;
use vello_svg::vello::peniko::color::{AlphaColor, Srgb};

#[derive(Debug, Clone)]
pub(crate) struct AnimatedColor {
    pub(crate) r: Animated<AnimatedU8, Instant>,
    pub(crate) g: Animated<AnimatedU8, Instant>,
    pub(crate) b: Animated<AnimatedU8, Instant>,
}

impl AnimatedColor {
    pub(crate) fn transition(&mut self, to: AlphaColor<Srgb>, now: Instant) {
        self.r.transition(AnimatedU8(to.to_rgba8().r), now);
        self.g.transition(AnimatedU8(to.to_rgba8().g), now);
        self.b.transition(AnimatedU8(to.to_rgba8().b), now);
    }
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
