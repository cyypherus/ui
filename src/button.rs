use crate::{
    Binding, ClickState, DEFAULT_CORNER_ROUNDING, DEFAULT_FONT_SIZE, DEFAULT_PURP,
    app::{AppState, DrawItem},
    rect,
};
use crate::{Color, DEFAULT_FG};
use backer::{Layout, nodes::stack};
use std::rc::Rc;
use vello_svg::vello::peniko::color::palette::css::TRANSPARENT;

#[derive(Debug, Clone, Copy, Default)]
pub struct ButtonState {
    pub hovered: bool,
    pub depressed: bool,
}

type BodyFn<State> = fn(&mut State, ButtonState) -> Layout<DrawItem<State>>;
type LabelFn<State> = Box<dyn Fn(&mut State, ButtonState) -> Layout<DrawItem<State>>>;

pub struct Button<State> {
    id: u64,
    body: Option<BodyFn<State>>,
    label: Option<LabelFn<State>>,
    text_label: Option<String>,
    corner_rounding: Option<f32>,
    on_click: Option<Rc<dyn Fn(&mut State, &mut AppState<State>)>>,
    state: Binding<State, ButtonState>,
    fill: Option<Color>,
    stroke: Option<(Color, f32)>,
    text_fill: Option<Color>,
}

pub fn button<State>(id: u64, binding: Binding<State, ButtonState>) -> Button<State> {
    Button {
        id,
        body: None,
        label: None,
        text_label: None,
        corner_rounding: None,
        on_click: None,
        state: binding,
        fill: None,
        stroke: None,
        text_fill: None,
    }
}

impl<State> Button<State> {
    pub fn surface(mut self, body: fn(&mut State, ButtonState) -> Layout<DrawItem<State>>) -> Self {
        self.body = Some(body);
        self
    }
    pub fn label(
        mut self,
        label: impl Fn(&mut State, ButtonState) -> Layout<DrawItem<State>> + 'static,
    ) -> Self {
        self.label = Some(Box::new(label));
        self
    }
    pub fn text_label(mut self, text_label: impl AsRef<str>) -> Self {
        self.text_label = Some(text_label.as_ref().to_string());
        self
    }
    pub fn corner_rounding(mut self, corner_rounding: f32) -> Self {
        self.corner_rounding = Some(corner_rounding);
        self
    }
    pub fn on_click(
        mut self,
        on_click: impl Fn(&mut State, &mut AppState<State>) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(on_click));
        self
    }
    pub fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }
    pub fn stroke(mut self, color: Color, line_width: f32) -> Self {
        self.stroke = Some((color, line_width));
        self
    }
    pub fn idle_text_fill(mut self, color: Color) -> Self {
        self.text_fill = Some(color);
        self
    }
    pub fn finish(self) -> Layout<DrawItem<State>>
    where
        State: 'static,
    {
        dynamic(move |state: &mut State, _app: &mut AppState<State>| {
            stack(vec![
                if let Some(body) = self.body {
                    let vs = self.state.get(state);
                    body(state, vs)
                } else {
                    rect(crate::id!(self.id))
                        .fill(
                            match (
                                self.state.get(state).depressed,
                                self.state.get(state).hovered,
                            ) {
                                (true, _) => {
                                    self.fill.unwrap_or(DEFAULT_PURP).map_lightness(|l| l - 0.1)
                                }
                                (false, true) => {
                                    self.fill.unwrap_or(DEFAULT_PURP).map_lightness(|l| l + 0.1)
                                }
                                (false, false) => self.fill.unwrap_or(DEFAULT_PURP),
                            },
                        )
                        .stroke(
                            self.stroke.map(|s| s.0).unwrap_or(TRANSPARENT),
                            self.stroke.map(|s| s.1).unwrap_or(0.),
                        )
                        .corner_rounding(self.corner_rounding.unwrap_or(DEFAULT_CORNER_ROUNDING))
                        .view()
                        .finish()
                },
                if let Some(label) = &self.label {
                    let vs = self.state.get(state);
                    label(state, vs)
                } else {
                    crate::text(
                        crate::id!(self.id),
                        self.text_label.clone().unwrap_or_default(),
                    )
                    .fill(
                        match (
                            self.state.get(state).depressed,
                            self.state.get(state).hovered,
                        ) {
                            (true, _) => self
                                .text_fill
                                .unwrap_or(DEFAULT_FG)
                                .map_lightness(|l| l - 0.1),
                            (false, true) => self
                                .text_fill
                                .unwrap_or(DEFAULT_FG)
                                .map_lightness(|l| l + 0.1),
                            (false, false) => self.text_fill.unwrap_or(DEFAULT_FG),
                        },
                    )
                    .font_size(DEFAULT_FONT_SIZE)
                    .view()
                    // .transition_duration(0.)
                    .finish()
                },
            ])
            .attach_over(
                rect(crate::id!(self.id))
                    .fill(TRANSPARENT)
                    .view()
                    .on_hover({
                        let binding = self.state.clone();
                        move |state, _app: &mut AppState<State>, h| {
                            binding.update(state, |s| s.hovered = h)
                        }
                    })
                    .on_click({
                        let binding = self.state.clone();
                        let on_click = self.on_click.clone();
                        move |state: &mut State, app: &mut AppState<State>, click_state, _| {
                            match click_state {
                                ClickState::Started => {
                                    binding.update(state, |s| s.depressed = true)
                                }
                                ClickState::Cancelled => {
                                    binding.update(state, |s| s.depressed = false)
                                }
                                ClickState::Completed => {
                                    if let Some(f) = &on_click {
                                        f(state, app);
                                    }
                                    binding.update(state, |s| s.depressed = false)
                                }
                            }
                        }
                    })
                    .finish(),
            )
        })
    }
}
