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
    on_toggle: Option<fn(&mut AppState<State>, bool)>,
    state: Binding<AppState<State>, ToggleState>,
}

pub fn toggle<State>(id: u64, binding: Binding<AppState<State>, ToggleState>) -> Toggle<State> {
    Toggle {
        id,
        on_toggle: None,
        state: binding,
    }
}

impl<State> Toggle<State> {
    pub fn on_toggle(mut self, on_toggle: fn(&mut AppState<State>, bool)) -> Self {
        self.on_toggle = Some(on_toggle);
        self
    }
    pub fn finish<'n>(self) -> Node<'n, AppState<State>>
    where
        State: 'static,
    {
        let height = 60.;
        let width = 120.;
        dynamic(move |s: &mut AppState<State>| {
            stack(vec![
                //
                rect(id!(self.id))
                    .fill(if self.state.get(s).on {
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
                        match (self.state.get(s).depressed, self.state.get(s).hovered) {
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
                        if self.state.get(s).on {
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
                        move |s: &mut AppState<State>, h| binding.update(s, |s| s.hovered = h)
                    })
                    .on_click({
                        let binding = self.state.clone();
                        move |s: &mut AppState<State>, click_state, _| match click_state {
                            ClickState::Started => binding.update(s, |s| s.depressed = true),
                            ClickState::Cancelled => binding.update(s, |s| s.depressed = false),
                            ClickState::Completed => {
                                if let Some(f) = self.on_toggle {
                                    f(s, binding.get(s).on);
                                }
                                binding.update(s, |s| {
                                    s.on = !s.on;
                                    s.depressed = false
                                })
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
