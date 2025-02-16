use backer::models::Area;
use core::default::Default;
use parley::{editor::SplitString, layout::PositionedLayoutItem, GenericFamily, StyleProperty};
use std::time::{Duration, Instant};
use vello_svg::vello::{
    kurbo::{Affine, Line, Point, Stroke},
    peniko::{color::palette, Brush, Fill},
    Scene,
};
use winit::{event::Modifiers, keyboard::NamedKey};

pub use parley::layout::editor::Generation;
use parley::{FontContext, LayoutContext, PlainEditor, PlainEditorDriver};

use crate::Key;

#[derive(Clone)]
pub struct Editor {
    pub(crate) editor: PlainEditor<Brush>,
    pub(crate) last_click_time: Option<Instant>,
    pub(crate) click_count: u32,
    pub(crate) pointer_down: bool,
    pub(crate) cursor_pos: (f32, f32),
    pub(crate) cursor_visible: bool,
    pub(crate) modifiers: Option<Modifiers>,
    pub(crate) start_time: Option<Instant>,
    pub(crate) blink_period: Duration,
}

impl Editor {
    pub fn new(text: &str) -> Self {
        let mut editor = PlainEditor::new(32.0);
        editor.set_text(text);
        editor.set_scale(1.0);
        let styles = editor.edit_styles();
        styles.insert(StyleProperty::LineHeight(1.2));
        styles.insert(GenericFamily::SystemUi.into());
        styles.insert(StyleProperty::Brush(palette::css::WHITE.into()));
        Self {
            editor,
            last_click_time: Default::default(),
            click_count: Default::default(),
            pointer_down: Default::default(),
            cursor_pos: Default::default(),
            cursor_visible: Default::default(),
            modifiers: Default::default(),
            start_time: Default::default(),
            blink_period: Default::default(),
        }
    }

