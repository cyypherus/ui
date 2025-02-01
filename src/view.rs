use crate::rect::{AnimatedRect, Rect};
use crate::text::{AnimatedText, Text};
use crate::ui::{AnimArea, RcUi};
use crate::{ClickState, DragState, GestureHandler};
use backer::nodes::{draw_object, dynamic};
use backer::traits::Drawable;
use backer::{models::Area, Node};
use lilt::{Animated, Easing};
use std::cell::{Ref, RefCell, RefMut};
use std::hash::DefaultHasher;
use std::time::Instant;

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

pub fn dynamic_view<'a, State: 'a>(
    view: impl Fn(RefMut<State>) -> View<State> + 'a,
) -> Node<'a, RcUi<State>> {
    dynamic(move |ui: &mut RcUi<State>| {
        view(RefMut::map(RefCell::borrow_mut(&ui.ui), |ui| &mut ui.state)).view(ui)
    })
}

pub fn dynamic_node<'a, State: 'a>(
    view: impl Fn(RefMut<State>) -> Node<'a, RcUi<State>> + 'a,
) -> Node<'a, RcUi<State>> {
    dynamic(move |ui: &mut RcUi<State>| {
        view(RefMut::map(RefCell::borrow_mut(&ui.ui), |ui| &mut ui.state))
    })
}

pub fn view<'a, State: 'a>(view: impl Fn() -> View<State> + 'a) -> Node<'a, RcUi<State>> {
    dynamic(move |ui: &mut RcUi<State>| view().view(ui))
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

pub(crate) trait ViewTrait<'s, State>: Sized {
    fn view(self, ui: &mut RcUi<State>, node: Node<'s, RcUi<State>>) -> Node<'s, RcUi<State>>;
}

#[derive(Debug)]
pub(crate) enum AnimatedView {
    Rect(AnimatedRect),
    Text(Box<AnimatedText>),
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
    pub fn easing(mut self, easing: lilt::Easing) -> Self {
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
    pub(crate) fn id(&self) -> u64 {
        match &self.view_type {
            ViewType::Text(view) => view.id,
            ViewType::Rect(view) => view.id,
        }
    }
    fn get_easing(&self) -> Easing {
        match &self.view_type {
            ViewType::Text(view) => view.easing,
            ViewType::Rect(view) => view.easing,
        }
        .unwrap_or(Easing::EaseOut)
    }
    fn get_duration(&self) -> f32 {
        match &self.view_type {
            ViewType::Text(view) => view.duration,
            ViewType::Rect(view) => view.duration,
        }
        .unwrap_or(200.)
    }
    fn get_delay(&self) -> f32 {
        match &self.view_type {
            ViewType::Text(view) => view.delay,
            ViewType::Rect(view) => view.delay,
        }
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

impl<State> Drawable<RcUi<State>> for View<State> {
    fn draw(&mut self, area: Area, state: &mut RcUi<State>, visible: bool) {
        let now = Instant::now();
        let mut anim = state
            .ui
            .borrow_mut()
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
        anim.visible.transition(visible, now);
        anim.x.transition(area.x, now);
        anim.y.transition(area.y, now);
        anim.width.transition(area.width, now);
        anim.height.transition(area.height, now);
        if visible || anim.visible.in_progress(now) {
            let visibility = anim.visible.animate_bool(0., 1., now);
            let area = Area {
                x: anim.x.animate_wrapped(now),
                y: anim.y.animate_wrapped(now),
                width: anim.width.animate_wrapped(now),
                height: anim.height.animate_wrapped(now),
            };
            if !visible || visibility == 0. {
                return;
            }
            RefCell::borrow_mut(&state.ui).gesture_handlers.push((
                self.id(),
                area,
                GestureHandler {
                    on_click: self.gesture_handler.on_click.take(),
                    on_drag: self.gesture_handler.on_drag.take(),
                    on_hover: self.gesture_handler.on_hover.take(),
                },
            ));
            match &mut self.view_type {
                ViewType::Text(view) => view.draw(area, state, visible, visibility),
                ViewType::Rect(view) => view.draw(area, state, visible, visibility),
            }
        }
        state
            .ui
            .borrow_mut()
            .cx
            .as_mut()
            .unwrap()
            .animation_bank
            .animations
            .insert(self.id(), anim);
    }
}
