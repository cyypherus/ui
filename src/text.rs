use crate::{
    view::{View, ViewTrait, ViewType},
    GestureHandler, Ui,
};
use backer::{models::*, transitions::TransitionDrawable, Node};
use parley::{FontStack, PositionedLayoutItem, TextStyle};
use std::hash::{DefaultHasher, Hash, Hasher};
use vello::{
    kurbo::Affine,
    peniko::{Color, Fill},
};

pub fn text(id: String, text: impl AsRef<str> + 'static) -> Text {
    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    Text {
        id: hasher.finish(),
        text: text.as_ref().to_owned(),
        font_size: 40,
        // font: None,
        fill: None,
        easing: None,
        duration: None,
        delay: 0.,
    }
}

#[derive(Debug, Clone)]
pub struct Text {
    id: u64,
    text: String,
    fill: Option<Color>,
    font_size: u32,
    // font: Option<font::Id>,
    pub(crate) easing: Option<backer::Easing>,
    pub(crate) duration: Option<f32>,
    pub(crate) delay: f32,
}

impl Text {
    pub fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
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

impl<State> TransitionDrawable<Ui<State>> for Text {
    fn draw_interpolated(
        &mut self,
        area: Area,
        state: &mut Ui<State>,
        visible: bool,
        visible_amount: f32,
    ) {
        if !visible && visible_amount == 0. {
            return;
        }

        let mut layout = state.cx().with_font_layout_ctx(|layout_cx, font_cx| {
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
                state
                    .cx()
                    .scene
                    .draw_glyphs(font)
                    .brush(
                        self.fill
                            .unwrap_or(Color::BLACK)
                            .multiply_alpha(visible_amount),
                    )
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
    }

    fn id(&self) -> &u64 {
        &self.id
    }
    fn easing(&self) -> backer::Easing {
        backer::Easing::EaseOut
    }
    fn duration(&self) -> f32 {
        0.
    }
    fn delay(&self) -> f32 {
        0.
    }
    fn constraints(
        &self,
        _available_area: Area,
        _state: &mut Ui<State>,
    ) -> Option<backer::SizeConstraints> {
        None
    }
}

impl<'s, State> ViewTrait<'s, State> for Text {
    fn view(self, ui: &mut Ui<State>, node: Node<Ui<State>>) -> Node<Ui<State>> {
        let mut layout = ui.cx().with_font_layout_ctx(|layout_cx, font_cx| {
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
