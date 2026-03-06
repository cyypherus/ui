use std::rc::Rc;

use crate::app::{AppCtx, View};
use crate::background_style::BrushSource;
use crate::{Binding, ClickState, DEFAULT_CORNER_ROUNDING, DEFAULT_PURP, app::AppState, rect};
use crate::{Color, DEFAULT_DARK_GRAY, DEFAULT_FG, DEFAULT_PADDING, TRANSPARENT, svg};
use backer::{Align, Layout, nodes::*};
use vello_svg::vello::kurbo::Stroke;
use vello_svg::vello::peniko::Brush;

#[derive(Debug, Clone)]
pub struct DropdownState<T> {
    pub selected: T,
    pub hovered: Option<usize>,
    pub expanded: bool,
}

impl<T: Default> Default for DropdownState<T> {
    fn default() -> Self {
        Self {
            selected: T::default(),
            hovered: None,
            expanded: false,
        }
    }
}

pub struct DropDown<State, T> {
    id: u64,
    state: DropdownState<T>,
    binding: Binding<State, DropdownState<T>>,
    background_corner_rounding: f32,
    background_fill: BrushSource<DropdownState<T>>,
    background_stroke: (BrushSource<DropdownState<T>>, Stroke),
    arrow_fill: Brush,
    highlight_fill: Brush,
    background_padding: f32,
    options: Vec<T>,
    view_fn: Rc<dyn Fn(usize, &T, &mut AppCtx) -> Layout<'static, View<State>, AppCtx>>,
    on_select: Option<Rc<dyn Fn(&mut State, &mut AppState, &T)>>,
}

pub fn dropdown<State, T: Clone + PartialEq + 'static>(
    id: u64,
    state: (DropdownState<T>, Binding<State, DropdownState<T>>),
    options: Vec<T>,
    view_fn: impl Fn(usize, &T, &mut AppCtx) -> Layout<'static, View<State>, AppCtx> + 'static,
) -> DropDown<State, T> {
    DropDown {
        id,
        state: state.0,
        binding: state.1,
        background_corner_rounding: DEFAULT_CORNER_ROUNDING,
        background_fill: Color::from_rgb8(50, 50, 50).into(),
        background_stroke: (Color::from_rgb8(60, 60, 60).into(), Stroke::new(1.)),
        arrow_fill: Brush::Solid(DEFAULT_FG),
        highlight_fill: Brush::Solid(DEFAULT_PURP),
        background_padding: DEFAULT_PADDING,
        options,
        view_fn: Rc::new(view_fn),
        on_select: None,
    }
}

