use crate::{
    Binding, Color, DEFAULT_DARK_GRAY, DEFAULT_FG, DEFAULT_GRAY, DEFAULT_PURP, TRANSPARENT,
    app::AppState, id, rect,
};
use backer::{
    Node,
    nodes::{area_reader, stack},
};

#[derive(Debug, Clone, Copy, Default)]
pub struct SliderState {
    pub hovered: bool,
    pub dragging: bool,
    pub value: f32,
}

pub struct Slider<State> {
    id: u64,
    min: f32,
    max: f32,
    on_change: Option<fn(&mut State, &mut AppState<State>, f32)>,
    state: Binding<State, SliderState>,
    knob_fill: Color,
}

pub fn slider<State>(id: u64, binding: Binding<State, SliderState>) -> Slider<State> {
    Slider {
        id,
        min: 0.0,
        max: 1.0,
        on_change: None,
        state: binding,
        knob_fill: DEFAULT_FG,
    }
}

impl<State> Slider<State> {
    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub fn on_change(mut self, on_change: fn(&mut State, &mut AppState<State>, f32)) -> Self {
        self.on_change = Some(on_change);
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
            let normalized_value = (self.state.get(state).value - self.min) / (self.max - self.min);
            let slider_width = (width - (height)) * normalized_value + height;

            stack(vec![
                rect(id!(self.id))
                    .fill(DEFAULT_GRAY)
                    .corner_rounding(height * 0.5)
                    .finish()
                    .height(height)
                    .width(width),
                rect(id!(self.id))
                    .fill(DEFAULT_DARK_GRAY)
                    .corner_rounding(height)
                    .finish()
                    .pad(height * 0.3)
                    .height(height)
                    .width(width),
                rect(id!(self.id))
                    .fill(DEFAULT_PURP)
                    .corner_rounding(height)
                    .finish()
                    .pad(height * 0.2)
                    .height(height)
                    .width(slider_width)
                    .offset_x((-width * 0.5) + (slider_width * 0.5)),
                rect(id!(self.id))
                    .fill(
                        match (
                            self.state.get(state).dragging,
                            self.state.get(state).hovered,
                        ) {
                            (true, _) => self.knob_fill.map_lightness(|l| l - 0.1),
                            (false, true) => self.knob_fill.map_lightness(|l| l + 0.1),
                            (false, false) => self.knob_fill,
                        },
                    )
                    .corner_rounding(height)
                    .finish()
                    .pad(height * 0.2)
                    .height(if self.state.get(state).dragging {
                        height * 1.1
                    } else {
                        height
                    })
                    .width(height)
                    .offset_x((-width * 0.5) + slider_width - (height * 0.5)),
                rect(id!(self.id))
                    .fill(TRANSPARENT)
                    .view()
                    .on_hover({
                        let binding = self.state.clone();
                        move |state: &mut State, _app: &mut AppState<State>, h| {
                            binding.update(state, |s| s.hovered = h)
                        }
                    })
                    .on_drag({
                        let binding = self.state.clone();
                        let min = self.min;
                        let max = self.max;
                        let on_change = self.on_change;
                        move |state: &mut State, app: &mut AppState<State>, drag_state| {
                            let update_value = |x: f64| {
                                let gesture_padding = 0.2;
                                let padded_start = gesture_padding * width;
                                let padded_end = width - (gesture_padding * width);
                                let padded_width = padded_end - padded_start;
                                let normalized = ((x - padded_start as f64) / padded_width as f64)
                                    .clamp(0.0, 1.0)
                                    as f32;
                                min + normalized * (max - min)
                            };

                            match drag_state {
                                crate::DragState::Began { start, .. } => {
                                    binding.update(state, |s| s.dragging = true);
                                    let new_value = update_value(start.x);
                                    binding.update(state, |s| s.value = new_value);
                                    if let Some(f) = on_change {
                                        f(state, app, new_value);
                                    }
                                }
                                crate::DragState::Updated { current, .. } => {
                                    let new_value = update_value(current.x);
                                    binding.update(state, |s| s.value = new_value);
                                    if let Some(f) = on_change {
                                        f(state, app, new_value);
                                    }
                                }
                                crate::DragState::Completed { .. } => {
                                    binding.update(state, |s| s.dragging = false);
                                }
                            }
                        }
                    })
                    .finish()
                    .height(height)
                    .width(width),
            ])
        })
        .aspect(3.)
    }
}