    fn driver<'a>(
        &'a mut self,
        font_context: &'a mut FontContext,
        layout_context: &'a mut LayoutContext<Brush>,
    ) -> PlainEditorDriver<'a, Brush> {
        self.editor.driver(font_context, layout_context)
    }

    pub fn editor(&mut self) -> &mut PlainEditor<Brush> {
        &mut self.editor
    }

    pub fn text(&self) -> SplitString<'_> {
        self.editor.text()
    }

    pub fn cursor_reset(&mut self) {
        self.start_time = Some(Instant::now());
        // TODO: for real world use, this should be reading from the system settings
        self.blink_period = Duration::from_millis(500);
        self.cursor_visible = true;
    }

    pub fn disable_blink(&mut self) {
        self.start_time = None;
    }

    pub fn next_blink_time(&self) -> Option<Instant> {
        self.start_time.map(|start_time| {
            let phase = Instant::now().duration_since(start_time);

            start_time
                + Duration::from_nanos(
                    ((phase.as_nanos() / self.blink_period.as_nanos() + 1)
                        * self.blink_period.as_nanos()) as u64,
                )
        })
    }

    pub fn cursor_blink(&mut self) {
        self.cursor_visible = self.start_time.is_some_and(|start_time| {
            let elapsed = Instant::now().duration_since(start_time);
            (elapsed.as_millis() / self.blink_period.as_millis()) % 2 == 0
        });
    }

    /// Return the current `Generation` of the layout.
    pub fn generation(&self) -> Generation {
        self.editor.generation()
    }

    pub fn handle_key(
        &mut self,
        key: Key,
        layout_cx: &mut LayoutContext<Brush>,
        font_cx: &mut FontContext,
        modifiers: Option<Modifiers>,
    ) {
        self.modifiers = modifiers;
        #[allow(unused)]
        let (shift, action_mod) = self
            .modifiers
            .map(|mods| {
                (
                    mods.state().shift_key(),
                    if cfg!(target_os = "macos") {
                        mods.state().super_key()
                    } else {
                        mods.state().control_key()
                    },
                )
            })
            .unwrap_or_default();
        let mut drv = self.driver(font_cx, layout_cx);
        match key {
            #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
            Key::Character(c) if action_mod && matches!(c.as_str(), "c" | "x" | "v") => {
                use clipboard_rs::{Clipboard, ClipboardContext};
                match c.to_lowercase().as_str() {
                    "c" => {
                        if let Some(text) = drv.editor.selected_text() {
                            let cb = ClipboardContext::new().unwrap();
                            cb.set_text(text.to_owned()).ok();
                        }
                    }
                    "x" => {
                        if let Some(text) = drv.editor.selected_text() {
                            let cb = ClipboardContext::new().unwrap();
                            cb.set_text(text.to_owned()).ok();
                            drv.delete_selection();
                        }
                    }
                    "v" => {
                        let cb = ClipboardContext::new().unwrap();
                        let text = cb.get_text().unwrap_or_default();
                        drv.insert_or_replace_selection(&text);
                    }
                    _ => (),
                }
            }
            Key::Character(c) if action_mod && matches!(c.to_lowercase().as_str(), "a") => {
                if shift {
                    drv.collapse_selection();
                } else {
                    drv.select_all();
                }
            }
            Key::Named(NamedKey::ArrowLeft) => {
                if action_mod {
                    if shift {
                        drv.select_word_left();
                    } else {
                        drv.move_word_left();
                    }
                } else if shift {
                    drv.select_left();
                } else {
                    drv.move_left();
                }
            }
            Key::Named(NamedKey::ArrowRight) => {
                if action_mod {
                    if shift {
                        drv.select_word_right();
                    } else {
                        drv.move_word_right();
                    }
                } else if shift {
                    drv.select_right();
                } else {
                    drv.move_right();
                }
            }
            Key::Named(NamedKey::ArrowUp) => {
                if shift {
                    drv.select_up();
                } else {
                    drv.move_up();
                }
            }
            Key::Named(NamedKey::ArrowDown) => {
                if shift {
                    drv.select_down();
                } else {
                    drv.move_down();
                }
            }
            Key::Named(NamedKey::Home) => {
                if action_mod {
                    if shift {
                        drv.select_to_text_start();
                    } else {
                        drv.move_to_text_start();
                    }
                } else if shift {
                    drv.select_to_line_start();
                } else {
                    drv.move_to_line_start();
                }
            }
            Key::Named(NamedKey::End) => {
                if action_mod {
                    if shift {
                        drv.select_to_text_end();
                    } else {
                        drv.move_to_text_end();
                    }
                } else if shift {
                    drv.select_to_line_end();
                } else {
                    drv.move_to_line_end();
                }
            }
            Key::Named(NamedKey::Delete) => {
                if action_mod {
                    drv.delete_word();
                } else {
                    drv.delete();
                }
            }
            Key::Named(NamedKey::Backspace) => {
                if action_mod {
                    drv.backdelete_word();
                } else {
                    drv.backdelete();
                }
            }
            Key::Named(NamedKey::Enter) => {
                drv.insert_or_replace_selection("\n");
            }
            Key::Named(NamedKey::Space) => {
                drv.insert_or_replace_selection(" ");
            }
            Key::Character(s) => {
                drv.insert_or_replace_selection(&s);
            }
            _ => (),
        }
    }
    pub(crate) fn mouse_pressed(
        &mut self,
        layout_cx: &mut LayoutContext<Brush>,
        font_cx: &mut FontContext,
    ) {
        self.cursor_reset();
        if !self.editor.is_composing() {
            self.pointer_down = true;
            let now = Instant::now();
            if let Some(last) = self.last_click_time.take() {
                if now.duration_since(last).as_secs_f64() < 0.25 {
                    self.click_count = (self.click_count + 1) % 4;
                } else {
                    self.click_count = 1;
                }
            } else {
                self.click_count = 1;
            }
            self.last_click_time = Some(now);
            let click_count = self.click_count;
            let cursor_pos = self.cursor_pos;
            let mut drv = self.editor.driver(font_cx, layout_cx);
            match click_count {
                2 => drv.select_word_at_point(cursor_pos.0, cursor_pos.1),
                3 => drv.select_line_at_point(cursor_pos.0, cursor_pos.1),
                _ => drv.move_to_point(cursor_pos.0, cursor_pos.1),
            }
        }
    }
    pub(crate) fn mouse_released(&mut self) {
        self.pointer_down = false;
    }
    pub(crate) fn mouse_moved(
        &mut self,
        position: Point,
        layout_cx: &mut LayoutContext<Brush>,
        font_cx: &mut FontContext,
    ) {
        let prev_pos = self.cursor_pos;
        self.cursor_pos = (position.x as f32, position.y as f32);
        // macOS seems to generate a spurious move after selecting word?
        if self.pointer_down && prev_pos != self.cursor_pos && !self.editor.is_composing() {
            self.cursor_reset();
            let cursor_pos = self.cursor_pos;
            self.driver(font_cx, layout_cx)
                .extend_selection_to_point(cursor_pos.0, cursor_pos.1);
        }
    }
    pub(crate) fn draw(
        &mut self,
        area: Area,
        scene: &mut Scene,
        layout_cx: &mut LayoutContext<Brush>,
        font_cx: &mut FontContext,
        _visible: bool,
        _visible_amount: f32,
    ) -> Generation {
        let transform = Affine::translate((area.x as f64, area.y as f64));

        for rect in self.editor.selection_geometry().iter() {
            scene.fill(
                Fill::NonZero,
                transform,
                palette::css::STEEL_BLUE,
                None,
                &rect,
            );
        }
        if self.cursor_visible {
            if let Some(cursor) = self.editor.cursor_geometry(1.5) {
                scene.fill(Fill::NonZero, transform, palette::css::WHITE, None, &cursor);
            }
        }

        let editor = &mut self.editor;
        editor.set_width(Some(area.width));
        let layout = editor.layout(font_cx, layout_cx);

        for line in layout.lines() {
            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                    continue;
                };
                let style = glyph_run.style();
                // We draw underlines under the text, then the strikethrough on top, following:
                // https://drafts.csswg.org/css-text-decor/#painting-order
                if let Some(underline) = &style.underline {
                    let underline_brush = &style.brush;
                    let run_metrics = glyph_run.run().metrics();
                    let offset = match underline.offset {
                        Some(offset) => offset,
                        None => run_metrics.underline_offset,
                    };
                    let width = match underline.size {
                        Some(size) => size,
                        None => run_metrics.underline_size,
                    };
                    // The `offset` is the distance from the baseline to the top of the underline
                    // so we move the line down by half the width
                    // Remember that we are using a y-down coordinate system
                    // If there's a custom width, because this is an underline, we want the custom
                    // width to go down from the default expectation
                    let y = glyph_run.baseline() - offset + width / 2.;

                    let line = Line::new(
                        (glyph_run.offset() as f64, y as f64),
                        ((glyph_run.offset() + glyph_run.advance()) as f64, y as f64),
                    );
                    scene.stroke(
                        &Stroke::new(width.into()),
                        transform,
                        underline_brush,
                        None,
                        &line,
                    );
                }
                let mut x = glyph_run.offset();
                let y = glyph_run.baseline();
                let run = glyph_run.run();
                let font = run.font();
                let font_size = run.font_size();
                let synthesis = run.synthesis();
                let glyph_xform = synthesis
                    .skew()
                    .map(|angle| Affine::skew(angle.to_radians().tan() as f64, 0.0));

                scene
                    .draw_glyphs(font)
                    .brush(&style.brush)
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
                if let Some(strikethrough) = &style.strikethrough {
                    let strikethrough_brush = &style.brush;
                    let run_metrics = glyph_run.run().metrics();
                    let offset = match strikethrough.offset {
                        Some(offset) => offset,
                        None => run_metrics.strikethrough_offset,
                    };
                    let width = match strikethrough.size {
                        Some(size) => size,
                        None => run_metrics.strikethrough_size,
                    };
                    // The `offset` is the distance from the baseline to the *top* of the strikethrough
                    // so we calculate the middle y-position of the strikethrough based on the font's
                    // standard strikethrough width.
                    // Remember that we are using a y-down coordinate system
                    let y = glyph_run.baseline() - offset + run_metrics.strikethrough_size / 2.;

                    let line = Line::new(
                        (glyph_run.offset() as f64, y as f64),
                        ((glyph_run.offset() + glyph_run.advance()) as f64, y as f64),
                    );
                    scene.stroke(
                        &Stroke::new(width.into()),
                        transform,
                        strikethrough_brush,
                        None,
                        &line,
                    );
                }
            }
        }
        self.editor.generation()
    }
}

