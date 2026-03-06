use crate::DEFAULT_FG;
use crate::background_style::BrushSource;
use crate::{
    Binding, ClickState, DEFAULT_CORNER_ROUNDING, DEFAULT_FONT_SIZE, DEFAULT_PADDING, DEFAULT_PURP,
    adjust_brush,
    app::{AppCtx, AppState, View},
    rect,
};
use backer::{Layout, nodes::{multi_draw, stack}};
use std::rc::Rc;
use vello_svg::vello::kurbo::Stroke;
use vello_svg::vello::peniko::Brush;
use vello_svg::vello::peniko::color::palette::css::TRANSPARENT;

#[derive(Debug, Clone, Copy, Default)]
pub struct ButtonState {
    pub hovered: bool,
    pub depressed: bool,
}

pub struct Button<State> {
    id: u64,
    body: Option<Layout<'static, View<State>, AppCtx>>,
    label: Option<Layout<'static, View<State>, AppCtx>>,
    text_label: Option<String>,
    background_corner_rounding: Option<f32>,
    background_padding: f32,
    on_click: Option<Rc<dyn Fn(&mut State, &mut AppState)>>,
    state: ButtonState,
    binding: Binding<State, ButtonState>,
    background_fill: Option<BrushSource<ButtonState>>,
    background_stroke: Option<(BrushSource<ButtonState>, Stroke)>,
    text_fill: Option<Brush>,
}

pub fn button<State>(id: u64, state: (ButtonState, Binding<State, ButtonState>)) -> Button<State> {
    Button {
        id,
        body: None,
        label: None,
        text_label: None,
        background_corner_rounding: None,
        background_padding: DEFAULT_PADDING,
        on_click: None,
        state: state.0,
        binding: state.1,
        background_fill: None,
        background_stroke: None,
        text_fill: None,
    }
}

impl<State> Button<State> {
    pub fn surface(mut self, body: Layout<'static, View<State>, AppCtx>) -> Self {
        self.body = Some(body);
        self
    }
    pub fn label(mut self, label: Layout<'static, View<State>, AppCtx>) -> Self {
        self.label = Some(label);
        self
    }
    pub fn text_label(mut self, text_label: impl AsRef<str>) -> Self {
        self.text_label = Some(text_label.as_ref().to_string());
        self
    }
    pub fn background_corner_rounding(mut self, corner_rounding: f32) -> Self {
        self.background_corner_rounding = Some(corner_rounding);
        self
    }
    pub fn on_click(
        mut self,
        on_click: impl Fn(&mut State, &mut AppState) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(on_click));
        self
    }
    pub fn background_fill(mut self, fill: impl Into<BrushSource<ButtonState>>) -> Self {
        self.background_fill = Some(fill.into());
        self
    }
    pub fn background_stroke(
        mut self,
        brush: impl Into<BrushSource<ButtonState>>,
        style: Stroke,
    ) -> Self {
        self.background_stroke = Some((brush.into(), style));
        self
    }
    pub fn background_padding(mut self, padding: f32) -> Self {
        self.background_padding = padding;
        self
    }
    pub fn text_fill(mut self, fill: impl Into<Brush>) -> Self {
        self.text_fill = Some(fill.into());
        self
    }
    pub fn build(self, ctx: &mut AppCtx) -> Layout<'static, View<State>, AppCtx>
    where
        State: 'static,
    {
        let btn_state = self.state;
        let bg_fill: BrushSource<ButtonState> = self.background_fill.unwrap_or(DEFAULT_PURP.into());
        let bg_stroke = self.background_stroke;
        let bg_rounding = self
            .background_corner_rounding
            .unwrap_or(DEFAULT_CORNER_ROUNDING);
        stack(vec![
            if let Some(body) = self.body {
                body
            } else {
                multi_draw(move |area, ctx: &mut AppCtx| {
                    let fill_brush = bg_fill.resolve(area, &btn_state);
                    let mut r = rect(crate::id!(self.id)).fill(adjust_brush(
                        &fill_brush,
                        btn_state.depressed,
                        btn_state.hovered,
                    ));
                    if let Some((ref stroke_source, ref stroke_style)) = bg_stroke {
                        let stroke_brush = stroke_source.resolve(area, &btn_state);
                        r = r.stroke(stroke_brush, stroke_style.clone());
                    }
                    r.corner_rounding(bg_rounding).build(ctx).draw(area, ctx)
                })
            },
            if let Some(label) = self.label {
                label
            } else {
                crate::text(
                    crate::id!(self.id),
                    self.text_label.clone().unwrap_or_default(),
                )
                .fill(adjust_brush(
                    &self.text_fill.unwrap_or(Brush::Solid(DEFAULT_FG)),
                    btn_state.depressed,
                    btn_state.hovered,
                ))
                .font_size(DEFAULT_FONT_SIZE)
                .view()
                // .transition_duration(0.)
                .finish(ctx)
            },
        ])
        .attach_over(
            rect(crate::id!(self.id))
                .fill(TRANSPARENT)
                .view()
                .on_hover({
                    let binding = self.binding.clone();
                    move |state, _app: &mut AppState, h| {
                        binding.update(state, |s| s.hovered = h)
                    }
                })
                .on_click({
                    let binding = self.binding.clone();
                    let on_click = self.on_click.clone();
                    move |state: &mut State, app: &mut AppState, click_state, _| {
                        match click_state {
                            ClickState::Started => binding.update(state, |s| s.depressed = true),
                            ClickState::Cancelled => binding.update(state, |s| s.depressed = false),
                            ClickState::Completed => {
                                if let Some(f) = &on_click {
                                    f(state, app);
                                }
                                binding.update(state, |s| s.depressed = false)
                            }
                        }
                    }
                })
                .finish(ctx),
        )
    }
}
