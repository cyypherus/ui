use std::fmt::Debug;
use std::rc::Rc;

use crate::app::{AppState, DrawItem, EditState};
use crate::rect::Rect;
use crate::shape::{Shape, ShapeType};
use crate::view::{View, ViewType};
use crate::{
    Binding, DEFAULT_CORNER_ROUNDING, DEFAULT_FG_COLOR, DEFAULT_FONT_FAMILY, DEFAULT_FONT_SIZE,
    DEFAULT_PADDING, DEFAULT_PURP, EditInteraction, Editor, Key, Text, rect,
};
use backer::Node;
use backer::models::Area;
use backer::nodes::*;
use lilt::Easing;
use parley::{Alignment, FontWeight, LineHeight, PlainEditor, StyleProperty};
use vello_svg::vello::kurbo::{Affine, Point};
use vello_svg::vello::peniko::color::palette::css::TRANSPARENT;
use vello_svg::vello::peniko::{Brush, Color};

#[derive(Debug, Clone, Default)]
pub struct TextState {
    pub text: String,
    pub editing: bool,
}

impl TextState {
    pub fn new(text: impl AsRef<str>) -> Self {
        Self {
            text: text.as_ref().to_string(),
            editing: false,
        }
    }
}

pub fn text_field<State>(id: u64, state: Binding<State, TextState>) -> TextField<State> {
    TextField {
        id,
        state,
        font_size: DEFAULT_FONT_SIZE,
        font_weight: FontWeight::NORMAL,
        font_family: None,
        fill: DEFAULT_FG_COLOR,
        easing: None,
        duration: None,
        delay: 0.,
        alignment: Alignment::Middle,
        editable: true,
        line_height: 1.,
        background_fill: Some(Color::from_rgb8(50, 50, 50)),
        background_stroke: Some((Color::from_rgb8(60, 60, 60), DEFAULT_PURP, 3.)),
        background_corner_rounding: DEFAULT_CORNER_ROUNDING,
        background_padding: DEFAULT_PADDING,
        wrap: false,
        cursor_fill: DEFAULT_PURP,
        highlight_fill: DEFAULT_PURP,
        on_edit: None,
        esc_end_editing: false,
        enter_end_editing: false,
    }
}

pub struct TextField<State> {
    pub(crate) id: u64,
    pub(crate) state: Binding<State, TextState>,
    pub(crate) fill: Color,
    pub(crate) font_size: u32,
    pub(crate) font_weight: FontWeight,
    pub(crate) font_family: Option<String>,
    pub(crate) alignment: Alignment,
    pub(crate) editable: bool,
    pub(crate) easing: Option<Easing>,
    pub(crate) duration: Option<f32>,
    pub(crate) delay: f32,
    pub(crate) line_height: f32,
    pub(crate) background_fill: Option<Color>,
    pub(crate) background_stroke: Option<(Color, Color, f32)>, // (normal, focused, width)
    pub(crate) background_corner_rounding: f32,
    pub(crate) background_padding: f32,
    pub(crate) wrap: bool,
    pub(crate) esc_end_editing: bool,
    pub(crate) enter_end_editing: bool,
    pub(crate) cursor_fill: Color,
    pub(crate) highlight_fill: Color,
    on_edit: Option<Rc<dyn Fn(&mut State, &mut AppState<State>, EditInteraction)>>,
}

impl<State> Debug for TextField<State> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Text")
            .field("id", &self.id)
            .field("state", &self.state)
            .field("fill", &self.fill)
            .field("font_size", &self.font_size)
            .field("font_weight", &self.font_weight)
            .field("alignment", &self.alignment)
            .field("editable", &self.editable)
            .field("easing", &self.easing)
            .field("duration", &self.duration)
            .field("delay", &self.delay)
            .field("line_height", &self.line_height)
            .field("background_fill", &self.background_fill)
            .field("background_stroke", &self.background_stroke)
            .field(
                "background_corner_rounding",
                &self.background_corner_rounding,
            )
            .field("background_padding", &self.background_padding)
            .field("wrap", &self.wrap)
            .field("cursor_fill", &self.cursor_fill)
            .field("highlight_fill", &self.highlight_fill)
            .field("on_edit", &self.on_edit.is_some())
            .finish()
    }
}