// #[derive(Debug)]
// pub struct TextField<State> {
//     pub(crate) id: u64,
//     state: Binding<State, Editor>,
//     pub(crate) easing: Option<Easing>,
//     pub(crate) duration: Option<f32>,
//     pub(crate) delay: f32,
// }

// impl<State> Clone for TextField<State> {
//     fn clone(&self) -> Self {
//         TextField {
//             id: self.id,
//             state: self.state.clone(),
//             easing: self.easing.clone(),
//             duration: self.duration.clone(),
//             delay: self.delay,
//         }
//     }
// }

// impl<State> TextField<State> {
//     pub fn view(self) -> View<State, ()> {
//         View {
//             view_type: ViewType::TextField(self),
//             gesture_handler: GestureHandler::default(),
//         }
//     }
//     pub fn finish<'n>(self) -> Node<'n, RcUi<State>>
//     where
//         State: 'static,
//     {
//         let binding = self.state.clone();
//         self.view()
//             // .on_click(|s, click_state, location| {
//             //     let mut editor = binding.clone().get(s);
//             //     editor.pointer_down = click_state == ClickState::Started;
//             //     editor.cursor_reset();
//             //     if editor.pointer_down && !editor.editor.is_composing() {
//             //         let now = Instant::now();
//             //         if let Some(last) = editor.last_click_time.take() {
//             //             if now.duration_since(last).as_secs_f64() < 0.25 {
//             //                 editor.click_count = (editor.click_count + 1) % 4;
//             //             } else {
//             //                 editor.click_count = 1;
//             //             }
//             //         } else {
//             //             editor.click_count = 1;
//             //         }
//             //         editor.last_click_time = Some(now);
//             //         let click_count = editor.click_count;
//             //         let cursor_pos = editor.cursor_pos;
//             //         let mut drv = editor.editor.driver(&mut self.font_cx, &mut self.layout_cx);
//             //         match click_count {
//             //             2 => drv.select_word_at_point(cursor_pos.0, cursor_pos.1),
//             //             3 => drv.select_line_at_point(cursor_pos.0, cursor_pos.1),
//             //             _ => drv.move_to_point(cursor_pos.0, cursor_pos.1),
//             //         }
//             //     }
//             // })
//             // .on_system_key({
//             //     move |s, cx, key| {
//             //         dbg!(&key);
//             //         let mut layout_cx = cx.layout_cx.take().unwrap();
//             //         let mut font_cx = cx.font_cx.take().unwrap();
//             //         let mut editor = binding.clone().get(s);
//             //         editor.modifiers = cx.modifiers;
//             //         let mut drv = editor.driver(&mut font_cx, &mut layout_cx);
//             //         #[allow(unused)]
//             //         let (shift, action_mod) = binding
//             //             .get(s)
//             //             .modifiers
//             //             .map(|mods| {
//             //                 (
//             //                     mods.state().shift_key(),
//             //                     if cfg!(target_os = "macos") {
//             //                         mods.state().super_key()
//             //                     } else {
//             //                         mods.state().control_key()
//             //                     },
//             //                 )
//             //             })
//             //             .unwrap_or_default();
//             //         match key {
//             //             #[cfg(any(
//             //                 target_os = "windows",
//             //                 target_os = "macos",
//             //                 target_os = "linux"
//             //             ))]
//             //             Key::Character(c)
//             //                 if action_mod && matches!(c.as_str(), "c" | "x" | "v") =>
//             //             {
//             //                 use clipboard_rs::{Clipboard, ClipboardContext};
//             //                 match c.to_lowercase().as_str() {
//             //                     "c" => {
//             //                         if let Some(text) = drv.editor.selected_text() {
//             //                             let cb = ClipboardContext::new().unwrap();
//             //                             cb.set_text(text.to_owned()).ok();
//             //                         }
//             //                     }
//             //                     "x" => {
//             //                         if let Some(text) = drv.editor.selected_text() {
//             //                             let cb = ClipboardContext::new().unwrap();
//             //                             cb.set_text(text.to_owned()).ok();
//             //                             drv.delete_selection();
//             //                         }
//             //                     }
//             //                     "v" => {
//             //                         let cb = ClipboardContext::new().unwrap();
//             //                         let text = cb.get_text().unwrap_or_default();
//             //                         drv.insert_or_replace_selection(&text);
//             //                     }
//             //                     _ => (),
//             //                 }
//             //             }
//             //             Key::Character(c)
//             //                 if action_mod && matches!(c.to_lowercase().as_str(), "a") =>
//             //             {
//             //                 if shift {
//             //                     drv.collapse_selection();
//             //                 } else {
//             //                     drv.select_all();
//             //                 }
//             //             }
//             //             Key::Named(NamedKey::ArrowLeft) => {
//             //                 if action_mod {
//             //                     if shift {
//             //                         drv.select_word_left();
//             //                     } else {
//             //                         drv.move_word_left();
//             //                     }
//             //                 } else if shift {
//             //                     drv.select_left();
//             //                 } else {
//             //                     drv.move_left();
//             //                 }
//             //             }
//             //             Key::Named(NamedKey::ArrowRight) => {
//             //                 if action_mod {
//             //                     if shift {
//             //                         drv.select_word_right();
//             //                     } else {
//             //                         drv.move_word_right();
//             //                     }
//             //                 } else if shift {
//             //                     drv.select_right();
//             //                 } else {
//             //                     drv.move_right();
//             //                 }
//             //             }
//             //             Key::Named(NamedKey::ArrowUp) => {
//             //                 if shift {
//             //                     drv.select_up();
//             //                 } else {
//             //                     drv.move_up();
//             //                 }
//             //             }
//             //             Key::Named(NamedKey::ArrowDown) => {
//             //                 if shift {
//             //                     drv.select_down();
//             //                 } else {
//             //                     drv.move_down();
//             //                 }
//             //             }
//             //             Key::Named(NamedKey::Home) => {
//             //                 if action_mod {
//             //                     if shift {
//             //                         drv.select_to_text_start();
//             //                     } else {
//             //                         drv.move_to_text_start();
//             //                     }
//             //                 } else if shift {
//             //                     drv.select_to_line_start();
//             //                 } else {
//             //                     drv.move_to_line_start();
//             //                 }
//             //             }
//             //             Key::Named(NamedKey::End) => {
//             //                 if action_mod {
//             //                     if shift {
//             //                         drv.select_to_text_end();
//             //                     } else {
//             //                         drv.move_to_text_end();
//             //                     }
//             //                 } else if shift {
//             //                     drv.select_to_line_end();
//             //                 } else {
//             //                     drv.move_to_line_end();
//             //                 }
//             //             }
//             //             Key::Named(NamedKey::Delete) => {
//             //                 if action_mod {
//             //                     drv.delete_word();
//             //                 } else {
//             //                     drv.delete();
//             //                 }
//             //             }
//             //             Key::Named(NamedKey::Backspace) => {
//             //                 if action_mod {
//             //                     drv.backdelete_word();
//             //                 } else {
//             //                     drv.backdelete();
//             //                 }
//             //             }
//             //             Key::Named(NamedKey::Enter) => {
//             //                 drv.insert_or_replace_selection("\n");
//             //             }
//             //             Key::Named(NamedKey::Space) => {
//             //                 drv.insert_or_replace_selection(" ");
//             //             }
//             //             Key::Character(s) => {
//             //                 drv.insert_or_replace_selection(&s);
//             //             }
//             //             _ => (),
//             //         }
//             //         cx.layout_cx.set(Some(layout_cx));
//             //         cx.font_cx.set(Some(font_cx));
//             //         binding.clone().set(s, editor);
//             //     }
//             // })
//             .finish()
//     }
//     pub(crate) fn draw(
//         &mut self,
//         area: Area,
//         state: &mut RcUi<State>,
//         _visible: bool,
//         _visible_amount: f32,
//     ) {
//         let transform = Affine::translate((area.x as f64, area.y as f64));

