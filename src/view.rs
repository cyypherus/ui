use crate::circle::{AnimatedCircle, Circle};
use crate::rect::{AnimatedRect, Rect};
use crate::svg::Svg;
use crate::text::{AnimatedText, Text};
use crate::ui::{AnimArea, RcUi};
use crate::{ClickState, DragState, GestureHandler, Key};
use backer::nodes::{draw_object, dynamic};
use backer::traits::Drawable;
use backer::{models::Area, Node};
use lilt::{Animated, Easing};
use std::rc::Rc;

// A simple const FNV-1a hash for our purposes
const FNV_OFFSET: u64 = 1469598103934665603;
const FNV_PRIME: u64 = 1099511628211;

pub const fn const_hash(s: &str, line: u32, col: u32) -> u64 {
    let mut hash = FNV_OFFSET;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        i += 1;
    }
    // Incorporate the line and column numbers into the hash.
    hash ^= line as u64;
    hash = hash.wrapping_mul(FNV_PRIME);
    hash ^= col as u64;
    hash = hash.wrapping_mul(FNV_PRIME);
    hash
}

/// This macro computes a compile-time ID from the file, line, and column
/// where it’s invoked, and at runtime combines it (via XOR) with another id.
#[macro_export]
macro_rules! id {
    () => {{
        const ID: u64 = $crate::const_hash(file!(), line!(), column!());
        ID
    }};
    ($other:expr) => {{
        const ID: u64 = $crate::const_hash(file!(), line!(), column!());
        ID ^ ($other)
    }};
}

// pub fn dynamic_view<'a, State: 'a>(
//     view: impl Fn(&mut State) -> View<State> + 'a,
// ) -> Node<'a, RcUi<State>> {
//     dynamic(move |ui: &mut RcUi<State>| view(&mut ui.ui.state).finish(ui))
// }

pub fn dynamic_node<'a, State: 'a>(
    view: impl Fn(&mut State) -> Node<'a, RcUi<State>> + 'a,
) -> Node<'a, RcUi<State>> {
    dynamic(move |ui: &mut RcUi<State>| view(&mut ui.ui.state))
}

// pub fn view<'a, State: 'a>(view: impl Fn() -> View<State> + 'a) -> Node<'a, RcUi<State>> {
//     dynamic(move |ui: &mut RcUi<State>| view().finish(ui))
// }

pub struct View<State> {
    pub(crate) view_type: ViewType,
    pub(crate) gesture_handler: GestureHandler<State>,
}

