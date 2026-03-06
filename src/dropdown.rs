use crate::app::{AppCtx, View};
use crate::{Binding, ClickState, DEFAULT_CORNER_ROUNDING, app::AppState, rect};
use crate::{Color, TRANSPARENT};
use backer::{Align, Layout, nodes::*};
use std::rc::Rc;
use vello_svg::vello::kurbo::Stroke;

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

pub struct DropdownItemCtx<'a, T> {
    pub index: usize,
    pub value: &'a T,
    pub selected: bool,
    pub hovered: bool,
    pub expanded: bool,
}

pub struct DropDown<'a, State, T> {
    id: u64,
    state: DropdownState<T>,
    binding: Binding<State, DropdownState<T>>,
    options: Vec<T>,
    view_fn: Rc<dyn Fn(DropdownItemCtx<T>, &mut AppCtx) -> Layout<'a, View<State>, AppCtx> + 'a>,
    background:
        Option<Rc<dyn Fn(&DropdownState<T>, &mut AppCtx) -> Layout<'a, View<State>, AppCtx> + 'a>>,
    on_select: Option<Rc<dyn Fn(&mut State, &mut AppState, &T)>>,
}

pub fn dropdown<'a, State, T: Clone + PartialEq + 'static>(
    id: u64,
    state: (DropdownState<T>, Binding<State, DropdownState<T>>),
    options: Vec<T>,
    view_fn: impl Fn(DropdownItemCtx<T>, &mut AppCtx) -> Layout<'a, View<State>, AppCtx> + 'a,
) -> DropDown<'a, State, T> {
    DropDown {
        id,
        state: state.0,
        binding: state.1,
        options,
        view_fn: Rc::new(view_fn),
        background: None,
        on_select: None,
    }
}

impl<'a, State, T: Clone + PartialEq + 'static> DropDown<'a, State, T> {
    pub fn background(
        mut self,
        f: impl Fn(&DropdownState<T>, &mut AppCtx) -> Layout<'a, View<State>, AppCtx> + 'a,
    ) -> Self {
        self.background = Some(Rc::new(f));
        self
    }

    pub fn on_select(
        mut self,
        on_select: impl Fn(&mut State, &mut AppState, &T) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(on_select));
        self
    }

    pub fn build(self, ctx: &mut AppCtx) -> Layout<'a, View<State>, AppCtx>
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
        let background_fn = self.background;
        let dd_state = self.state.clone();

        let row = |index: usize, option: &T, ctx: &mut AppCtx| -> Layout<'a, View<State>, AppCtx> {
            let item_ctx = DropdownItemCtx {
                index,
                value: option,
                selected: selected_index == index,
                hovered: hovered == Some(index),
                expanded,
            };
            let content = (self.view_fn)(item_ctx, ctx);

            stack(vec![
                {
                    let option = option.clone();
                    rect(crate::id!(index as u64, id))
                        .fill(TRANSPARENT)
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
                }
                .inert(),
                content,
            ])
            .expand_x()
        };

        let visible: Vec<_> = if expanded {
            self.options.iter().enumerate().collect()
        } else {
            vec![(0, &self.options[selected_index])]
        };
        let rows: Vec<_> = visible
            .into_iter()
            .map(|(index, option)| row(index, option, ctx))
            .collect();

        let bg = {
            let binding = binding.clone();
            if let Some(f) = background_fn {
                f(&dd_state, ctx)
            } else {
                draw(move |area, ctx: &mut AppCtx| {
                    rect(crate::id!(id))
                        .fill(Color::from_rgb8(50, 50, 50))
                        .stroke(Color::from_rgb8(60, 60, 60), Stroke::new(1.))
                        .corner_rounding(DEFAULT_CORNER_ROUNDING)
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
            }
        };

        stack(vec![bg.inert(), column(rows).align(Align::Top)])
    }
}
