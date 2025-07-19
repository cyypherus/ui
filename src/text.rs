use crate::animated_color::{AnimatedColor, AnimatedU8};
use crate::app::AppState;
use crate::gestures::EditInteraction;
use crate::{
    id, rect, Binding, Editor, DEFAULT_CORNER_ROUNDING, DEFAULT_FG_COLOR, DEFAULT_FONT_SIZE,
    DEFAULT_PADDING,
};
use crate::{
    view::{AnimatedView, View, ViewType},
    DEFAULT_DURATION, DEFAULT_EASING,
};
use backer::nodes::{dynamic, space, stack};
use backer::{models::*, Node};
use lilt::{Animated, Easing};
use parley::{
    AlignmentOptions, FontStack, Layout, LineHeight, PlainEditor, PositionedLayoutItem,
    StyleProperty, TextStyle,
};
use std::time::Instant;
use vello_svg::vello::kurbo::{Point, Vec2};
use vello_svg::vello::peniko::color::AlphaColor;
use vello_svg::vello::peniko::Brush;
use vello_svg::vello::{
    kurbo::Affine,
    peniko::{Color, Fill},
};

pub(crate) const DEEP_PURP: Color = AlphaColor::from_rgb8(113, 70, 232);

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
        background_fill: None,
        background_stroke: None,
        background_corner_rounding: DEFAULT_CORNER_ROUNDING,
        background_padding: DEFAULT_PADDING,
        wrap: false,
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
        background_fill: Some(AlphaColor::from_rgb8(50, 50, 50)),
        background_stroke: Some((AlphaColor::from_rgb8(60, 60, 60), DEEP_PURP, 3.)),
        background_corner_rounding: DEFAULT_CORNER_ROUNDING,
        background_padding: DEFAULT_PADDING,
        wrap: false,
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
    pub(crate) background_fill: Option<Color>,
    pub(crate) background_stroke: Option<(Color, Color, f32)>, // (normal, focused, width)
    pub(crate) background_corner_rounding: f32,
    pub(crate) background_padding: f32,
    pub(crate) wrap: bool,
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
            background_fill: self.background_fill,
            background_stroke: self.background_stroke,
            background_corner_rounding: self.background_corner_rounding,
            background_padding: self.background_padding,
            wrap: self.wrap,
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
    pub fn background_fill(mut self, color: Option<Color>) -> Self {
        self.background_fill = color;
        self
    }
    pub fn background_stroke(mut self, normal: Color, focused: Color, width: f32) -> Self {
        self.background_stroke = Some((normal, focused, width));
        self
    }
    pub fn no_background_stroke(mut self) -> Self {
        self.background_stroke = None;
        self
    }
    pub fn background_corner_rounding(mut self, rounding: f32) -> Self {
        self.background_corner_rounding = rounding;
        self
    }
    pub fn background_padding(mut self, padding: f32) -> Self {
        self.background_padding = padding;
        self
    }
    // pub(crate) fn font(mut self, font_id: font::Id) -> Self {
    //     self.font = Some(font_id);
    //     self
    // }
    pub fn view(self) -> View<State>
    where
        State: 'static,
    {
        if self.editable {
            let binding = self.state.clone();
            View {
                view_type: ViewType::Text(self),
                gesture_handlers: Vec::new(),
            }
            .on_click({
                let binding = binding.clone();
                move |state: &mut State, _, _, _| {
                    let editing = binding.get(state).editing;
                    if !editing {
                        binding.update(state, |s| s.editing = true);
                    }
                }
            })
            .on_edit({
                move |state, _app, edit| {
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
    pub fn finish<'n>(self) -> Node<'n, State, AppState<State>>
    where
        State: 'static,
    {
        let binding = self.state.clone();
        let id = self.id;
        let editable = self.editable;
        let bg_fill = self.background_fill;
        let bg_stroke = self.background_stroke;
        let bg_rounding = self.background_corner_rounding;
        let bg_padding = self.background_padding;
        stack(vec![self
            .view()
            .finish()
            .pad(if editable { bg_padding } else { 0. })
            .attach_under(
                if editable && (bg_fill.is_some() || bg_stroke.is_some()) {
                    dynamic({
                        move |s, _a| {
                            let mut rect_node = rect(id!(id));
                            if let Some(fill) = bg_fill {
                                rect_node = rect_node.fill(fill);
                            }
                            if let Some((normal, focused, width)) = bg_stroke {
                                rect_node = rect_node.stroke(
                                    if binding.get(s).editing {
                                        focused
                                    } else {
                                        normal
                                    },
                                    width,
                                );
                            }
                            rect_node.corner_rounding(bg_rounding).finish()
                        }
                    })
                } else {
                    space()
                },
            )])
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
        state: &mut State,
        app: &mut AppState<State>,
        visible: bool,
        visible_amount: f32,
    ) {
        if !visible && visible_amount == 0. {
            return;
        }
        let editing = self.state.get(state).editing;
        if editing && app.editor.is_none() {
            let mut editor = PlainEditor::new(self.font_size as f32);
            editor.set_text(&self.state.get(state).text);
            let styles = editor.edit_styles();
            styles.insert(StyleProperty::LineHeight(LineHeight::FontSizeRelative(
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

            let AppState {
                font_cx, layout_cx, ..
            } = app;
            if let Some(pos) = app.cursor_position {
                editor.mouse_moved(
                    Point::new(pos.x - area.x as f64, pos.y - area.y as f64),
                    layout_cx,
                    font_cx,
                );
                editor.mouse_pressed(layout_cx, font_cx);
            }
            app.editor = Some((self.id, area, editor, true, self.state.clone()));
        }
        if let Some((id, ref mut edit_area, _, _, _)) = &mut app.editor {
            if *id == self.id {
                *edit_area = area;
                return;
            }
        }
        let AnimatedView::Text(mut animated) = app
            .view_state
            .remove(&self.id)
            .unwrap_or(AnimatedView::Text(Box::new(AnimatedText::new_from(self))))
        else {
            return;
        };
        AnimatedText::update(app.now, self, &mut animated);
        let layout = self.current_layout(area.width, state, app);

        let anim_fill = Color::from_rgba8(
            animated.fill.r.animate_wrapped(app.now).0,
            animated.fill.g.animate_wrapped(app.now).0,
            animated.fill.b.animate_wrapped(app.now).0,
            animated.fill.a.animate_wrapped(app.now).0,
        )
        .multiply_alpha(visible_amount);
        let transform =
            Affine::translate((area.x as f64, area.y as f64)).then_scale(app.scale_factor);
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
                app.scene
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
        app.view_state.insert(self.id, AnimatedView::Text(animated));
    }
}

impl<'s, State> Text<State> {
    pub(crate) fn current_layout(
        &self,
        available_width: f32,
        state: &mut State,
        app: &mut AppState<State>,
    ) -> Layout<Brush> {
        let available_width = available_width;
        let current_text = self.state.get(state).text;
        if let Some((_, _, layout)) = app.layout_cache.get(&self.id).and_then(|cached| {
            cached
                .iter()
                .find(|(text, width, _)| *text == current_text && *width == available_width)
        }) {
            layout.clone()
        } else {
            let font_stack = FontStack::Single(parley::FontFamily::Named("Rubik".into()));
            let mut builder = app.layout_cx.tree_builder(
                &mut app.font_cx,
                1.,
                true,
                &TextStyle {
                    brush: Brush::Solid(AlphaColor::WHITE),
                    font_stack,
                    line_height: LineHeight::FontSizeRelative(self.line_height),
                    font_size: self.font_size as f32,
                    ..Default::default()
                },
            );
            builder.push_text(&current_text);
            let mut layout = builder.build().0;

            layout.break_all_lines(Some(available_width));
            layout.align(
                Some(available_width),
                self.alignment.into(),
                AlignmentOptions {
                    align_when_overflowing: true,
                },
            );
            let entry = app.layout_cache.entry(self.id).or_insert(vec![(
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
        state: &mut State,
        app: &mut AppState<State>,
        node: Node<'s, State, AppState<State>>,
    ) -> Node<'s, State, AppState<State>>
    where
        State: 'static,
    {
        if self.wrap {
            node.dynamic_height(move |width, state, app| {
                self.current_layout(width, state, app).height()
            })
        } else {
            let layout = self.current_layout(10000., state, app);
            node.height(layout.height()).width(layout.width())
        }
    }
}
