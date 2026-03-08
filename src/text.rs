use crate::app::{AppCtx, AppState, LayoutCache, View};
use crate::background_style::BrushSource;
use crate::draw_layout::draw_layout;
use crate::view::{Drawable, DrawableType};
use crate::{DEFAULT_FG_COLOR, DEFAULT_FONT_FAMILY, DEFAULT_FONT_SIZE};
use vello_svg::vello::Scene;
use backer::{Area, Layout};
use parley::{
    Alignment, AlignmentOptions, FontContext, FontStack, FontWeight, Layout as ParleyLayout,
    LayoutContext, LineHeight, TextStyle,
};
use std::fmt::Debug;
use vello_svg::vello::kurbo::Affine;
use vello_svg::vello::peniko::Brush;

pub fn text(id: u64, text: impl AsRef<str>) -> Text {
    Text {
        id,
        string: text.as_ref().to_string(),
        font_size: DEFAULT_FONT_SIZE,
        font_weight: FontWeight::NORMAL,
        font_family: Some(DEFAULT_FONT_FAMILY.to_string()),
        fill: BrushSource::Static(Brush::Solid(DEFAULT_FG_COLOR)),
        alignment: Alignment::Center,
        line_height: 1.,
        wrap: false,
    }
}

pub struct Text {
    pub(crate) id: u64,
    pub(crate) string: String,
    pub(crate) fill: BrushSource<()>,
    pub(crate) font_size: u32,
    pub(crate) font_weight: FontWeight,
    pub(crate) font_family: Option<String>,
    pub(crate) alignment: Alignment,
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
            fill: self.fill.clone(),
            font_size: self.font_size,
            font_weight: self.font_weight,
            font_family: self.font_family.clone(),
            alignment: self.alignment,
            line_height: self.line_height,
            wrap: self.wrap,
        }
    }
}

impl Text {
    pub fn fill(mut self, fill: impl Into<BrushSource<()>>) -> Self {
        self.fill = fill.into();
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
    pub fn view<State>(self) -> Drawable<State> {
        Drawable {
            view_type: DrawableType::Text(self),
            gesture_handlers: Vec::new(),
        }
    }
    pub fn build<State: 'static>(self, ctx: &mut AppCtx) -> Layout<'static, View<State>, AppCtx> {
        self.view().finish(ctx)
    }
}

pub struct TextLayout {
    pub(crate) layout_cache: LayoutCache,
    pub(crate) font_cx: FontContext,
    pub(crate) layout_cx: LayoutContext<Brush>,
}

impl TextLayout {
    pub(crate) fn new(
        layout_cache: LayoutCache,
        font_cx: FontContext,
        layout_cx: LayoutContext<Brush>,
    ) -> Self {
        Self {
            layout_cache,
            font_cx,
            layout_cx,
        }
    }

    pub(crate) fn build_layout(
        &mut self,
        text: &Text,
        current_fill: &Brush,
        available_width: f32,
        cache: bool,
    ) -> ParleyLayout<Brush> {
        let text_str = text.string.clone();
        let current_text = if text_str.is_empty() {
            " ".to_string()
        } else {
            text_str
        };

        if !current_text.is_empty()
            && let Some((_, _, layout)) = self.layout_cache.get(&text.id).and_then(|cached| {
                cached
                    .iter()
                    .find(|(t, width, _)| *t == current_text && *width == available_width)
            })
        {
            return layout.clone();
        }

        {
            let mut builder = self.layout_cx.tree_builder(
                &mut self.font_cx,
                1.,
                true,
                &TextStyle {
                    brush: current_fill.clone(),
                    font_stack: FontStack::Single(parley::FontFamily::Named(
                        text.font_family
                            .clone()
                            .unwrap_or(DEFAULT_FONT_FAMILY.to_string())
                            .into(),
                    )),
                    font_weight: text.font_weight,
                    line_height: LineHeight::FontSizeRelative(text.line_height),
                    font_size: text.font_size as f32,
                    overflow_wrap: parley::OverflowWrap::Anywhere,
                    ..Default::default()
                },
            );
            builder.push_text(&current_text);
            let mut layout = builder.build().0;
            layout.break_all_lines(Some(available_width));
            layout.align(
                Some(available_width),
                text.alignment,
                AlignmentOptions {
                    align_when_overflowing: true,
                },
            );
            if cache {
                let entry = self.layout_cache.entry(text.id).or_insert(vec![(
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
}

impl Text {
    pub(crate) fn draw(&mut self, animated_area: Area, area: Area, scene: &mut Scene, app: &mut AppState) {
        let fill = self.fill.resolve(area, &());

        let layout = app
            .app_context
            .text_layout
            .build_layout(self, &fill, area.width, true);

        let transform = Affine::translate((animated_area.x as f64, animated_area.y as f64))
            .then_scale(app.app_context.scale_factor);

        draw_layout(Some(fill), transform, &layout, scene);
    }
}

impl Text {
    pub(crate) fn with_text_constraints<State>(
        self,
        ctx: &mut AppCtx,
        node: Layout<'static, View<State>, AppCtx>,
    ) -> Layout<'static, View<State>, AppCtx> {
        if self.wrap {
            node.dynamic_height(move |w, ctx| {
                let default_brush = Brush::Solid(crate::DEFAULT_FG_COLOR);
                ctx.text_layout
                    .build_layout(&self, &default_brush, w, true)
                    .height()
            })
        } else {
            let default_brush = Brush::Solid(crate::DEFAULT_FG_COLOR);
            let layout = ctx
                .text_layout
                .build_layout(&self, &default_brush, 10000., true);
            node.height(layout.height()).width(layout.width().max(10.))
        }
    }
}
