use crate::animated_color::{AnimatedColor, AnimatedU8};
use crate::gestures::EditInteraction;
use crate::{
    dynamic_node, id, rect, Binding, Editor, DEFAULT_CORNER_ROUNDING, DEFAULT_FG_COLOR,
    DEFAULT_FONT_SIZE, DEFAULT_PADDING,
};
use crate::{
    ui::RcUi,
    view::{AnimatedView, View, ViewType},
    DEFAULT_DURATION, DEFAULT_EASING,
};
use backer::nodes::{space, stack};
use backer::{models::*, Node};
use lilt::{Animated, Easing};
use parley::{
    AlignmentOptions, FontStack, Layout, LineHeight, PlainEditor, PositionedLayoutItem,
    StyleProperty, TextStyle,
};
use std::time::Instant;
use vello_svg::vello::kurbo::Point;
use vello_svg::vello::peniko::color::AlphaColor;
use vello_svg::vello::peniko::Brush;
use vello_svg::vello::{
    kurbo::Affine,
    peniko::{Color, Fill},
};

pub fn text<State>(id: u64, text: impl AsRef<str> + 'static) -> Text<State> {
    Text {
        id,
        state: Binding::constant(TextState {
            text: text.as_ref().to_string(),
            editing: false,
        }),
        font_size: DEFAULT_FONT_SIZE,
        // font: None,
        fill: DEFAULT_FG_COLOR,
        easing: None,
        duration: None,
        delay: 0.,
        alignment: TextAlign::Centered,
        editable: false,
        line_height: 1.,
    }
}

pub fn text_field<State>(id: u64, state: Binding<State, TextState>) -> Text<State> {
    Text {
        id,
        state,
        font_size: DEFAULT_FONT_SIZE,
        // font: None,
        fill: DEFAULT_FG_COLOR,
        easing: None,
        duration: None,
        delay: 0.,
        alignment: TextAlign::Centered,
        editable: true,
        line_height: 1.,
    }
}

#[derive(Debug)]
pub struct Text<State> {
    pub(crate) id: u64,
    pub(crate) state: Binding<State, TextState>,
    pub(crate) fill: Color,
    pub(crate) font_size: u32,
    pub(crate) alignment: TextAlign,
    pub(crate) editable: bool,
    // font: Option<font::Id>,
    pub(crate) easing: Option<Easing>,
    pub(crate) duration: Option<f32>,
    pub(crate) delay: f32,
    pub(crate) line_height: f32,
}

