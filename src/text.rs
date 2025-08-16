use crate::animated_color::{AnimatedColor, AnimatedU8};
use crate::app::AppState;
use crate::draw_layout::draw_layout;
use crate::{
    DEFAULT_DURATION, DEFAULT_EASING,
    view::{AnimatedView, View, ViewType},
};
use crate::{DEFAULT_FG, DEFAULT_FG_COLOR, DEFAULT_FONT_FAMILY, DEFAULT_FONT_SIZE};
use backer::{Node, models::*};
use lilt::{Animated, Easing};
use parley::{Alignment, AlignmentOptions, FontStack, FontWeight, Layout, LineHeight, TextStyle};
use std::fmt::Debug;
use std::time::Instant;
use vello_svg::vello::peniko::Brush;
use vello_svg::vello::{kurbo::Affine, peniko::Color};

pub fn text(id: u64, text: impl AsRef<str> + 'static) -> Text {
    Text {
        id,
        string: text.as_ref().to_string(),
        font_size: DEFAULT_FONT_SIZE,
        font_weight: FontWeight::NORMAL,
        font_family: None,
        // font: None,
        fill: DEFAULT_FG_COLOR,
        easing: None,
        duration: None,
        delay: 0.,
        alignment: Alignment::Middle,
        line_height: 1.,
        wrap: false,
    }
}

pub struct Text {
    pub(crate) id: u64,
    pub(crate) string: String,
    pub(crate) fill: Color,
    pub(crate) font_size: u32,
    pub(crate) font_weight: FontWeight,
    pub(crate) font_family: Option<String>,
    pub(crate) alignment: Alignment,
    pub(crate) easing: Option<Easing>,
    pub(crate) duration: Option<f32>,
    pub(crate) delay: f32,
    pub(crate) line_height: f32,
    pub(crate) wrap: bool,
}

impl Debug for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Text")
            .field("id", &self.id)
            .field("state", &self.string)
            .field("fill", &self.fill)
            .field("font_size", &self.font_size)
            .field("font_weight", &self.font_weight)
            .field("alignment", &self.alignment)
            .field("easing", &self.easing)
            .field("duration", &self.duration)
            .field("delay", &self.delay)
            .field("line_height", &self.line_height)
            .field("wrap", &self.wrap)
            .finish()
    }
}

impl Clone for Text {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            string: self.string.clone(),
            fill: self.fill,
            font_size: self.font_size,
            font_weight: self.font_weight,
            font_family: self.font_family.clone(),
            alignment: self.alignment,
            easing: self.easing,
            duration: self.duration,
            delay: self.delay,
            line_height: self.line_height,
            wrap: self.wrap,
        }
    }
}

impl Text {
    pub fn fill(mut self, color: Color) -> Self {
        self.fill = color;
        self
    }
    pub fn font_size(mut self, size: u32) -> Self {
        self.font_size = size;
        self
    }
    pub fn font_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = weight;
        self
    }
    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = Some(family.into());
        self
    }
    pub fn align(mut self, align: Alignment) -> Self {
        self.alignment = align;
        self
    }
    pub fn wrap(mut self) -> Self {
        self.wrap = true;
        self
    }
}

