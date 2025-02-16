use crate::animated_color::{AnimatedColor, AnimatedU8};
use crate::DEFAULT_FONT_SIZE;
use crate::{
    ui::RcUi,
    view::{AnimatedView, View, ViewType},
    GestureHandler, DEFAULT_DURATION, DEFAULT_EASING,
};
use backer::{models::*, Node};
use lilt::{Animated, Easing};
use parley::{FontStack, Layout, PositionedLayoutItem, TextStyle};
use std::time::Instant;
use vello_svg::vello::{
    kurbo::Affine,
    peniko::{Color, Fill},
};

pub fn text(id: u64, text: impl AsRef<str> + 'static) -> Text {
    Text {
        id,
        text: text.as_ref().to_owned(),
        font_size: DEFAULT_FONT_SIZE,
        // font: None,
        fill: Color::BLACK,
        easing: None,
        duration: None,
        delay: 0.,
        alignment: TextAlign::Centered,
    }
}

#[derive(Debug, Clone)]
pub struct Text {
    pub(crate) id: u64,
    text: String,
    fill: Color,
    font_size: u32,
    alignment: TextAlign,
    // font: Option<font::Id>,
    pub(crate) easing: Option<Easing>,
    pub(crate) duration: Option<f32>,
    pub(crate) delay: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum TextAlign {
    Leading,
    Centered,
    Trailing,
    Justified,
}

impl From<TextAlign> for parley::Alignment {
    fn from(value: TextAlign) -> Self {
        match value {
            TextAlign::Leading => parley::Alignment::Start,
            TextAlign::Centered => parley::Alignment::Middle,
            TextAlign::Trailing => parley::Alignment::End,
            TextAlign::Justified => parley::Alignment::Justified,
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
    pub fn align(mut self, align: TextAlign) -> Self {
        self.alignment = align;
        self
    }
    // pub(crate) fn font(mut self, font_id: font::Id) -> Self {
    //     self.font = Some(font_id);
    //     self
    // }
    pub fn view<State>(self) -> View<State, ()> {
        View {
            view_type: ViewType::Text(self),
            gesture_handler: GestureHandler {
                on_click: None,
                on_drag: None,
                on_hover: None,
                on_key: None,
                on_scroll: None,
            },
        }
    }
    pub fn finish<'n, State: 'n>(self) -> Node<'n, RcUi<State>> {
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
        area: Area,
        state: &mut RcUi<State>,
        visible: bool,
        visible_amount: f32,
    ) {
        if !visible && visible_amount == 0. {
            return;
        }
        let AnimatedView::Text(mut animated) = state
            .ui
            .cx()
            .view_state
            .remove(&self.id)
            .unwrap_or(AnimatedView::Text(Box::new(AnimatedText::new_from(self))))
        else {
            return;
        };
        AnimatedText::update(state.ui.now, self, &mut animated);
        let layout = self.current_layout(area.width, state);

        let anim_fill = Color::from_rgba8(
            animated.fill.r.animate_wrapped(state.ui.now).0,
            animated.fill.g.animate_wrapped(state.ui.now).0,
            animated.fill.b.animate_wrapped(state.ui.now).0,
            animated.fill.a.animate_wrapped(state.ui.now).0,
        )
        .multiply_alpha(visible_amount);
        let transform = Affine::translate((area.x as f64, area.y as f64));
        for line in layout.lines() {
            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                    continue;
                };
                let mut x = glyph_run.offset();
                let y = glyph_run.baseline();
                let run = glyph_run.run();
                let font = run.font();
                let font_size = run.font_size();
                let synthesis = run.synthesis();
                let glyph_xform = synthesis
                    .skew()
                    .map(|angle| Affine::skew(angle.to_radians().tan() as f64, 0.0));
                state
                    .ui
                    .cx()
                    .scene
                    .draw_glyphs(font)
                    .brush(anim_fill)
                    .hint(true)
                    .transform(transform)
                    .glyph_transform(glyph_xform)
                    .font_size(font_size)
                    .normalized_coords(run.normalized_coords())
                    .draw(
                        Fill::NonZero,
                        glyph_run.glyphs().map(|glyph| {
                            let gx = x + glyph.x;
                            let gy = y - glyph.y;
                            x += glyph.advance;
                            vello_svg::vello::Glyph {
                                id: glyph.id as _,
                                x: gx,
                                y: gy,
                            }
                        }),
                    );
            }
        }
        state
            .ui
            .cx()
            .view_state
            .insert(self.id, AnimatedView::Text(animated));
    }
}

impl<'s> Text {
    pub(crate) fn current_layout<State>(
        &self,
        available_width: f32,
        ui: &mut RcUi<State>,
    ) -> Layout<[u8; 4]> {
        if let Some((_, _, layout)) = ui.ui.cx().layout_cache.get(&self.id).and_then(|cached| {
            cached
                .iter()
                .find(|(text, width, _)| *text == self.text && *width == available_width)
        }) {
            layout.clone()
        } else {
            let scale = ui.ui.cx().display_scale as f32;
            let mut layout = ui.ui.cx().with_font_layout_ctx(|layout_cx, font_cx| {
                let font_stack = FontStack::Single(parley::FontFamily::Named("Rubik".into()));
                let mut builder = layout_cx.tree_builder(
                    font_cx,
                    scale,
                    &TextStyle {
                        brush: [255, 0, 0, 0],
                        font_stack,
                        line_height: 1.3,
                        font_size: self.font_size as f32,
                        ..Default::default()
                    },
                );
                builder.push_text(&self.text);
                builder.build().0
            });
            layout.break_all_lines(Some(available_width));
            layout.align(Some(available_width), self.alignment.into(), true);
            let entry = ui.ui.cx().layout_cache.entry(self.id).or_insert(vec![(
                self.text.clone(),
                available_width,
                layout.clone(),
            )]);
            entry.push((self.text.clone(), available_width, layout.clone()));
            if entry.len() > 2 {
                entry.remove(0);
            }
            layout
        }
    }
    pub(crate) fn create_node<State>(
        self,
        _ui: &mut RcUi<State>,
        node: Node<'s, RcUi<State>>,
    ) -> Node<'s, RcUi<State>> {
        node.dynamic_height(move |width, ui| self.current_layout(width, ui).height())
    }
}
