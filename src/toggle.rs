use crate::{app::AppState, id, rect, Binding, ClickState};
use backer::{
    nodes::{dynamic, stack},
    Node,
};
use vello_svg::vello::peniko::color::AlphaColor;

#[derive(Debug, Clone, Copy, Default)]
pub struct ToggleState {
    pub hovered: bool,
    pub depressed: bool,
    pub on: bool,
}

pub struct Toggle<State> {
    id: u64,
    on_toggle: Option<fn(&mut State, &mut AppState, bool)>,
    state: Binding<State, ToggleState>,
}

pub fn toggle<State>(id: u64, binding: Binding<State, ToggleState>) -> Toggle<State> {
    Toggle {
        id,
        on_toggle: None,
        state: binding,
    }
}

impl<State> Toggle<State> {
    pub fn on_toggle(mut self, on_toggle: fn(&mut State, &mut AppState, bool)) -> Self {
        self.on_toggle = Some(on_toggle);
        self
    }
    pub fn finish<'n>(self) -> Node<'n, State, AppState>
    where
        State: 'static,
    {
        let height = 60.;
        let width = 120.;
        dynamic(move |state, app: &mut AppState| {
            stack(vec![
                //
                rect(id!(self.id))
                    .fill(if self.state.get(state).on {
                        AlphaColor::from_rgb8(113, 70, 232)
                    } else {
                        AlphaColor::from_rgb8(50, 50, 50)
                    })
                    .corner_rounding(height * 0.5)
                    .finish()
                    .height(height)
                    .width(width),
                rect(id!(self.id))
                    .fill(
                        match (
                            self.state.get(state).depressed,
                            self.state.get(state).hovered,
                        ) {
                            (true, _) => AlphaColor::from_rgb8(190, 190, 190),
                            (false, true) => AlphaColor::from_rgb8(255, 255, 255),
                            (false, false) => AlphaColor::from_rgb8(230, 230, 230),
                        },
                    )
                    .corner_rounding(height)
                    .box_shadow(AlphaColor::from_rgba8(0, 0, 0, 100), 5.)
                    .finish()
                    .pad(8.)
                    .height(height)
                    .width(height)
                    .offset_x({
                        if self.state.get(state).on {
                            width * 0.25
                        } else {
                            -width * 0.25
                        }
                    }),
                rect(crate::id!(self.id))
                    .fill(AlphaColor::TRANSPARENT)
                    .view()
                    .on_hover({
                        let binding = self.state.clone();
                        move |state: &mut State, app: &mut AppState, h| {
                            binding.update(state, |s| s.hovered = h)
                        }
                    })
                    .on_click({
                        let binding = self.state.clone();
                        move |state: &mut State, app: &mut AppState, click_state, _| {
                            match click_state {
                                ClickState::Started => {
                                    binding.update(state, |s| s.depressed = true)
                                }
                                ClickState::Cancelled => {
                                    binding.update(state, |s| s.depressed = false)
                                }
                                ClickState::Completed => {
                                    if let Some(f) = self.on_toggle {
                                        f(state, app, binding.get(state).on);
                                    }
                                    binding.update(state, |s| {
                                        s.on = !s.on;
                                        s.depressed = false
                                    })
                                }
                            }
                        }
                    })
                    .finish()
                    .height(height)
                    .width(width),
            ])
        })
    }
}
