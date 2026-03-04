use backer::Area;
use std::{fmt::Debug, rc::Rc};
use vello_svg::vello::{
    kurbo::Stroke,
    peniko::{Brush, Gradient},
};

use crate::Color;

#[derive(Clone)]
pub enum BrushSource<V> {
    Static(Brush),
    Dynamic(Rc<dyn Fn(Area, &V) -> Brush>),
}

impl<V> BrushSource<V> {
    pub(crate) fn resolve(&self, area: Area, state: &V) -> Brush {
        match self {
            BrushSource::Static(brush) => brush.clone(),
            BrushSource::Dynamic(func) => func(area, state),
        }
    }

    pub fn resolve_to_stateless(&self, state: &V) -> BrushSource<()>
    where
        V: Clone + 'static,
    {
        match self {
            BrushSource::Static(brush) => BrushSource::Static(brush.clone()),
            BrushSource::Dynamic(func) => {
                let func = func.clone();
                let state = state.clone();
                BrushSource::Dynamic(Rc::new(move |area, _| func(area, &state)))
            }
        }
    }
}

impl<V> From<Brush> for BrushSource<V> {
    fn from(value: Brush) -> Self {
        BrushSource::Static(value)
    }
}

impl<V> From<Color> for BrushSource<V> {
    fn from(value: Color) -> Self {
        BrushSource::Static(Brush::Solid(value))
    }
}

impl<V> From<Gradient> for BrushSource<V> {
    fn from(value: Gradient) -> Self {
        BrushSource::Static(Brush::Gradient(value))
    }
}

impl<T, V> From<T> for BrushSource<V>
where
    T: Fn(Area, &V) -> Brush + 'static,
{
    fn from(value: T) -> Self {
        BrushSource::Dynamic(Rc::new(value))
    }
}

impl<V> Debug for BrushSource<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrushSource::Static(brush) => write!(f, "Brush: {:?}", brush),
            BrushSource::Dynamic(_) => write!(f, "Dynamic <function>"),
        }
    }
}

#[derive(Clone)]
pub enum StrokeSource<V> {
    Static(Stroke),
    Dynamic(Rc<dyn Fn(Area, &V) -> Stroke>),
}

impl<V> StrokeSource<V> {
    pub(crate) fn resolve(&self, area: Area, state: &V) -> Stroke {
        match self {
            StrokeSource::Static(stroke) => stroke.clone(),
            StrokeSource::Dynamic(func) => func(area, state),
        }
    }
}

impl<V> From<Stroke> for StrokeSource<V> {
    fn from(value: Stroke) -> Self {
        StrokeSource::Static(value)
    }
}

impl<T, V> From<T> for StrokeSource<V>
where
    T: Fn(Area, &V) -> Stroke + 'static,
{
    fn from(value: T) -> Self {
        StrokeSource::Dynamic(Rc::new(value))
    }
}

impl<V> From<f32> for StrokeSource<V> {
    fn from(value: f32) -> Self {
        StrokeSource::Static(Stroke::new(value as f64))
    }
}

impl<V> Debug for StrokeSource<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrokeSource::Static(stroke) => write!(f, "Stroke: {:?}", stroke),
            StrokeSource::Dynamic(_) => write!(f, "Dynamic <function>"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Style<V> {
    pub(crate) fill: Option<BrushSource<V>>,
    pub(crate) stroke: Option<(BrushSource<V>, StrokeSource<V>)>,
    pub(crate) rounding: f32,
    pub(crate) padding: f32,
}

impl<V> Default for Style<V> {
    fn default() -> Self {
        Self {
            fill: None,
            stroke: None,
            rounding: 0.,
            padding: 0.,
        }
    }
}

pub trait BackgroundStyled {
    type V;
    fn background(&mut self) -> &mut Style<Self::V>;
}

pub trait BackgroundStylable: Sized + BackgroundStyled {
    fn background_fill(mut self, fill: impl Into<BrushSource<Self::V>>) -> Self {
        self.background().fill = Some(fill.into());
        self
    }
    fn background_stroke(
        mut self,
        brush: impl Into<BrushSource<Self::V>>,
        stroke: impl Into<StrokeSource<Self::V>>,
    ) -> Self {
        self.background().stroke = Some((brush.into(), stroke.into()));
        self
    }
    fn background_corner_rounding(mut self, rounding: f32) -> Self {
        self.background().rounding = rounding;
        self
    }
    fn background_padding(mut self, padding: f32) -> Self {
        self.background().padding = padding;
        self
    }
}