impl<State> Clone for Text<State> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            state: self.state.clone(),
            fill: self.fill,
            font_size: self.font_size,
            alignment: self.alignment,
            editable: self.editable,
            // font: self.font,
            easing: self.easing,
            duration: self.duration,
            delay: self.delay,
            line_height: self.line_height,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TextState {
    pub text: String,
    pub editing: bool,
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

impl<State> Text<State> {
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
    pub fn editable(mut self) -> Self {
        self.editable = true;
        self
    }
    // pub(crate) fn font(mut self, font_id: font::Id) -> Self {
    //     self.font = Some(font_id);
    //     self
    // }
    pub fn view(self) -> View<State, ()>
    where
        State: 'static,
    {
        if self.editable {
            let binding = self.state.clone();
            let on_click = {
                let binding = binding.clone();
                move |ui: &mut RcUi<State>, _, _| {
                    let editing = binding.get(&ui.ui.state).editing;
                    if !editing {
                        binding.update(&mut ui.ui.state, |s| s.editing = true);
                    }
                }
            };
            View {
                view_type: ViewType::Text(self),
                gesture_handlers: Vec::new(),
            }
            .on_click_system(on_click)
            .on_edit({
                move |state, edit| {
                    binding.update(state, move |s| match edit.clone() {
                        EditInteraction::Update(text) => s.text = text.clone(),
                        EditInteraction::End => s.editing = false,
                    });
                }
            })
        } else {
            View {
                view_type: ViewType::Text(self),
                gesture_handlers: Vec::new(),
            }
        }
    }
    pub fn finish<'n>(self) -> Node<'n, RcUi<State>>
    where
        State: 'static,
    {
        let binding = self.state.clone();
        let id = self.id;
        let editable = self.editable;
        stack(vec![self
            .view()
            .finish()
            .pad(if editable { DEFAULT_PADDING } else { 0. })
            .attach_under(if editable {
                dynamic_node({
                    move |s| {
                        rect(id!(id))
                            .fill(AlphaColor::from_rgb8(50, 50, 50))
                            .stroke(
                                if binding.get(s).editing {
                                    AlphaColor::from_rgb8(113, 70, 232)
                                } else {
                                    AlphaColor::from_rgb8(60, 60, 60)
                                },
                                3.,
                            )
                            .corner_rounding(DEFAULT_CORNER_ROUNDING)
                            .finish()
                    }
                })
            } else {
                space()
            })])
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AnimatedText {
    pub(crate) fill: AnimatedColor,
}

impl AnimatedText {
    pub(crate) fn update<State>(now: Instant, from: &Text<State>, existing: &mut AnimatedText) {
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
    pub(crate) fn new_from<State>(from: &Text<State>) -> Self {
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

impl<State> Text<State> {
    pub(crate) fn draw(
        &mut self,
        area: Area,
        state: &mut RcUi<State>,
        visible: bool,
        visible_amount: f32,
    ) {
        if !visible && visible_amount == 0. {
            return;
        }
        let editing = self.state.get(&state.ui.state).editing;
        if editing && state.ui.editor.is_none() {
            let mut editor = PlainEditor::new(self.font_size as f32);
            editor.set_text(&self.state.get(&state.ui.state).text);
            editor.set_scale(state.ui.cx().display_scale as f32);
            let styles = editor.edit_styles();
            styles.insert(StyleProperty::LineHeight(LineHeight::Absolute(
                self.line_height,
            )));
            styles.insert(parley::FontFamily::Named("Rubik".into()).into());
            styles.insert(StyleProperty::Brush(self.fill.into()));
            editor.set_alignment(self.alignment.into());
            editor.set_width(Some(area.width));
            let mut editor = Editor {
                editor,
                last_click_time: Default::default(),
                click_count: Default::default(),
                pointer_down: Default::default(),
                cursor_pos: Default::default(),
                cursor_visible: Default::default(),
                modifiers: Default::default(),
                start_time: Default::default(),
                blink_period: Default::default(),
            };
            state.ui.cx().with_font_layout_ctx_passthrough(
                &mut editor,
                |layout_cx, font_cx, editor| {
                    editor.mouse_moved(
                        Point::new(area.width as f64, area.height as f64),
                        layout_cx,
                        font_cx,
                    );
                    editor.mouse_pressed(layout_cx, font_cx);
                    editor.mouse_released();
                },
            );
            state.ui.editor = Some((self.id, Area::default(), editor, true));
        }
        if let Some((id, ref mut edit_area, _, _)) = &mut state.ui.editor {
            if *id == self.id {
                *edit_area = area;
                return;
            }
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

impl<'s, State> Text<State> {
    pub(crate) fn current_layout(
        &self,
        available_width: f32,
        ui: &mut RcUi<State>,
    ) -> Layout<Brush> {
        let current_text = self.state.get(&ui.ui.state).text;
        if let Some((_, _, layout)) = ui.ui.cx().layout_cache.get(&self.id).and_then(|cached| {
            cached
                .iter()
                .find(|(text, width, _)| *text == current_text && *width == available_width)
        }) {
            layout.clone()
        } else {
            let scale = ui.ui.cx().display_scale as f32;
            let mut layout = ui.ui.cx().with_font_layout_ctx(|layout_cx, font_cx| {
                let font_stack = FontStack::Single(parley::FontFamily::Named("Rubik".into()));
                let mut builder = layout_cx.tree_builder(
                    font_cx,
                    scale,
                    true,
                    &TextStyle {
                        brush: Brush::Solid(AlphaColor::WHITE),
                        font_stack,
                        line_height: LineHeight::Absolute(self.line_height),
                        font_size: self.font_size as f32,
                        ..Default::default()
                    },
                );
                builder.push_text(&current_text);
                builder.build().0
            });
            layout.break_all_lines(Some(available_width));
            layout.align(
                Some(available_width),
                self.alignment.into(),
                AlignmentOptions {
                    align_when_overflowing: true,
                },
            );
            let entry = ui.ui.cx().layout_cache.entry(self.id).or_insert(vec![(
                current_text.clone(),
                available_width,
                layout.clone(),
            )]);
            entry.push((current_text.clone(), available_width, layout.clone()));
            if entry.len() > 2 {
                entry.remove(0);
            }
            layout
        }
    }
    pub(crate) fn create_node(
        self,
        _ui: &mut RcUi<State>,
        node: Node<'s, RcUi<State>>,
    ) -> Node<'s, RcUi<State>>
    where
        State: 'static,
    {
        node.dynamic_height(move |width, ui| self.current_layout(width, ui).height())
    }
}