//         for rect in self
//             .state
//             .get(&state.ui.state)
//             .editor
//             .selection_geometry()
//             .iter()
//         {
//             state.ui.cx().scene.fill(
//                 Fill::NonZero,
//                 transform,
//                 palette::css::STEEL_BLUE,
//                 None,
//                 &rect,
//             );
//         }
//         if self.state.get(&state.ui.state).cursor_visible {
//             if let Some(cursor) = self.state.get(&state.ui.state).editor.cursor_geometry(1.5) {
//                 state.ui.cx().scene.fill(
//                     Fill::NonZero,
//                     transform,
//                     palette::css::WHITE,
//                     None,
//                     &cursor,
//                 );
//             }
//         }

//         let mut layout_cx = state.ui.cx().layout_cx.take().unwrap();
//         let mut font_cx = state.ui.cx().font_cx.take().unwrap();
//         let mut editor = self.state.get(&state.ui.state).editor;
//         editor.set_width(Some(area.width));
//         let layout = editor.layout(&mut font_cx, &mut layout_cx);
//         state.ui.cx().layout_cx.set(Some(layout_cx));
//         state.ui.cx().font_cx.set(Some(font_cx));

//         for line in layout.lines() {
//             for item in line.items() {
//                 let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
//                     continue;
//                 };
//                 let style = glyph_run.style();
//                 // We draw underlines under the text, then the strikethrough on top, following:
//                 // https://drafts.csswg.org/css-text-decor/#painting-order
//                 if let Some(underline) = &style.underline {
//                     let underline_brush = &style.brush;
//                     let run_metrics = glyph_run.run().metrics();
//                     let offset = match underline.offset {
//                         Some(offset) => offset,
//                         None => run_metrics.underline_offset,
//                     };
//                     let width = match underline.size {
//                         Some(size) => size,
//                         None => run_metrics.underline_size,
//                     };
//                     // The `offset` is the distance from the baseline to the top of the underline
//                     // so we move the line down by half the width
//                     // Remember that we are using a y-down coordinate system
//                     // If there's a custom width, because this is an underline, we want the custom
//                     // width to go down from the default expectation
//                     let y = glyph_run.baseline() - offset + width / 2.;

