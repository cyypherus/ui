use crate::{app::AppState, rect, Binding, ClickState, DEFAULT_CORNER_ROUNDING, DEFAULT_FONT_SIZE};
use backer::{
    nodes::{dynamic, stack},
    Node,
};
use vello_svg::vello::peniko::color::AlphaColor;

#[derive(Debug, Clone, Copy, Default)]
pub struct ButtonState {
    pub hovered: bool,
    pub depressed: bool,
}

type BodyFn<'n, State> = fn(ButtonState) -> Node<'n, State, AppState>;
type LabelFn<'n, State> = fn(ButtonState) -> Node<'n, State, AppState>;

pub struct Button<'n, State> {
    id: u64,
    body: Option<BodyFn<'n, State>>,
    label: Option<LabelFn<'n, State>>,
    text_label: Option<String>,
    corner_rounding: Option<f32>,
    on_click: Option<fn(&mut State, &mut AppState)>,
    state: Binding<State, ButtonState>,
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
    }
}

impl<'n, State> Button<'n, State> {
    pub fn body(mut self, body: fn(ButtonState) -> Node<'n, State, AppState>) -> Self {
        self.body = Some(body);
        self
    }
    pub fn label(mut self, label: fn(ButtonState) -> Node<'n, State, AppState>) -> Self {
        self.label = Some(label);
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
    pub fn on_click(mut self, on_click: fn(&mut State, &mut AppState)) -> Self {
        self.on_click = Some(on_click);
        self
    }
    pub fn finish(self) -> Node<'n, State, AppState>
    where
        State: 'static,
    {
        dynamic(move |state: &mut State, app: &mut AppState| {
            stack(vec![
                if let Some(body) = self.body {
                    body(self.state.get(state))
                } else {
                    rect(crate::id!(self.id))
                        .fill(
                            match (
                                self.state.get(state).depressed,
                                self.state.get(state).hovered,
                            ) {
                                (true, _) => AlphaColor::from_rgb8(93, 50, 212),
                                (false, true) => AlphaColor::from_rgb8(133, 90, 252),
                                (false, false) => AlphaColor::from_rgb8(113, 70, 232),
                            },
                        )
                        .corner_rounding(self.corner_rounding.unwrap_or(DEFAULT_CORNER_ROUNDING))
                        .view()
                        .transition_duration(0.)
                        .finish()
                },
                if let Some(label) = self.label {
                    label(self.state.get(state))
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
                            (true, _) => AlphaColor::from_rgb8(190, 190, 190),
                            (false, true) => AlphaColor::from_rgb8(250, 250, 250),
                            (false, false) => AlphaColor::from_rgb8(240, 240, 240),
                        },
                    )
                    .font_size(DEFAULT_FONT_SIZE)
                    .view()
                    .transition_duration(0.)
                    .finish()
                },
                rect(crate::id!(self.id))
                    .fill(AlphaColor::TRANSPARENT)
                    .view()
                    .on_hover({
                        let binding = self.state.clone();
                        move |state, app: &mut AppState, h| binding.update(app, |s| s.hovered = h)
                    })
                    .on_click({
                        let binding = self.state.clone();
                        move |state: &mut State, app: &mut AppState, click_state, _| {
                            match click_state {
                                ClickState::Started => binding.update(app, |s| s.depressed = true),
                                ClickState::Cancelled => {
                                    binding.update(app, |s| s.depressed = false)
                                }
                                ClickState::Completed => {
                                    if let Some(f) = self.on_click {
                                        f(app);
                                    }
                                    binding.update(app, |s| s.depressed = false)
                                }
                            }
                        }
                    })
                    .finish(),
            ])
        })
    }
}