impl Text {
    pub fn view<State>(self) -> View<State>
    where
        State: 'static,
    {
        View {
            view_type: ViewType::Text(self),
            gesture_handlers: Vec::new(),
        }
    }
    pub fn finish<'n, State>(self) -> Node<'n, State, AppState<State>>
    where
        State: 'static,
    {
        self.view().finish()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AnimatedText {
    pub(crate) fill: AnimatedColor,
}

impl AnimatedText {
    pub(crate) fn update(now: Instant, from: &Text, existing: &mut AnimatedText) {
        existing
            .fill
            .r
            .transition(AnimatedU8(from.fill.to_rgba8().r), now);
        existing
            .fill
            .g
            .transition(AnimatedU8(from.fill.to_rgba8().g), now);
        existing
            .fill
            .b
            .transition(AnimatedU8(from.fill.to_rgba8().b), now);
    }
    pub(crate) fn new_from(from: &Text) -> Self {
        AnimatedText {
            fill: AnimatedColor {
                r: Animated::new(AnimatedU8(from.fill.to_rgba8().r))
                    .easing(from.easing.unwrap_or(DEFAULT_EASING))
                    .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                    .delay(from.delay),
                g: Animated::new(AnimatedU8(from.fill.to_rgba8().g))
                    .easing(from.easing.unwrap_or(DEFAULT_EASING))
                    .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                    .delay(from.delay),
                b: Animated::new(AnimatedU8(from.fill.to_rgba8().b))
                    .easing(from.easing.unwrap_or(DEFAULT_EASING))
                    .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                    .delay(from.delay),
                a: Animated::new(AnimatedU8(from.fill.to_rgba8().a))
                    .easing(from.easing.unwrap_or(DEFAULT_EASING))
                    .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                    .delay(from.delay),
            },
        }
    }
}

impl Text {
    pub(crate) fn draw<State>(
        &mut self,
        animated_area: Area,
        area: Area,
        state: &mut State,
        app: &mut AppState<State>,
        visible: bool,
        visible_amount: f32,
    ) {
        if !visible && visible_amount == 0. {
            return;
        }

        let AnimatedView::Text(mut animated) = app
            .view_state
            .remove(&self.id)
            .unwrap_or(AnimatedView::Text(Box::new(AnimatedText::new_from(self))))
        else {
            return;
        };
        AnimatedText::update(app.now, self, &mut animated);
        let anim_fill = Color::from_rgba8(
            animated.fill.r.animate_wrapped(app.now).0,
            animated.fill.g.animate_wrapped(app.now).0,
            animated.fill.b.animate_wrapped(app.now).0,
            animated.fill.a.animate_wrapped(app.now).0,
        )
        .multiply_alpha(visible_amount);

        let layout = self.current_layout(anim_fill, area.width, true, state, app);

        let transform = Affine::translate((animated_area.x as f64, animated_area.y as f64))
            .then_scale(app.scale_factor);
        draw_layout(Some(anim_fill), transform, &layout, &mut app.scene);
        app.view_state.insert(self.id, AnimatedView::Text(animated));
    }
}

impl<'s> Text {
    pub(crate) fn current_layout<State>(
        &self,
        current_fill: Color,
        available_width: f32,
        cache: bool,
        _state: &mut State,
        app: &mut AppState<State>,
    ) -> Layout<Brush> {
        let text = self.string.clone();
        let current_text = if text.is_empty() {
            " ".to_string()
        } else {
            text
        };
        if !current_text.is_empty()
            && let Some((_, _, layout)) = app.layout_cache.get(&self.id).and_then(|cached| {
                cached
                    .iter()
                    .find(|(text, width, _)| *text == current_text && *width == available_width)
            })
        {
            layout.clone()
        } else {
            let mut builder = app.layout_cx.tree_builder(
                &mut app.font_cx,
                1.,
                true,
                &TextStyle {
                    brush: Brush::Solid(current_fill),
                    font_stack: FontStack::Single(parley::FontFamily::Named(
                        self.font_family
                            .clone()
                            .unwrap_or(DEFAULT_FONT_FAMILY.to_string())
                            .into(),
                    )),
                    font_weight: self.font_weight,
                    line_height: LineHeight::FontSizeRelative(self.line_height),
                    font_size: self.font_size as f32,
                    overflow_wrap: parley::OverflowWrap::Anywhere,
                    ..Default::default()
                },
            );
            builder.push_text(&current_text);
            let mut layout = builder.build().0;
            layout.break_all_lines(Some(available_width));
            layout.align(
                Some(available_width),
                self.alignment,
                AlignmentOptions {
                    align_when_overflowing: true,
                },
            );
            if cache {
                let entry = app.layout_cache.entry(self.id).or_insert(vec![(
                    current_text.clone(),
                    available_width,
                    layout.clone(),
                )]);
                entry.push((current_text.clone(), available_width, layout.clone()));
                if entry.len() > 2 {
                    entry.remove(0);
                }
            }
            layout
        }
    }
    pub(crate) fn create_node<State>(
        self,
        state: &mut State,
        app: &mut AppState<State>,
        node: Node<'s, State, AppState<State>>,
    ) -> Node<'s, State, AppState<State>>
    where
        State: 'static,
    {
        if self.wrap {
            node.dynamic_height(move |width, state, app| {
                self.current_layout(DEFAULT_FG, width, false, state, app)
                    .height()
            })
        } else {
            let layout = self.current_layout(DEFAULT_FG, 10000., false, state, app);
            node.height(layout.height()).width(layout.width().max(10.))
        }
    }
}
