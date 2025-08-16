use crate::{
    Binding, ClickState, DEFAULT_CORNER_ROUNDING, DEFAULT_FONT_SIZE, DEFAULT_PURP, app::AppState,
    rect, text,
};
use crate::{Color, DEFAULT_FG};
use backer::{
    Node,
    nodes::{column, dynamic, row, stack},
};
use vello_svg::vello::peniko::color::palette::css::TRANSPARENT;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl Default for Orientation {
    fn default() -> Self {
        Self::Horizontal
    }
}

#[derive(Debug, Clone)]
pub struct SegmentPickerState<T> {
    pub selected: T,
    pub hovered: Option<usize>,
}

impl<T> SegmentPickerState<T> {
    pub fn new(selected: T) -> Self {
        Self {
            selected,
            hovered: None,
        }
    }
}

pub struct SegmentPicker<State, T> {
    id: u64,
    options: Vec<T>,
    labels: Vec<String>,
    orientation: Orientation,
    corner_rounding: Option<f32>,
    on_select: Option<fn(&mut State, &mut AppState<State>, T)>,
    state: Binding<State, SegmentPickerState<T>>,
    fill: Option<Color>,
    selected_fill: Option<Color>,
    text_fill: Option<Color>,
    selected_text_fill: Option<Color>,
}

pub fn segment_picker<State, T>(
    id: u64,
    options: Vec<T>,
    binding: Binding<State, SegmentPickerState<T>>,
) -> SegmentPicker<State, T>
where
    T: Clone + std::fmt::Display,
{
    let labels = options.iter().map(|opt| opt.to_string()).collect();
    SegmentPicker {
        id,
        options,
        labels,
        orientation: Orientation::default(),
        corner_rounding: None,
        on_select: None,
        state: binding,
        fill: None,
        selected_fill: None,
        text_fill: None,
        selected_text_fill: None,
    }
}

pub fn segment_picker_with_labels<State, T>(
    id: u64,
    options: Vec<T>,
    labels: Vec<String>,
    binding: Binding<State, SegmentPickerState<T>>,
) -> SegmentPicker<State, T>
where
    T: Clone,
{
    SegmentPicker {
        id,
        options,
        labels,
        orientation: Orientation::default(),
        corner_rounding: None,
        on_select: None,
        state: binding,
        fill: None,
        selected_fill: None,
        text_fill: None,
        selected_text_fill: None,
    }
}

impl<State, T> SegmentPicker<State, T>
where
    T: Clone + PartialEq,
{
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn horizontal(mut self) -> Self {
        self.orientation = Orientation::Horizontal;
        self
    }

    pub fn vertical(mut self) -> Self {
        self.orientation = Orientation::Vertical;
        self
    }

    pub fn corner_rounding(mut self, corner_rounding: f32) -> Self {
        self.corner_rounding = Some(corner_rounding);
        self
    }

    pub fn on_select(mut self, on_select: fn(&mut State, &mut AppState<State>, T)) -> Self {
        self.on_select = Some(on_select);
        self
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    pub fn selected_fill(mut self, color: Color) -> Self {
        self.selected_fill = Some(color);
        self
    }

    pub fn text_fill(mut self, color: Color) -> Self {
        self.text_fill = Some(color);
        self
    }

    pub fn selected_text_fill(mut self, color: Color) -> Self {
        self.selected_text_fill = Some(color);
        self
    }

    pub fn finish<'n>(self) -> Node<'n, State, AppState<State>>
    where
        State: 'static,
        T: 'static,
    {
        let corner_rounding = self.corner_rounding.unwrap_or(DEFAULT_CORNER_ROUNDING);

        dynamic(move |state: &mut State, _app: &mut AppState<State>| {
            let picker_state = self.state.get(state);
            let selected_index = self
                .options
                .iter()
                .position(|opt| *opt == picker_state.selected)
                .unwrap_or(0);

            let create_segment = |index: usize, option: &T| {
                let is_selected = selected_index == index;
                let is_hovered = picker_state.hovered == Some(index);

                stack(vec![
                    rect(crate::id!(self.id, index as u64))
                        .fill(if is_hovered {
                            Color::from_rgba8(100, 100, 100, 255)
                        } else {
                            TRANSPARENT
                        })
                        .corner_rounding(corner_rounding - 2.0)
                        .view()
                        .transition_duration(0.)
                        .finish()
                        .pad(2.0),
                    rect(crate::id!(self.id, index as u64))
                        .fill(if is_selected {
                            self.selected_fill.unwrap_or(DEFAULT_PURP)
                        } else {
                            TRANSPARENT
                        })
                        .corner_rounding(corner_rounding - 2.0)
                        .view()
                        .finish()
                        .pad(2.0),
                    text(
                        crate::id!(self.id, index as u64),
                        self.labels.get(index).cloned().unwrap_or_default(),
                    )
                    .fill(if is_selected {
                        self.selected_text_fill.unwrap_or(DEFAULT_FG)
                    } else {
                        self.text_fill.unwrap_or(DEFAULT_FG)
                    })
                    .font_size(DEFAULT_FONT_SIZE)
                    .view()
                    .finish(),
                    rect(crate::id!(self.id, index as u64))
                        .fill(TRANSPARENT)
                        .view()
                        .on_hover({
                            let binding = self.state.clone();
                            move |state, _app: &mut AppState<State>, h| {
                                binding.update(state, |s| {
                                    if h {
                                        s.hovered = Some(index)
                                    }
                                })
                            }
                        })
                        .on_click({
                            let binding = self.state.clone();
                            let option = option.clone();
                            move |state: &mut State, app: &mut AppState<State>, click_state, _| {
                                if let ClickState::Completed = click_state {
                                    if let Some(f) = self.on_select {
                                        f(state, app, option.clone());
                                    }
                                    binding.update(state, |s| s.selected = option.clone());
                                }
                            }
                        })
                        .finish(),
                ])
            };

            stack(vec![
                rect(crate::id!(self.id))
                    .fill(self.fill.unwrap_or(Color::from_rgba8(60, 60, 60, 255)))
                    .corner_rounding(corner_rounding)
                    .view()
                    .on_hover({
                        let binding = self.state.clone();
                        move |state, _app: &mut AppState<State>, h| {
                            binding.update(state, |s| {
                                if !h {
                                    s.hovered = None
                                }
                            })
                        }
                    })
                    .finish(),
                match self.orientation {
                    Orientation::Horizontal => row(self
                        .options
                        .iter()
                        .enumerate()
                        .map(|(index, option)| create_segment(index, option))
                        .collect()),
                    Orientation::Vertical => column(
                        self.options
                            .iter()
                            .enumerate()
                            .map(|(index, option)| create_segment(index, option))
                            .collect(),
                    ),
                },
            ])
        })
    }
}