//                     let line = Line::new(
//                         (glyph_run.offset() as f64, y as f64),
//                         ((glyph_run.offset() + glyph_run.advance()) as f64, y as f64),
//                     );
//                     state.ui.cx().scene.stroke(
//                         &Stroke::new(width.into()),
//                         transform,
//                         underline_brush,
//                         None,
//                         &line,
//                     );
//                 }
//                 let mut x = glyph_run.offset();
//                 let y = glyph_run.baseline();
//                 let run = glyph_run.run();
//                 let font = run.font();
//                 let font_size = run.font_size();
//                 let synthesis = run.synthesis();
//                 let glyph_xform = synthesis
//                     .skew()
//                     .map(|angle| Affine::skew(angle.to_radians().tan() as f64, 0.0));
//                 state
//                     .ui
//                     .cx()
//                     .scene
//                     .draw_glyphs(font)
//                     .brush(&style.brush)
//                     .hint(true)
//                     .transform(transform)
//                     .glyph_transform(glyph_xform)
//                     .font_size(font_size)
//                     .normalized_coords(run.normalized_coords())
//                     .draw(
//                         Fill::NonZero,
//                         glyph_run.glyphs().map(|glyph| {
//                             let gx = x + glyph.x;
//                             let gy = y - glyph.y;
//                             x += glyph.advance;
//                             vello_svg::vello::Glyph {
//                                 id: glyph.id as _,
//                                 x: gx,
//                                 y: gy,
//                             }
//                         }),
//                     );
//                 if let Some(strikethrough) = &style.strikethrough {
//                     let strikethrough_brush = &style.brush;
//                     let run_metrics = glyph_run.run().metrics();
//                     let offset = match strikethrough.offset {
//                         Some(offset) => offset,
//                         None => run_metrics.strikethrough_offset,
//                     };
//                     let width = match strikethrough.size {
//                         Some(size) => size,
//                         None => run_metrics.strikethrough_size,
//                     };
//                     // The `offset` is the distance from the baseline to the *top* of the strikethrough
//                     // so we calculate the middle y-position of the strikethrough based on the font's
//                     // standard strikethrough width.
//                     // Remember that we are using a y-down coordinate system
//                     let y = glyph_run.baseline() - offset + run_metrics.strikethrough_size / 2.;

//                     let line = Line::new(
//                         (glyph_run.offset() as f64, y as f64),
//                         ((glyph_run.offset() + glyph_run.advance()) as f64, y as f64),
//                     );
//                     state.ui.cx().scene.stroke(
//                         &Stroke::new(width.into()),
//                         transform,
//                         strikethrough_brush,
//                         None,
//                         &line,
//                     );
//                 }
//             }
//         }
//         self.state.get(&state.ui.state).editor.generation();
//     }
// }

// pub fn text_field<State>(id: u64, binding: Binding<State, Editor>) -> TextField<State> {
//     TextField {
//         id,
//         state: binding,
//         easing: None,
//         duration: None,
//         delay: 0.,
//     }
// }
