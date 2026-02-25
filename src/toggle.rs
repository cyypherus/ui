use crate::app::{AppContext, DrawItem};
use crate::{Binding, ClickState, app::AppState, id, rect};
use crate::{Color, DEFAULT_FG, DEFAULT_GRAY, DEFAULT_LIGHT_GRAY, TRANSPARENT, circle};
use backer::{
    Layout,
    nodes::{area_reader, stack},
};

#[derive(Default, Debug, Clone, Copy)]
pub struct ToggleState {
    pub hovered: bool,
    pub depressed: bool,
    pub on: bool,
}

impl ToggleState {
    pub fn on() -> Self {
        ToggleState {
            hovered: false,
            depressed: false,
            on: true,
        }
    }

    pub fn off() -> Self {
        ToggleState {
            hovered: false,
            depressed: false,
            on: false,
        }
    }
}

pub struct Toggle<State> {
    id: u64,
    on_toggle: Option<fn(&mut State, &mut AppState<State>, bool)>,
    state: ToggleState,
    binding: Binding<State, ToggleState>,
    on_fill: Color,
    off_fill: Color,
    knob_fill: Color,
}

pub fn toggle<State>(id: u64, state: (ToggleState, Binding<State, ToggleState>)) -> Toggle<State> {
    Toggle {
        id,
        on_toggle: None,
        state: state.0,
        binding: state.1,
        on_fill: DEFAULT_LIGHT_GRAY,
        off_fill: DEFAULT_GRAY,
        knob_fill: DEFAULT_FG,
    }
}

impl<State> Toggle<State> {
    pub fn on_toggle(mut self, on_toggle: fn(&mut State, &mut AppState<State>, bool)) -> Self {
        self.on_toggle = Some(on_toggle);
        self
    }

    pub fn on_fill(mut self, fill: Color) -> Self {
        self.on_fill = fill;
        self
    }

    pub fn off_fill(mut self, fill: Color) -> Self {
        self.off_fill = fill;
        self
    }

    pub fn knob_fill(mut self, fill: Color) -> Self {
        self.knob_fill = fill;
        self
    }
    pub fn finish(self, _ctx: &mut AppContext) -> Layout<DrawItem<State>, AppContext>
    where
        State: 'static,
    {
        let state = self.state;
        area_reader(move |area, ctx: &mut AppContext| {
            let width = area.width;
            let height = area.height;
            stack(vec![
                rect(id!(self.id))
                    .fill(if state.on {
                        self.on_fill
                    } else {
                        self.off_fill
                    })
                    .corner_rounding(height * 0.5)
                    .finish(ctx)
                    .height(height)
                    .width(width),
                circle(id!(self.id))
                    .fill(match (state.depressed, state.hovered) {
                        (true, _) => self.knob_fill.map_lightness(|l| l - 0.1),
                        (false, true) => self.knob_fill.map_lightness(|l| l + 0.1),
                        (false, false) => self.knob_fill,
                    })
                    .finish(ctx)
                    .pad(height * 0.1)
                    .height(height)
                    .width(height)
                    .offset(
                        {
                            let button_padding = height - (height * 0.5);
                            if state.on {
                                (width * 0.5) - button_padding
                            } else {
                                (-width * 0.5) + button_padding
                            }
                        },
                        0.,
                    ),
                rect(crate::id!(self.id))
                    .fill(TRANSPARENT)
                    .view()
                    .on_hover({
                        let binding = self.binding.clone();
                        move |state: &mut State, _app: &mut AppState<State>, h| {
                            binding.update(state, |s| s.hovered = h)
                        }
                    })
                    .on_click({
                        let binding = self.binding.clone();
                        move |state: &mut State, app: &mut AppState<State>, click_state, _| {
                            match click_state {
                                ClickState::Started => {
                                    binding.update(state, |s| s.depressed = true)
                                }
                                ClickState::Cancelled => {
                                    binding.update(state, |s| s.depressed = false)
                                }
                                ClickState::Completed => {
                                    if let Some(f) = self.on_toggle {
                                        f(state, app, !binding.get(state).on);
                                    }
                                    binding.update(state, |s| {
                                        s.on = !s.on;
                                        s.depressed = false
                                    })
                                }
                            }
                        }
                    })
                    .finish(ctx)
                    .height(height)
                    .width(width),
            ])
        })
    }
}
