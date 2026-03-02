use crate::{
    Binding, ClickState, DEFAULT_CORNER_ROUNDING, DEFAULT_FONT_SIZE, DEFAULT_PADDING, DEFAULT_PURP,
    app::{AppContext, AppState, DrawItem},
    rect,
};
use crate::{Color, DEFAULT_FG};
use backer::{Layout, nodes::stack};
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
    body: Option<Layout<DrawItem<State>, AppContext>>,
    label: Option<Layout<DrawItem<State>, AppContext>>,
    text_label: Option<String>,
    background_corner_rounding: Option<f32>,
    background_padding: f32,
    on_click: Option<Rc<dyn Fn(&mut State, &mut AppState<State>)>>,
    state: ButtonState,
    binding: Binding<State, ButtonState>,
    background_fill: Option<FillStyle>,
    background_stroke: Option<(Brush, Stroke)>,
    text_fill: Option<Color>,
}

enum FillStyle {
    Auto(Brush),
    Custom(Rc<dyn Fn(ButtonState) -> Brush>),
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
    pub fn surface(mut self, body: Layout<DrawItem<State>, AppContext>) -> Self {
        self.body = Some(body);
        self
    }
    pub fn label(mut self, label: Layout<DrawItem<State>, AppContext>) -> Self {
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
        on_click: impl Fn(&mut State, &mut AppState<State>) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(on_click));
        self
    }
    pub fn background_fill(mut self, fill: impl Into<Brush>) -> Self {
        self.background_fill = Some(FillStyle::Auto(fill.into()));
        self
    }
    pub fn background_fill_with(mut self, f: impl Fn(ButtonState) -> Brush + 'static) -> Self {
        self.background_fill = Some(FillStyle::Custom(Rc::new(f)));
        self
    }
    pub fn background_stroke(mut self, brush: impl Into<Brush>, style: Stroke) -> Self {
        self.background_stroke = Some((brush.into(), style));
        self
    }
    pub fn background_padding(mut self, padding: f32) -> Self {
        self.background_padding = padding;
        self
    }
    pub fn text_fill(mut self, color: Color) -> Self {
        self.text_fill = Some(color);
        self
    }
    pub fn build(self, ctx: &mut AppContext) -> Layout<DrawItem<State>, AppContext>
    where
        State: 'static,
    {
        let btn_state = self.state;
        let default_fill = FillStyle::Auto(DEFAULT_PURP.into());
        let bg_fill = self.background_fill.unwrap_or(default_fill);
        let resolved_fill = match &bg_fill {
            FillStyle::Auto(brush) => match brush {
                Brush::Solid(color) => {
                    let adjusted = match (btn_state.depressed, btn_state.hovered) {
                        (true, _) => color.map_lightness(|l| l - 0.1),
                        (false, true) => color.map_lightness(|l| l + 0.1),
                        (false, false) => *color,
                    };
                    Brush::Solid(adjusted)
                }
                other => other.clone(),
            },
            FillStyle::Custom(f) => f(btn_state),
        };
        let bg_stroke = self.background_stroke;
        let bg_rounding = self
            .background_corner_rounding
            .unwrap_or(DEFAULT_CORNER_ROUNDING);
        stack(vec![
            if let Some(body) = self.body {
                body
            } else {
                rect(crate::id!(self.id))
                    .fill(resolved_fill)
                    .stroke(
                        bg_stroke
                            .as_ref()
                            .map(|s| s.0.clone())
                            .unwrap_or(TRANSPARENT.into()),
                        bg_stroke.map(|s| s.1).unwrap_or(Stroke::new(0.)),
                    )
                    .corner_rounding(bg_rounding)
                    .view()
                    .finish(ctx)
            },
            if let Some(label) = self.label {
                label
            } else {
                crate::text(
                    crate::id!(self.id),
                    self.text_label.clone().unwrap_or_default(),
                )
                .fill(match (btn_state.depressed, btn_state.hovered) {
                    (true, _) => self
                        .text_fill
                        .unwrap_or(DEFAULT_FG)
                        .map_lightness(|l| l - 0.1),
                    (false, true) => self
                        .text_fill
                        .unwrap_or(DEFAULT_FG)
                        .map_lightness(|l| l + 0.1),
                    (false, false) => self.text_fill.unwrap_or(DEFAULT_FG),
                })
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
                    move |state, _app: &mut AppState<State>, h| {
                        binding.update(state, |s| s.hovered = h)
                    }
                })
                .on_click({
                    let binding = self.binding.clone();
                    let on_click = self.on_click.clone();
                    move |state: &mut State, app: &mut AppState<State>, click_state, _| {
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