impl<State> Clone for View<State> {
    fn clone(&self) -> Self {
        Self {
            view_type: self.view_type.clone(),
            gesture_handler: self.gesture_handler.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ViewType {
    Text(Text),
    Rect(Rect),
    Circle(Circle),
    Svg(Svg),
}

pub(crate) trait ViewTrait<'s, State>: Sized {
    fn create_node(
        self,
        ui: &mut RcUi<State>,
        node: Node<'s, RcUi<State>>,
    ) -> Node<'s, RcUi<State>>;
}

#[derive(Debug)]
pub(crate) enum AnimatedView {
    Rect(Box<AnimatedRect>),
    Text(Box<AnimatedText>),
    Circle(Box<AnimatedCircle>),
}

impl<State> View<State> {
    pub fn on_click(mut self, f: impl Fn(&mut State, ClickState) + 'static) -> View<State> {
        self.gesture_handler.on_click = Some(Rc::new(f));
        self
    }
    pub fn on_drag(mut self, f: impl Fn(&mut State, DragState) + 'static) -> View<State> {
        self.gesture_handler.on_drag = Some(Rc::new(f));
        self
    }
    pub fn on_hover(mut self, f: impl Fn(&mut State, bool) + 'static) -> View<State> {
        self.gesture_handler.on_hover = Some(Rc::new(f));
        self
    }
    pub fn on_key(mut self, f: impl Fn(&mut State, Key) + 'static) -> View<State> {
        self.gesture_handler.on_key = Some(Rc::new(f));
        self
    }
    pub fn easing(mut self, easing: lilt::Easing) -> Self {
        match self.view_type {
            ViewType::Text(ref mut view) => view.easing = Some(easing),
            ViewType::Rect(ref mut view) => view.shape.easing = Some(easing),
            ViewType::Svg(ref mut view) => view.easing = Some(easing),
            ViewType::Circle(ref mut view) => view.shape.easing = Some(easing),
        }
        self
    }
    pub fn transition_duration(mut self, duration_ms: f32) -> Self {
        match self.view_type {
            ViewType::Text(ref mut view) => view.duration = Some(duration_ms),
            ViewType::Rect(ref mut view) => view.shape.duration = Some(duration_ms),
            ViewType::Svg(ref mut view) => view.duration = Some(duration_ms),
            ViewType::Circle(ref mut view) => view.shape.duration = Some(duration_ms),
        }
        self
    }
    pub fn transition_delay(mut self, delay_ms: f32) -> Self {
        match self.view_type {
            ViewType::Text(ref mut view) => view.delay = delay_ms,
            ViewType::Rect(ref mut view) => view.shape.delay = delay_ms,
            ViewType::Svg(ref mut view) => view.delay = delay_ms,
            ViewType::Circle(ref mut view) => view.shape.delay = delay_ms,
        }
        self
    }
    pub(crate) fn id(&self) -> u64 {
        match &self.view_type {
            ViewType::Text(view) => view.id,
            ViewType::Rect(view) => view.id,
            ViewType::Svg(view) => view.id,
            ViewType::Circle(view) => view.id,
        }
    }
    fn get_easing(&self) -> Easing {
        match &self.view_type {
            ViewType::Text(view) => view.easing,
            ViewType::Rect(view) => view.shape.easing,
            ViewType::Svg(view) => view.easing,
            ViewType::Circle(view) => view.shape.easing,
        }
        .unwrap_or(Easing::EaseOut)
    }
    fn get_duration(&self) -> f32 {
        match &self.view_type {
            ViewType::Text(view) => view.duration,
            ViewType::Rect(view) => view.shape.duration,
            ViewType::Svg(view) => view.duration,
            ViewType::Circle(view) => view.shape.duration,
        }
        .unwrap_or(200.)
    }
    fn get_delay(&self) -> f32 {
        match &self.view_type {
            ViewType::Text(view) => view.delay,
            ViewType::Rect(view) => view.shape.delay,
            ViewType::Svg(view) => view.delay,
            ViewType::Circle(view) => view.shape.delay,
        }
    }
}

impl<State> View<State> {
    pub fn finish<'a>(self) -> Node<'a, RcUi<State>>
    where
        State: 'a,
    {
        dynamic(move |ui: &mut RcUi<State>| {
            let moved = self.clone();
            match moved.view_type.clone() {
                ViewType::Text(view) => view.create_node(ui, draw_object(moved)),
                ViewType::Rect(view) => view.create_node(ui, draw_object(moved)),
                ViewType::Svg(view) => view.create_node(ui, draw_object(moved)),
                ViewType::Circle(view) => view.create_node(ui, draw_object(moved)),
            }
        })
    }
}

impl<State> Drawable<RcUi<State>> for View<State> {
    fn draw(&mut self, area: Area, state: &mut RcUi<State>, visible: bool) {
        let mut anim = state
            .ui
            .cx
            .as_mut()
            .unwrap()
            .animation_bank
            .animations
            .remove(&self.id())
            .unwrap_or(AnimArea {
                visible: Animated::new(visible)
                    .duration(self.get_duration())
                    .easing(self.get_easing())
                    .delay(self.get_delay()),
                x: Animated::new(area.x)
                    .duration(self.get_duration())
                    .easing(self.get_easing())
                    .delay(self.get_delay()),
                y: Animated::new(area.y)
                    .duration(self.get_duration())
                    .easing(self.get_easing())
                    .delay(self.get_delay()),
                width: Animated::new(area.width)
                    .duration(self.get_duration())
                    .easing(self.get_easing())
                    .delay(self.get_delay()),
                height: Animated::new(area.height)
                    .duration(self.get_duration())
                    .easing(self.get_easing())
                    .delay(self.get_delay()),
            });
        anim.visible.transition(visible, state.ui.now);
        anim.x.transition(area.x, state.ui.now);
        anim.y.transition(area.y, state.ui.now);
        anim.width.transition(area.width, state.ui.now);
        anim.height.transition(area.height, state.ui.now);
        if visible || anim.visible.in_progress(state.ui.now) {
            let visibility = anim.visible.animate_bool(0., 1., state.ui.now);
            let area = Area {
                x: anim.x.animate_wrapped(state.ui.now),
                y: anim.y.animate_wrapped(state.ui.now),
                width: anim.width.animate_wrapped(state.ui.now),
                height: anim.height.animate_wrapped(state.ui.now),
            };
            if !visible || visibility == 0. {
                return;
            }
            state.ui.gesture_handlers.push((
                self.id(),
                area,
                GestureHandler {
                    on_click: self.gesture_handler.on_click.take(),
                    on_drag: self.gesture_handler.on_drag.take(),
                    on_hover: self.gesture_handler.on_hover.take(),
                    on_key: self.gesture_handler.on_key.take(),
                },
            ));
            match &mut self.view_type {
                ViewType::Text(view) => view.draw(area, state, visible, visibility),
                ViewType::Rect(view) => view.draw(area, state, visible, visibility),
                ViewType::Svg(view) => view.draw(area, state, visible, visibility),
                ViewType::Circle(view) => view.draw(area, state, visible, visibility),
            }
        }
        state
            .ui
            .cx
            .as_mut()
            .unwrap()
            .animation_bank
            .animations
            .insert(self.id(), anim);
    }
}
