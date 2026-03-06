use crate::background_style::BrushSource;
use crate::{
    Binding, DEFAULT_DARK_GRAY, DEFAULT_FG, DEFAULT_GRAY, DEFAULT_PURP, DragState, TRANSPARENT,
    adjust_brush,
    app::{AppCtx, AppState, View},
    circle, id, rect,
};
use backer::{
    Layout,
    nodes::{draw, stack},
};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, Default)]
pub struct SliderState {
    pub hovered: bool,
    pub dragging: bool,
    pub value: f32,
}

pub struct Slider<State> {
    id: u64,
    state: SliderState,
    binding: Binding<State, SliderState>,
    min: f32,
    max: f32,
    on_change: Option<Rc<dyn Fn(&mut State, &mut AppState, f32)>>,
    knob_fill: BrushSource<SliderState>,
    background_fill: BrushSource<SliderState>,
    track_fill: BrushSource<SliderState>,
    traveled_track_fill: BrushSource<SliderState>,
}

pub fn slider<State>(id: u64, state: (SliderState, Binding<State, SliderState>)) -> Slider<State> {
    Slider {
        id,
        state: state.0,
        binding: state.1,
        min: 0.0,
        max: 1.0,
        on_change: None,
        knob_fill: DEFAULT_FG.into(),
        background_fill: DEFAULT_GRAY.into(),
        track_fill: DEFAULT_DARK_GRAY.into(),
        traveled_track_fill: DEFAULT_PURP.into(),
    }
}

impl<State> Slider<State> {
    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub fn on_change(
        mut self,
        on_change: impl Fn(&mut State, &mut AppState, f32) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(on_change));
        self
    }

    pub fn knob_fill(mut self, fill: impl Into<BrushSource<SliderState>>) -> Self {
        self.knob_fill = fill.into();
        self
    }

    pub fn background_fill(mut self, fill: impl Into<BrushSource<SliderState>>) -> Self {
        self.background_fill = fill.into();
        self
    }

    pub fn track_fill(mut self, fill: impl Into<BrushSource<SliderState>>) -> Self {
        self.track_fill = fill.into();
        self
    }

    pub fn traveled_track_fill(mut self, fill: impl Into<BrushSource<SliderState>>) -> Self {
        self.traveled_track_fill = fill.into();
        self
    }

    pub fn build(self, _ctx: &mut AppCtx) -> Layout<'static, View<State>, AppCtx>
    where
        State: 'static,
    {
        let state = self.state;
        draw(move |area, ctx: &mut AppCtx| {
            let width = area.width;
            let height = area.height;
            let normalized_value = (state.value - self.min) / (self.max - self.min);
            let slider_width = (width - height) * normalized_value + height;

            stack(vec![
                rect(id!(self.id))
                    .fill(self.background_fill.resolve(area, &state))
                    .corner_rounding(height * 0.5)
                    .build(ctx)
                    .height(height)
                    .width(width),
                rect(id!(self.id))
                    .fill(self.track_fill.resolve(area, &state))
                    .corner_rounding(height)
                    .build(ctx)
                    .pad(height * 0.3)
                    .height(height)
                    .width(width),
                rect(id!(self.id))
                    .fill(self.traveled_track_fill.resolve(area, &state))
                    .corner_rounding(height)
                    .build(ctx)
                    .pad(height * 0.2)
                    .height(height)
                    .width(slider_width)
                    .offset((-width * 0.5) + (slider_width * 0.5), 0.),
                {
                    let knob = self.knob_fill.resolve(area, &state);
                    let dragging = state.dragging;
                    let hovered = state.hovered;
                    circle(id!(self.id))
                        .fill(adjust_brush(&knob, dragging, hovered))
                        .finish(ctx)
                        .pad(height * 0.1)
                        .height(if state.dragging { height * 1.1 } else { height })
                        .width(height)
                        .offset((-width * 0.5) + slider_width - (height * 0.5), 0.)
                },
                rect(id!(self.id))
                    .fill(TRANSPARENT)
                    .view()
                    .on_hover({
                        let binding = self.binding.clone();
                        move |state: &mut State, _app: &mut AppState, h| {
                            binding.update(state, |s| s.hovered = h)
                        }
                    })
                    .on_drag({
                        let binding = self.binding.clone();
                        let min = self.min;
                        let max = self.max;
                        let on_change = self.on_change.clone();
                        move |state: &mut State, app: &mut AppState, drag_state| {
                            let gesture_padding = height / width;
                            let update_value = |x: f64| {
                                let padded_start = gesture_padding * width;
                                let padded_end = width - (gesture_padding * width);
                                let padded_width = padded_end - padded_start;
                                let normalized = ((x - padded_start as f64) / padded_width as f64)
                                    .clamp(0.0, 1.0)
                                    as f32;
                                min + normalized * (max - min)
                            };

                            match drag_state {
                                DragState::Began { start, .. } => {
                                    binding.update(state, |s| s.dragging = true);
                                    let new_value = update_value(start.x);
                                    binding.update(state, |s| s.value = new_value);
                                    if let Some(ref f) = on_change {
                                        f(state, app, new_value);
                                    }
                                }
                                DragState::Updated { current, .. } => {
                                    let new_value = update_value(current.x);
                                    binding.update(state, |s| s.value = new_value);
                                    if let Some(ref f) = on_change {
                                        f(state, app, new_value);
                                    }
                                }
                                DragState::Completed { .. } => {
                                    binding.update(state, |s| s.dragging = false);
                                }
                            }
                        }
                    })
                    .finish(ctx)
                    .height(height)
                    .width(width),
            ])
            .draw(area, ctx)
        })
    }
}
