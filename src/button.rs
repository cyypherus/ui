use crate::{
    Binding, ClickState, DEEP_PURP, DEFAULT_CORNER_ROUNDING, DEFAULT_FONT_SIZE, app::AppState, rect,
};
use backer::{
    Node,
    nodes::{dynamic, stack},
};
use vello_svg::vello::peniko::Color;

#[derive(Debug, Clone, Copy, Default)]
pub struct ButtonState {
    pub hovered: bool,
    pub depressed: bool,
}

type BodyFn<'n, State> = fn(&mut State, ButtonState) -> Node<'n, State, AppState<State>>;
type LabelFn<'n, State> =
    Box<dyn Fn(&mut State, ButtonState) -> Node<'n, State, AppState<State>> + 'n>;

pub struct Button<'n, State> {
    id: u64,
    body: Option<BodyFn<'n, State>>,
    label: Option<LabelFn<'n, State>>,
    text_label: Option<String>,
    corner_rounding: Option<f32>,
    on_click: Option<fn(&mut State, &mut AppState<State>)>,
    state: Binding<State, ButtonState>,
    fill: Option<Color>,
    text_fill: Option<Color>,
}

pub fn button<'n, State>(id: u64, binding: Binding<State, ButtonState>) -> Button<'n, State> {
    Button {
        id,
        body: None,
        label: None,
        text_label: None,
        corner_rounding: None,
        on_click: None,
        state: binding,
        fill: None,
        text_fill: None,
    }
}

impl<'n, State> Button<'n, State> {
    pub fn surface(
        mut self,
        body: fn(&mut State, ButtonState) -> Node<'n, State, AppState<State>>,
    ) -> Self {
        self.body = Some(body);
        self
    }
    pub fn label(
        mut self,
        label: impl Fn(&mut State, ButtonState) -> Node<'n, State, AppState<State>> + 'n,
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
    pub fn on_click(mut self, on_click: fn(&mut State, &mut AppState<State>)) -> Self {
        self.on_click = Some(on_click);
        self
    }
    pub fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }
    pub fn idle_text_fill(mut self, color: Color) -> Self {
        self.text_fill = Some(color);
        self
    }
    pub fn finish(self) -> Node<'n, State, AppState<State>>
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
                                    self.fill.unwrap_or(DEEP_PURP).map_lightness(|l| l - 0.1)
                                }
                                (false, true) => {
                                    self.fill.unwrap_or(DEEP_PURP).map_lightness(|l| l + 0.1)
                                }
                                (false, false) => self.fill.unwrap_or(DEEP_PURP),
                            },
                        )
                        .corner_rounding(self.corner_rounding.unwrap_or(DEFAULT_CORNER_ROUNDING))
                        .view()
                        // .transition_duration(0.)
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
                                .unwrap_or(DEEP_PURP)
                                .map_lightness(|l| l - 0.1),
                            (false, true) => self
                                .text_fill
                                .unwrap_or(DEEP_PURP)
                                .map_lightness(|l| l + 0.1),
                            (false, false) => self.text_fill.unwrap_or(DEEP_PURP),
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
                    .fill(Color::TRANSPARENT)
                    .view()
                    .on_hover({
                        let binding = self.state.clone();
                        move |state, _app: &mut AppState<State>, h| {
                            binding.update(state, |s| s.hovered = h)
                        }
                    })
                    .on_click({
                        let binding = self.state.clone();
                        move |state: &mut State, app: &mut AppState<State>, click_state, _| {
                            match click_state {
                                ClickState::Started => {
                                    binding.update(state, |s| s.depressed = true)
                                }
                                ClickState::Cancelled => {
                                    binding.update(state, |s| s.depressed = false)
                                }
                                ClickState::Completed => {
                                    if let Some(f) = self.on_click {
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
