use crate::rect::{AnimatedRect, Rect};
use crate::text::Text;
use crate::ui::RcUi;
use crate::{ClickState, DragState, GestureHandler};
use backer::nodes::{draw_object, dynamic};
use backer::transitions::TransitionDrawable;
use backer::{models::Area, Node};
use std::cell::RefCell;

pub fn view<'a, State: 'a>(
    view: impl Fn(&mut RcUi<State>) -> View<State> + 'a,
) -> Node<'a, RcUi<State>> {
    dynamic(move |ui| view(ui).view(ui))
}

pub struct View<State> {
    pub(crate) view_type: ViewType,
    pub(crate) gesture_handler: GestureHandler<State>,
}

#[derive(Debug, Clone)]
pub(crate) enum ViewType {
    Text(Text),
    Rect(Rect),
}

pub(crate) trait ViewTrait<'s, State>: TransitionDrawable<RcUi<State>> + Sized {
    fn view(self, ui: &mut RcUi<State>, node: Node<'s, RcUi<State>>) -> Node<'s, RcUi<State>>;
}

#[derive(Debug)]
pub(crate) enum AnimatedView {
    Rect(AnimatedRect),
}

impl<State> View<State> {
    pub fn on_click(mut self, f: impl Fn(&mut State, ClickState) + 'static) -> View<State> {
        self.gesture_handler.on_click = Some(Box::new(f));
        self
    }
    pub fn on_drag(mut self, f: impl Fn(&mut State, DragState) + 'static) -> View<State> {
        self.gesture_handler.on_drag = Some(Box::new(f));
        self
    }
    pub fn on_hover(mut self, f: impl Fn(&mut State, bool) + 'static) -> View<State> {
        self.gesture_handler.on_hover = Some(Box::new(f));
        self
    }
    pub fn easing(mut self, easing: backer::Easing) -> Self {
        match self.view_type {
            ViewType::Text(ref mut view) => view.easing = Some(easing),
            ViewType::Rect(ref mut view) => view.easing = Some(easing),
        }
        self
    }
    pub fn transition_duration(mut self, duration_ms: f32) -> Self {
        match self.view_type {
            ViewType::Text(ref mut view) => view.duration = Some(duration_ms),
            ViewType::Rect(ref mut view) => view.duration = Some(duration_ms),
        }
        self
    }
    pub fn transition_delay(mut self, delay_ms: f32) -> Self {
        match self.view_type {
            ViewType::Text(ref mut view) => view.delay = delay_ms,
            ViewType::Rect(ref mut view) => view.delay = delay_ms,
        }
        self
    }
}

impl<State> View<State> {
    fn view<'a>(self, ui: &mut RcUi<State>) -> Node<'a, RcUi<State>>
    where
        State: 'a,
    {
        match self.view_type.clone() {
            ViewType::Text(view) => view.view(ui, draw_object(self)),
            ViewType::Rect(view) => view.view(ui, draw_object(self)),
        }
    }
}

impl<State> TransitionDrawable<RcUi<State>> for View<State> {
    fn draw_interpolated(
        &mut self,
        area: Area,
        state: &mut RcUi<State>,
        visible: bool,
        visible_amount: f32,
    ) {
        if !visible && visible_amount == 0. {
            return;
        }
        RefCell::borrow_mut(&state.ui).gesture_handlers.push((
            *self.id(),
            area,
            GestureHandler {
                on_click: self.gesture_handler.on_click.take(),
                on_drag: self.gesture_handler.on_drag.take(),
                on_hover: self.gesture_handler.on_hover.take(),
            },
        ));
        match &mut self.view_type {
            ViewType::Text(view) => view.draw_interpolated(area, state, visible, visible_amount),
            ViewType::Rect(view) => view.draw_interpolated(area, state, visible, visible_amount),
        }
    }

    fn id(&self) -> &u64 {
        match &self.view_type {
            ViewType::Text(view) => <Text as TransitionDrawable<RcUi<State>>>::id(view),
            ViewType::Rect(view) => <Rect as TransitionDrawable<RcUi<State>>>::id(view),
        }
    }

    fn easing(&self) -> backer::Easing {
        match &self.view_type {
            ViewType::Text(view) => view.easing,
            ViewType::Rect(view) => view.easing,
        }
        .unwrap_or(backer::Easing::EaseOut)
    }

    fn duration(&self) -> f32 {
        match &self.view_type {
            ViewType::Text(view) => view.duration,
            ViewType::Rect(view) => view.duration,
        }
        .unwrap_or(200.)
    }
    fn delay(&self) -> f32 {
        match &self.view_type {
            ViewType::Text(view) => view.delay,
            ViewType::Rect(view) => view.delay,
        }
    }
    fn constraints(
        &self,
        available_area: Area,
        state: &mut RcUi<State>,
    ) -> Option<backer::SizeConstraints> {
        match &self.view_type {
            ViewType::Text(view) => view.constraints(available_area, state),
            ViewType::Rect(view) => view.constraints(available_area, state),
        }
    }
}
