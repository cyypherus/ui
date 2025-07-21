use crate::{Binding, ClickState, app::AppState, id, rect};
use backer::{
    Node,
    nodes::{dynamic, stack},
};
use vello_svg::vello::peniko::color::AlphaColor;

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
}

pub fn slider<State>(id: u64, binding: Binding<State, SliderState>) -> Slider<State> {
    Slider {
        id,
        min: 0.0,
        max: 1.0,
        on_change: None,
        state: binding,
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

    pub fn finish<'n>(self) -> Node<'n, State, AppState<State>>
    where
        State: 'static,
    {
        let height = 24.;
        let width = 200.;
        let handle_size = 20.;

        dynamic(move |state, _app: &mut AppState<State>| {
            let normalized_value = (self.state.get(state).value - self.min) / (self.max - self.min);
            let handle_position =
                normalized_value * (width - handle_size) - (width - handle_size) * 0.5;

            stack(vec![
                rect(id!(self.id))
                    .fill(AlphaColor::from_rgb8(100, 100, 100))
                    .corner_rounding(height * 0.5)
                    .finish()
                    .height(6.)
                    .width(width),
                rect(id!(self.id))
                    .fill(AlphaColor::from_rgb8(113, 70, 232))
                    .corner_rounding(height * 0.5)
                    .finish()
                    .height(6.)
                    .width((width * normalized_value).max(6.))
                    .offset_x(-(width - width * normalized_value) * 0.5),
                rect(id!(self.id))
                    .fill(
                        match (
                            self.state.get(state).dragging,
                            self.state.get(state).hovered,
                        ) {
                            (true, _) => AlphaColor::from_rgb8(190, 190, 190),
                            (false, true) => AlphaColor::from_rgb8(255, 255, 255),
                            (false, false) => AlphaColor::from_rgb8(230, 230, 230),
                        },
                    )
                    .corner_rounding(handle_size * 0.5)
                    .box_shadow(AlphaColor::from_rgba8(0, 0, 0, 100), 3.)
                    .finish()
                    .height(handle_size)
                    .width(handle_size)
                    .offset_x(handle_position),
                rect(id!(self.id))
                    .fill(AlphaColor::TRANSPARENT)
                    .view()
                    .on_hover({
                        let binding = self.state.clone();
                        move |state: &mut State, _app: &mut AppState<State>, h| {
                            binding.update(state, |s| s.hovered = h)
                        }
                    })
                    .on_click({
                        let binding = self.state.clone();
                        let min = self.min;
                        let max = self.max;
                        let on_change = self.on_change;
                        move |state: &mut State,
                              app: &mut AppState<State>,
                              click_state,
                              position| {
                            match click_state {
                                ClickState::Started => {
                                    binding.update(state, |s| s.dragging = true);
                                    let pos = position.local();
                                    let normalized = ((pos.x + width as f64 * 0.5) / width as f64)
                                        .clamp(0.0, 1.0)
                                        as f32;
                                    let new_value = min + normalized * (max - min);
                                    binding.update(state, |s| s.value = new_value);
                                    if let Some(f) = on_change {
                                        f(state, app, new_value);
                                    }
                                }
                                ClickState::Cancelled => {
                                    binding.update(state, |s| s.dragging = false)
                                }
                                ClickState::Completed => {
                                    binding.update(state, |s| s.dragging = false)
                                }
                            }
                        }
                    })
                    .on_drag({
                        let binding = self.state.clone();
                        let min = self.min;
                        let max = self.max;
                        let on_change = self.on_change;
                        move |state: &mut State, app: &mut AppState<State>, drag_state| {
                            if binding.get(state).dragging {
                                match drag_state {
                                    crate::DragState::Updated { current, .. }
                                    | crate::DragState::Began(current) => {
                                        let normalized = ((current.x + width as f64 * 0.5)
                                            / width as f64)
                                            .clamp(0.0, 1.0)
                                            as f32;
                                        let new_value = min + normalized * (max - min);
                                        binding.update(state, |s| s.value = new_value);
                                        if let Some(f) = on_change {
                                            f(state, app, new_value);
                                        }
                                    }
                                    _ => {}
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
