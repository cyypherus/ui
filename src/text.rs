use crate::{
    rect::{AnimatedColor, AnimatedU8},
    ui::RcUi,
    view::{AnimatedView, View, ViewTrait, ViewType},
    GestureHandler, DEFAULT_DURATION, DEFAULT_EASING,
};
use backer::{models::*, Node};
use lilt::{Animated, Easing};
use parley::{FontStack, PositionedLayoutItem, TextStyle};
use std::{cell::RefCell, time::Instant};
use vello::{
    kurbo::Affine,
    peniko::{Color, Fill},
};

pub fn text(id: u64, text: impl AsRef<str> + 'static) -> Text {
    Text {
        id,
        text: text.as_ref().to_owned(),
        font_size: 40,
        // font: None,
        fill: Color::BLACK,
        easing: None,
        duration: None,
        delay: 0.,
    }
}

#[derive(Debug, Clone)]
pub struct Text {
    pub(crate) id: u64,
    text: String,
    fill: Color,
    font_size: u32,
    // font: Option<font::Id>,
    pub(crate) easing: Option<Easing>,
    pub(crate) duration: Option<f32>,
    pub(crate) delay: f32,
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
    // pub(crate) fn font(mut self, font_id: font::Id) -> Self {
    //     self.font = Some(font_id);
    //     self
    // }
    pub fn finish<State>(self) -> View<State> {
        View {
            view_type: ViewType::Text(self),
            gesture_handler: GestureHandler {
                on_click: None,
                on_drag: None,
                on_hover: None,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AnimatedText {
    pub(crate) fill: AnimatedColor,
}

impl AnimatedText {
    pub(crate) fn update(now: Instant, from: &Text, existing: &mut AnimatedText) {
        existing.fill.r.transition(AnimatedU8(from.fill.r), now);
        existing.fill.g.transition(AnimatedU8(from.fill.g), now);
        existing.fill.b.transition(AnimatedU8(from.fill.b), now);
    }
    pub(crate) fn new_from(from: &Text) -> Self {
        AnimatedText {
            fill: AnimatedColor {
                r: Animated::new(AnimatedU8(from.fill.r))
                    .easing(from.easing.unwrap_or(DEFAULT_EASING))
                    .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                    .delay(from.delay),
                g: Animated::new(AnimatedU8(from.fill.g))
                    .easing(from.easing.unwrap_or(DEFAULT_EASING))
                    .duration(from.duration.unwrap_or(DEFAULT_DURATION))
                    .delay(from.delay),
                b: Animated::new(AnimatedU8(from.fill.b))
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
        let now = Instant::now();
        let AnimatedView::Text(mut animated) = state
            .ui
            .borrow_mut()
            .cx()
            .view_state
            .remove(&self.id)
            .unwrap_or(AnimatedView::Text(Box::new(AnimatedText::new_from(self))))
        else {
            return;
        };
        AnimatedText::update(now, self, &mut animated);
        let mut layout =
            RefCell::borrow_mut(&state.ui)
                .cx()
                .with_font_layout_ctx(|layout_cx, font_cx| {
                    let font_stack = FontStack::Single(parley::FontFamily::Named("Rubik".into()));
                    let mut builder = layout_cx.tree_builder(
                        font_cx,
                        1.,
                        &TextStyle {
                            brush: [255, 0, 0, 0],
                            font_stack,
                            line_height: 1.3,
                            font_size: 16.0,
                            ..Default::default()
                        },
                    );
                    builder.push_text(&self.text);
                    builder.build().0
                });

        layout.break_all_lines(None);
        layout.align(None, parley::Alignment::Middle);

        let anim_fill = Color::rgba8(
            animated.fill.r.animate_wrapped(now).0,
            animated.fill.g.animate_wrapped(now).0,
            animated.fill.b.animate_wrapped(now).0,
            255,
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
                let coords = run
                    .normalized_coords()
                    .iter()
                    .map(|coord| vello::skrifa::instance::NormalizedCoord::from_bits(*coord))
                    .collect::<Vec<_>>();
                RefCell::borrow_mut(&state.ui)
                    .cx()
                    .scene
                    .draw_glyphs(font)
                    .brush(anim_fill)
                    .hint(true)
                    .transform(transform)
                    .glyph_transform(glyph_xform)
                    .font_size(font_size)
                    .normalized_coords(&coords)
                    .draw(
                        Fill::NonZero,
                        glyph_run.glyphs().map(|glyph| {
                            let gx = x + glyph.x;
                            let gy = y - glyph.y;
                            x += glyph.advance;
                            vello::Glyph {
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
            .borrow_mut()
            .cx()
            .view_state
            .insert(self.id, AnimatedView::Text(animated));
    }

    // fn id(&self) -> &u64 {
    //     &self.id
    // }
    // fn easing(&self) -> backer::Easing {
    //     backer::Easing::EaseOut
    // }
    // fn duration(&self) -> f32 {
    //     0.
    // }
    // fn delay(&self) -> f32 {
    //     0.
    // }
}

impl<'s, State> ViewTrait<'s, State> for Text {
    fn view(self, ui: &mut RcUi<State>, node: Node<'s, RcUi<State>>) -> Node<'s, RcUi<State>> {
        let mut layout =
            RefCell::borrow_mut(&ui.ui)
                .cx()
                .with_font_layout_ctx(|layout_cx, font_cx| {
                    let font_stack = FontStack::Single(parley::FontFamily::Named("Rubik".into()));
                    let mut builder = layout_cx.tree_builder(
                        font_cx,
                        1.,
                        &TextStyle {
                            brush: [255, 0, 0, 0],
                            font_stack,
                            line_height: 1.3,
                            font_size: 16.0,
                            ..Default::default()
                        },
                    );
                    builder.push_text(&self.text);
                    builder.build().0
                });
        layout.break_all_lines(None);
        layout.align(None, parley::Alignment::Middle);
        node.width(layout.full_width()).height(layout.height())
    }
}