impl<State, T: Clone + PartialEq + 'static> DropDown<State, T> {
    pub fn background_corner_rounding(mut self, corner_rounding: f32) -> Self {
        self.background_corner_rounding = corner_rounding;
        self
    }
    pub fn background_fill(mut self, fill: impl Into<BrushSource<DropdownState<T>>>) -> Self {
        self.background_fill = fill.into();
        self
    }
    pub fn background_stroke(
        mut self,
        brush: impl Into<BrushSource<DropdownState<T>>>,
        style: Stroke,
    ) -> Self {
        self.background_stroke = (brush.into(), style);
        self
    }
    pub fn arrow_fill(mut self, fill: impl Into<Brush>) -> Self {
        self.arrow_fill = fill.into();
        self
    }
    pub fn highlight_fill(mut self, fill: impl Into<Brush>) -> Self {
        self.highlight_fill = fill.into();
        self
    }
    pub fn background_padding(mut self, padding: f32) -> Self {
        self.background_padding = padding;
        self
    }
    pub fn on_select(
        mut self,
        on_select: impl Fn(&mut State, &mut AppState, &T) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(on_select));
        self
    }

    pub fn build(self, ctx: &mut AppCtx) -> Layout<'static, View<State>, AppCtx>
    where
        State: 'static,
    {
        let expanded = self.state.expanded;
        let selected_index = self
            .options
            .iter()
            .position(|o| *o == self.state.selected)
            .unwrap_or(0);
        let hovered = self.state.hovered;
        let id = self.id;
        let binding = self.binding.clone();
        let on_select = self.on_select.clone();
        let fill = self.background_fill;
        let stroke = self.background_stroke;
        let corner_rounding = self.background_corner_rounding;
        let arrow_fill = self.arrow_fill;
        let highlight_fill = self.highlight_fill;
        dbg!(&highlight_fill);

        let arrow_svg = if expanded {
            include_str!("../assets/arrow-down.svg")
        } else {
            include_str!("../assets/arrow-right.svg")
        };

        let row =
            |index: usize, option: &T, ctx: &mut AppCtx| -> Layout<'static, View<State>, AppCtx> {
                let row_fill: Brush = if expanded && selected_index == index {
                    highlight_fill.clone()
                } else if let Some(hovered) = hovered
                    && hovered == index
                {
                    DEFAULT_DARK_GRAY.into()
                } else {
                    TRANSPARENT.into()
                };

                let content = (self.view_fn)(index, option, ctx);

                stack(vec![
                    {
                        let option = option.clone();
                        rect(crate::id!(index as u64, id))
                            .fill(row_fill)
                            .corner_rounding(corner_rounding)
                            .view()
                            .on_click({
                                let binding = binding.clone();
                                let on_select = on_select.clone();
                                move |state: &mut State, app, click, _pos| {
                                    let ClickState::Completed = click else { return };
                                    if expanded {
                                        if let Some(ref on_select) = on_select {
                                            on_select(state, app, &option);
                                        }
                                        binding.update(state, {
                                            let option = option.clone();
                                            move |s| {
                                                s.selected = option.clone();
                                                s.expanded = false;
                                            }
                                        });
                                    } else {
                                        binding.update(state, |s| s.expanded = true);
                                    }
                                }
                            })
                            .on_hover({
                                let binding = binding.clone();
                                move |state: &mut State, _app, hovered| {
                                    binding.update(state, move |s| {
                                        if expanded && hovered {
                                            s.hovered = Some(index)
                                        }
                                    });
                                }
                            })
                            .finish(ctx)
                            .expand_x()
                    }
                    .inert(),
                    row_spaced(
                        5.,
                        vec![
                            if index == 0 {
                                svg(crate::id!(id), arrow_svg)
                                    .fill(arrow_fill.clone())
                                    .finish(ctx)
                                    .width(10.)
                                    .height(10.)
                            } else {
                                empty()
                            },
                            content,
                        ],
                    )
                    .expand_x()
                    .pad(self.background_padding),
                ])
            };

        let dd_state = self.state.clone();
        let visible: Vec<_> = if expanded {
            self.options.iter().enumerate().collect()
        } else {
            vec![(0, &self.options[selected_index])]
        };
        let rows: Vec<_> = visible
            .into_iter()
            .map(|(index, option)| row(index, option, ctx))
            .collect();

        stack(vec![
            draw(move |area, ctx: &mut AppCtx| {
                rect(crate::id!(id))
                    .fill(fill.resolve(area, &dd_state))
                    .stroke(stroke.0.resolve(area, &dd_state), stroke.1.clone())
                    .corner_rounding(corner_rounding)
                    .view()
                    .on_click_outside({
                        let binding = binding.clone();
                        move |state: &mut State, _app, click, _pos| {
                            let ClickState::Completed = click else { return };
                            binding.update(state, |s| s.expanded = false);
                        }
                    })
                    .on_hover({
                        let binding = binding.clone();
                        move |state: &mut State, _app, hovered| {
                            binding.update(state, move |s| {
                                if !hovered {
                                    s.hovered = None
                                }
                            });
                        }
                    })
                    .finish(ctx)
                    .draw(area, ctx)
            })
            .inert(),
            column(rows).align(Align::Top),
        ])
    }
}