impl<State> Clone for TextField<State> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            state: self.state.clone(),
            fill: self.fill,
            font_size: self.font_size,
            font_weight: self.font_weight,
            font_family: self.font_family.clone(),
            alignment: self.alignment,
            editable: self.editable,
            easing: self.easing,
            duration: self.duration,
            delay: self.delay,
            line_height: self.line_height,
            background_fill: self.background_fill,
            background_stroke: self.background_stroke,
            background_corner_rounding: self.background_corner_rounding,
            background_padding: self.background_padding,
            wrap: self.wrap,
            cursor_fill: self.cursor_fill,
            highlight_fill: self.highlight_fill,
            on_edit: self.on_edit.clone(),
            esc_end_editing: false,
            enter_end_editing: false,
        }
    }
}

impl<State> TextField<State> {
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
    pub fn cursor_fill(mut self, color: Color) -> Self {
        self.cursor_fill = color;
        self
    }
    pub fn highlight_fill(mut self, color: Color) -> Self {
        self.highlight_fill = color;
        self
    }
    pub fn on_edit(
        mut self,
        on_edit: impl Fn(&mut State, &mut AppState<State>, EditInteraction) + 'static,
    ) -> Self {
        self.on_edit = Some(Rc::new(on_edit));
        self
    }
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
    pub fn esc_end_editing(mut self) -> Self {
        self.esc_end_editing = true;
        self
    }
    pub fn enter_end_editing(mut self) -> Self {
        self.enter_end_editing = true;
        self
    }
}

