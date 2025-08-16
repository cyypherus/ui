use crate::{Binding, ClickState, app::AppState, id, rect};
use crate::{Color, DEFAULT_FG, DEFAULT_GRAY, DEFAULT_LIGHT_GRAY, TRANSPARENT};
use backer::{
    Node,
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
    state: Binding<State, ToggleState>,
    on_fill: Color,
    off_fill: Color,
    knob_fill: Color,
}

pub fn toggle<State>(id: u64, binding: Binding<State, ToggleState>) -> Toggle<State> {
    Toggle {
        id,
        on_toggle: None,
        state: binding,
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
    pub fn finish<'n>(self) -> Node<'n, State, AppState<State>>
    where
        State: 'static,
    {
        area_reader(move |area, state, _app: &mut AppState<State>| {
            let width = area.width;
            let height = area.height;
            stack(vec![
                //
                rect(id!(self.id))
                    .fill(if self.state.get(state).on {
                        self.on_fill
                    } else {
                        self.off_fill
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
                            (true, _) => self.knob_fill.map_lightness(|l| l - 0.1),
                            (false, true) => self.knob_fill.map_lightness(|l| l + 0.1),
                            (false, false) => self.knob_fill,
                        },
                    )
                    .corner_rounding(height)
                    // .box_shadow(Color::from_rgba8(0, 0, 0, 100), 5.)
                    .finish()
                    .pad(height * 0.2)
                    .height(height)
                    .width(height)
                    .offset_x({
                        let button_padding = height - (height * 0.4);
                        if self.state.get(state).on {
                            (width * 0.5) - button_padding
                        } else {
                            (-width * 0.5) + button_padding
                        }
                    }),
                rect(crate::id!(self.id))
                    .fill(TRANSPARENT)
                    .view()
                    .on_hover({
                        let binding = self.state.clone();
                        move |state: &mut State, _app: &mut AppState<State>, h| {
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
                    .finish()
                    .height(height)
                    .width(width),
            ])
        })
        .aspect(2.)
    }
}
