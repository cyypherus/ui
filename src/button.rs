use crate::{dynamic_node, view::View, Binding, ClickState, RcUi, DEFAULT_FONT_SIZE};
use backer::{nodes::stack, Node};
use vello_svg::vello::peniko::color::AlphaColor;

#[derive(Debug, Clone, Copy, Default)]
pub struct ButtonState {
    pub hovered: bool,
    pub depressed: bool,
}

pub struct Button<State> {
    id: u64,
    body: Option<View<State, ()>>,
    label: Option<View<State, ()>>,
    text_label: Option<String>,
    corner_rounding: Option<f32>,
    on_click: Option<fn(&mut State)>,
    state: Binding<State, ButtonState>,
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
    }
}

impl<State> Button<State> {
    pub fn body(mut self, body: View<State, ()>) -> Self {
        self.body = Some(body);
        self
    }
    pub fn label(mut self, label: View<State, ()>) -> Self {
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
    pub fn on_click(mut self, on_click: fn(&mut State)) -> Self {
        self.on_click = Some(on_click);
        self
    }
    pub fn finish<'n>(self) -> Node<'n, RcUi<State>>
    where
        State: 'static,
    {
        dynamic_node(move |s: &mut State| {
            stack(vec![
                self.body
                    .clone()
                    .unwrap_or(
                        crate::rect(crate::id!(self.id))
                            .fill(
                                match (self.state.get(s).depressed, self.state.get(s).hovered) {
                                    (true, _) => AlphaColor::from_rgb8(93, 50, 212),
                                    (false, true) => AlphaColor::from_rgb8(133, 90, 252),
                                    (false, false) => AlphaColor::from_rgb8(113, 70, 232),
                                },
                            )
                            .corner_rounding(self.corner_rounding.unwrap_or(10.))
                            .view(),
                    )
                    .on_hover({
                        let binding = self.state.clone();
                        move |s: &mut State, h| binding.update(s, |s| s.hovered = h)
                    })
                    .on_click({
                        let binding = self.state.clone();
                        move |s: &mut State, click_state| match click_state {
                            ClickState::Started => binding.update(s, |s| s.depressed = true),
                            ClickState::Cancelled => binding.update(s, |s| s.depressed = false),
                            ClickState::Completed => {
                                if let Some(f) = self.on_click {
                                    f(s);
                                }
                                binding.update(s, |s| s.depressed = false)
                            }
                        }
                    })
                    .transition_duration(0.)
                    .finish(),
                self.label
                    .clone()
                    .unwrap_or(
                        crate::text(
                            crate::id!(self.id),
                            self.text_label.clone().unwrap_or_default(),
                        )
                        .fill(
                            match (self.state.get(s).depressed, self.state.get(s).hovered) {
                                (true, _) => AlphaColor::from_rgb8(190, 190, 190),
                                (false, true) => AlphaColor::from_rgb8(250, 250, 250),
                                (false, false) => AlphaColor::from_rgb8(200, 200, 200),
                            },
                        )
                        .font_size(DEFAULT_FONT_SIZE)
                        .view(),
                    )
                    .transition_duration(0.)
                    .finish(),
            ])
        })
    }
}