impl<State> TextField<State> {
    pub fn finish<'n>(self) -> Node<'n, State, AppState<State>>
    where
        State: 'static,
    {
        let id = self.id;
        let editable = self.editable;
        let bg_fill = self.background_fill;
        let bg_stroke = self.background_stroke;
        let bg_rounding = self.background_corner_rounding;
        let bg_padding = self.background_padding;
        let binding = self.state.clone();
        let font_size = self.font_size;
        let font_weight = self.font_weight;
        let font_family = self.font_family.clone();
        let fill = self.fill;
        let cursor_fill = self.cursor_fill;
        let highlight_fill = self.highlight_fill;
        let easing = self.easing;
        let duration = self.duration;
        let delay = self.delay;
        let alignment = self.alignment;
        let line_height = self.line_height;
        let wrap = self.wrap;
        let root_id = crate::id!(id);
        let text_id = crate::id!(id);
        dynamic(move |state, app: &mut AppState<State>| {
            if binding.get(state).editing
                && let Some(ref mut edit_state) = app.editor
            {
                let cursor_width = 2f64;
                let half_cursor_width = 1f64;
                let selection_rects = edit_state
                    .editor
                    .editor
                    .selection_geometry()
                    .iter()
                    .map(|(rect, _i)| *rect)
                    .collect::<Vec<_>>();
                let is_empty = edit_state.editor.text().to_string().is_empty();
                let cursor = edit_state
                    .editor
                    .editor
                    .cursor_geometry(cursor_width as f32);
                let layout = edit_state
                    .editor
                    .editor
                    .layout(&mut app.font_cx, &mut app.layout_cx)
                    .clone();
                let width = layout.width();
                let height = layout.height();

                let base = draw(move |area, _state, app: &mut AppState<State>| {
                    if wrap && let Some(ref mut edit_state) = app.editor {
                        edit_state.editor.editor.set_width(Some(area.width));
                    }
                    let transform = Affine::translate((area.x as f64, area.y as f64))
                        .then_scale(app.scale_factor);
                    for rect in selection_rects.clone() {
                        app.draw_list.entry(0).or_default().push(DrawItem {
                            view: View {
                                view_type: ViewType::Rect(Rect {
                                    id,
                                    shape: Shape {
                                        shape: ShapeType::Rect {
                                            corner_rounding: (2., 2., 2., 2.),
                                        },
                                        fill: Some(highlight_fill),
                                        stroke: None,
                                        easing: Some(Easing::EaseOut),
                                        duration: Some(0.),
                                        delay: 0.,
                                    },
                                }),
                                z_index: 0,
                                gesture_handlers: Vec::new(),
                            },
                            area: Area {
                                x: area.x + rect.x0 as f32,
                                y: area.y + rect.y0 as f32,
                                width: rect.width() as f32,
                                height: rect.height() as f32,
                            },
                            visible: true,
                            opacity: 1.,
                        });
                    }

                    if let Some(cursor) = if is_empty {
                        Some(vello_svg::vello::kurbo::Rect::new(
                            (area.x) as f64 - half_cursor_width,
                            (area.y) as f64,
                            (area.x) as f64 - half_cursor_width,
                            (area.y + area.height) as f64,
                        ))
                    } else {
                        cursor
                    } {
                        let rounding = (cursor_width * 0.5) as f32;
                        app.draw_list.entry(0).or_default().push(DrawItem {
                            view: View {
                                view_type: ViewType::Rect(Rect {
                                    id,
                                    shape: Shape {
                                        shape: ShapeType::Rect {
                                            corner_rounding: (
                                                rounding, rounding, rounding, rounding,
                                            ),
                                        },
                                        fill: Some(cursor_fill),
                                        stroke: None,
                                        easing: None,
                                        duration: Some(0.),
                                        delay: 0.,
                                    },
                                }),
                                z_index: 0,
                                gesture_handlers: Vec::new(),
                            },
                            area: Area {
                                x: area.x + cursor.x0 as f32,
                                y: area.y + cursor.y0 as f32,
                                width: cursor.width() as f32,
                                height: cursor.height() as f32,
                            },
                            visible: true,
                            opacity: 1.,
                        });
                    }
                    app.draw_list.entry(0).or_default().push(DrawItem {
                        view: View {
                            view_type: ViewType::Layout(layout.clone(), transform),
                            z_index: 0,
                            gesture_handlers: Vec::new(),
                        },
                        area: Area {
                            x: area.x,
                            y: area.y,
                            width: area.width,
                            height: area.height,
                        },
                        visible: true,
                        opacity: 1.,
                    });
                })
                .height(height);
                if wrap { base } else { base.width(width) }
            } else {
                Text {
                    id: text_id,
                    string: binding.get(state).text,
                    font_size,
                    font_weight,
                    font_family: font_family.clone(),
                    fill: if binding.get(state).editing {
                        TRANSPARENT
                    } else {
                        fill
                    },
                    easing,
                    duration,
                    delay,
                    alignment,
                    line_height,
                    wrap,
                }
                .view()
                .finish()
            }
            .attach_under(area_reader({
                let binding = binding.clone();
                let font_family = self.font_family.clone();
                let on_edit = self.on_edit.clone();
                move |area, _state, app: &mut AppState<State>| {
                    if let Some(EditState {
                        id,
                        area: edit_area,
                        ..
                    }) = &mut app.editor
                        && *id == root_id
                    {
                        *edit_area = area;
                    }
                    rect(root_id)
                        .fill(TRANSPARENT)
                        .view()
                        .on_key({
                            let on_edit = on_edit.clone();
                            let binding = binding.clone();
                            move |state, app, key| {
                                if (self.enter_end_editing
                                    && key == Key::Named(winit::keyboard::NamedKey::Enter))
                                    || (self.esc_end_editing
                                        && key == Key::Named(winit::keyboard::NamedKey::Escape))
                                {
                                    app.end_editing();
                                    binding.update(state, |s| s.editing = false);
                                    if let Some(ref on_edit) = on_edit {
                                        (on_edit)(state, app, EditInteraction::End);
                                    }
                                    return;
                                };
                                if let AppState {
                                    editor: Some(EditState { editor, id, .. }),
                                    layout_cx,
                                    font_cx,
                                    modifiers,
                                    ..
                                } = app
                                    && *id == root_id
                                {
                                    editor.handle_key(key.clone(), layout_cx, font_cx, *modifiers);
                                }
                                let edit_text =
                                    app.editor.as_ref().map(|e| e.editor.text().to_string());

                                if let Some(edit_text) = edit_text
                                    && app.editor.as_ref().map(|e| e.id) == Some(root_id)
                                {
                                    if let Some(ref on_edit) = on_edit {
                                        on_edit(
                                            state,
                                            app,
                                            EditInteraction::Update(edit_text.clone()),
                                        );
                                    }
                                    binding.update(state, |s| {
                                        s.text = edit_text.clone();
                                    });
                                }
                            }
                        })
                        .on_click_outside({
                            let binding = binding.clone();
                            let on_edit = on_edit.clone();
                            move |state: &mut State, app, _, _| {
                                if let AppState {
                                    editor: Some(EditState { id, .. }),
                                    ..
                                } = app
                                    && *id == root_id
                                {
                                    app.end_editing();
                                    binding.update(state, |s| s.editing = false);
                                    if let Some(ref on_edit) = on_edit {
                                        (on_edit)(state, app, EditInteraction::End);
                                    }
                                }
                            }
                        })
                        .on_click({
                            let binding = binding.clone();
                            let font_family = font_family.clone();
                            move |state: &mut State, app, _, _| {
                                let editing = binding.get(state).editing;
                                if !editing && app.editor.is_none() {
                                    app.animation_bank.animations.remove(&text_id);
                                    binding.update(state, |s| s.editing = true);
                                    let editing = binding.get(state).editing;
                                    if editing && app.editor.is_none() {
                                        let mut editor = PlainEditor::new(self.font_size as f32);
                                        editor.set_text(&binding.get(state).text);
                                        let styles = editor.edit_styles();

                                        styles
                                            .insert(StyleProperty::Brush(Brush::Solid(self.fill)));
                                        styles.insert(
                                            parley::FontFamily::Named(
                                                font_family
                                                    .clone()
                                                    .unwrap_or(DEFAULT_FONT_FAMILY.to_string())
                                                    .into(),
                                            )
                                            .into(),
                                        );
                                        styles.insert(StyleProperty::FontWeight(self.font_weight));
                                        styles.insert(StyleProperty::LineHeight(
                                            LineHeight::FontSizeRelative(self.line_height),
                                        ));
                                        styles
                                            .insert(StyleProperty::FontSize(self.font_size as f32));
                                        styles.insert(StyleProperty::OverflowWrap(
                                            parley::OverflowWrap::Anywhere,
                                        ));

                                        editor.set_alignment(alignment);
                                        if wrap {
                                            editor.set_width(Some(area.width));
                                        }
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

                                        if let AppState {
                                            cursor_position: Some(pos),
                                            font_cx,
                                            layout_cx,
                                            ..
                                        } = app
                                        {
                                            editor.mouse_moved(
                                                Point::new(
                                                    pos.x - area.x as f64,
                                                    pos.y - area.y as f64,
                                                ),
                                                layout_cx,
                                                font_cx,
                                            );
                                        }
                                        app.editor = Some(EditState {
                                            id: root_id,
                                            area,
                                            editor,
                                            editing: true,
                                            binding: binding.clone(),
                                            cursor_color: self.cursor_fill,
                                            highlight_color: self.highlight_fill,
                                        });
                                    }
                                }
                            }
                        })
                        .finish()
                }
            }))
            .pad(if editable { bg_padding } else { 0. })
            .attach_under(if bg_fill.is_some() || bg_stroke.is_some() {
                dynamic({
                    let binding = binding.clone();
                    move |s, _a| {
                        let mut rect_node = rect(crate::id!(id));
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
            })
        })
    }
}
