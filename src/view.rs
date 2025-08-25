use crate::app::AppState;
use crate::circle::{AnimatedCircle, Circle};
use crate::gestures::{ClickLocation, Interaction, InteractionType, ScrollDelta};
use crate::image::Image;
use crate::rect::{AnimatedRect, Rect};
use crate::svg::Svg;
use crate::text::{AnimatedText, Text};
use crate::ui::AnimArea;
use crate::{ClickState, DragState, GestureHandler, Key};
use backer::nodes::{draw_object, dynamic, intermediate};
use backer::traits::Drawable;
use backer::{Node, models::Area};
use lilt::{Animated, Easing};
use std::rc::Rc;
use vello_svg::vello::kurbo::{Affine, BezPath};
use vello_svg::vello::peniko::Mix;

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
/// where it's invoked, and at runtime combines it (via XOR) with another id.
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
    ($other:expr, $other2:expr) => {{
        const ID: u64 = $crate::const_hash(file!(), line!(), column!());
        ID ^ ($other) ^ (($other2).wrapping_mul(1099511628211))
    }};
}

#[macro_export]
macro_rules! binding {
    ($State:ty, $field:ident) => {
        Binding::new(
            |s: &$State| s.$field.clone(),
            |s: &mut $State, value| s.$field = value,
        )
    };
}

pub fn clipping<'a, State: 'a>(
    path: fn(Area) -> BezPath,
    node: Node<'a, State, AppState<State>>,
) -> Node<'a, State, AppState<State>> {
    intermediate(
        move |available_area: Area, _state: &mut State, app: &mut AppState<State>| {
            app.scene
                .push_layer(Mix::Normal, 1., Affine::IDENTITY, &(path)(available_area));
        },
        move |_state: &mut State, app: &mut AppState<State>| {
            app.scene.pop_layer();
        },
        node,
    )
}

pub struct View<State> {
    pub(crate) view_type: ViewType,
    pub(crate) gesture_handlers: Vec<GestureHandler<State, AppState<State>>>,
}

impl<State> Clone for View<State> {
    fn clone(&self) -> Self {
        Self {
            view_type: self.view_type.clone(),
            gesture_handlers: self.gesture_handlers.clone(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum ViewType {
    Text(Text),
    Rect(Rect),
    Circle(Circle),
    Svg(Svg),
    Image(Image),
}

impl Clone for ViewType {
    fn clone(&self) -> Self {
        match self {
            ViewType::Text(text) => ViewType::Text(text.clone()),
            ViewType::Rect(rect) => ViewType::Rect(*rect),
            ViewType::Circle(circle) => ViewType::Circle(circle.clone()),
            ViewType::Svg(svg) => ViewType::Svg(svg.clone()),
            ViewType::Image(image) => ViewType::Image(image.clone()),
        }
    }
}

#[derive(Debug)]
pub(crate) enum AnimatedView {
    Rect(Box<AnimatedRect>),
    Text(Box<AnimatedText>),
    Circle(Box<AnimatedCircle>),
}

impl<State> View<State> {
    pub fn on_click(
        mut self,
        f: impl Fn(&mut State, &mut AppState<State>, ClickState, ClickLocation) + 'static,
    ) -> Self {
        self.gesture_handlers.push(GestureHandler {
            interaction_type: InteractionType {
                click: true,
                ..Default::default()
            },
            interaction_handler: Some(Rc::new(move |state, app_state, interaction| {
                let Interaction::Click(click, location) = interaction else {
                    return;
                };
                (f)(state, app_state, click, location);
            })),
        });
        self
    }
    pub fn on_click_outside(
        mut self,
        f: impl Fn(&mut State, &mut AppState<State>, ClickState, ClickLocation) + 'static,
    ) -> Self {
        self.gesture_handlers.push(GestureHandler {
            interaction_type: InteractionType {
                click_outside: true,
                ..Default::default()
            },
            interaction_handler: Some(Rc::new(move |state, app_state, interaction| {
                let Interaction::ClickOutside(click, location) = interaction else {
                    return;
                };
                (f)(state, app_state, click, location);
            })),
        });
        self
    }
    pub fn on_drag(
        mut self,
        f: impl Fn(&mut State, &mut AppState<State>, DragState) + 'static,
    ) -> Self {
        self.gesture_handlers.push(GestureHandler {
            interaction_type: InteractionType {
                drag: true,
                ..Default::default()
            },
            interaction_handler: Some(Rc::new(move |state, app_state, interaction| {
                let Interaction::Drag(drag) = interaction else {
                    return;
                };
                (f)(state, app_state, drag);
            })),
        });
        self
    }
    pub fn on_hover(
        mut self,
        f: impl Fn(&mut State, &mut AppState<State>, bool) + 'static,
    ) -> Self {
        self.gesture_handlers.push(GestureHandler {
            interaction_type: InteractionType {
                hover: true,
                ..Default::default()
            },
            interaction_handler: Some(Rc::new(move |state, app_state, interaction| {
                let Interaction::Hover(hovered) = interaction else {
                    return;
                };
                (f)(state, app_state, hovered);
            })),
        });
        self
    }
    pub fn on_key(mut self, f: impl Fn(&mut State, &mut AppState<State>, Key) + 'static) -> Self {
        self.gesture_handlers.push(GestureHandler {
            interaction_type: InteractionType {
                key: true,
                ..Default::default()
            },
            interaction_handler: Some(Rc::new(move |state, app_state, interaction| {
                let Interaction::Key(key) = interaction else {
                    return;
                };
                (f)(state, app_state, key);
            })),
        });
        self
    }
    pub fn on_scroll(
        mut self,
        f: impl Fn(&mut State, &mut AppState<State>, ScrollDelta) + 'static,
    ) -> Self {
        self.gesture_handlers.push(GestureHandler {
            interaction_type: InteractionType {
                scroll: true,
                ..Default::default()
            },
            interaction_handler: Some(Rc::new(move |state, app_state, interaction| {
                let Interaction::Scroll(scroll) = interaction else {
                    return;
                };
                (f)(state, app_state, scroll);
            })),
        });
        self
    }
    pub fn on_appear(mut self, f: impl Fn(&mut State, &mut AppState<State>) + 'static) -> Self {
        self.gesture_handlers.push(GestureHandler {
            interaction_type: InteractionType {
                appear: true,
                ..Default::default()
            },
            interaction_handler: Some(Rc::new(move |state, app_state, interaction| {
                let Interaction::Appear = interaction else {
                    return;
                };
                (f)(state, app_state);
            })),
        });
        self
    }
    pub fn easing(mut self, easing: lilt::Easing) -> Self {
        match self.view_type {
            ViewType::Text(ref mut view) => view.easing = Some(easing),

            ViewType::Rect(ref mut view) => view.shape.easing = Some(easing),
            ViewType::Svg(ref mut view) => view.easing = Some(easing),
            ViewType::Circle(ref mut view) => view.shape.easing = Some(easing),
            ViewType::Image(ref mut view) => view.easing = Some(easing),
        }
        self
    }
    pub fn transition_duration(mut self, duration_ms: f32) -> Self {
        match self.view_type {
            ViewType::Text(ref mut view) => view.duration = Some(duration_ms),

            ViewType::Rect(ref mut view) => view.shape.duration = Some(duration_ms),
            ViewType::Svg(ref mut view) => view.duration = Some(duration_ms),
            ViewType::Circle(ref mut view) => view.shape.duration = Some(duration_ms),
            ViewType::Image(ref mut view) => view.duration = Some(duration_ms),
        }
        self
    }
    pub fn transition_delay(mut self, delay_ms: f32) -> Self {
        match self.view_type {
            ViewType::Text(ref mut view) => view.delay = delay_ms,
            ViewType::Rect(ref mut view) => view.shape.delay = delay_ms,
            ViewType::Svg(ref mut view) => view.delay = delay_ms,
            ViewType::Circle(ref mut view) => view.shape.delay = delay_ms,
            ViewType::Image(ref mut view) => view.delay = delay_ms,
        }
        self
    }
    pub(crate) fn id(&self) -> u64 {
        match &self.view_type {
            ViewType::Text(view) => view.id,
            ViewType::Rect(view) => view.id,
            ViewType::Svg(view) => view.id,
            ViewType::Circle(view) => view.id,
            ViewType::Image(view) => view.id,
        }
    }
    fn get_easing(&self) -> Easing {
        match &self.view_type {
            ViewType::Text(view) => view.easing,
            ViewType::Rect(view) => view.shape.easing,
            ViewType::Svg(view) => view.easing,
            ViewType::Circle(view) => view.shape.easing,
            ViewType::Image(view) => view.easing,
        }
        .unwrap_or(Easing::EaseOut)
    }
    fn get_duration(&self) -> f32 {
        match &self.view_type {
            ViewType::Text(view) => view.duration,
            ViewType::Rect(view) => view.shape.duration,
            ViewType::Svg(view) => view.duration,
            ViewType::Circle(view) => view.shape.duration,
            ViewType::Image(view) => view.duration,
        }
        .unwrap_or(200.)
    }
    fn get_delay(&self) -> f32 {
        match &self.view_type {
            ViewType::Text(view) => view.delay,
            ViewType::Rect(view) => view.shape.delay,
            ViewType::Svg(view) => view.delay,
            ViewType::Circle(view) => view.shape.delay,
            ViewType::Image(view) => view.delay,
        }
    }
}

impl<State> View<State> {
    pub fn finish<'a>(self) -> Node<'a, State, AppState<State>>
    where
        State: 'static,
    {
        dynamic(move |state: &mut State, app: &mut AppState<State>| {
            let moved = self.clone();
            if let ViewType::Text(view) = self.view_type.clone() {
                view.create_node(state, app, draw_object(moved))
            } else {
                draw_object(moved)
            }
        })
    }
}

impl<State> Drawable<State, AppState<State>> for View<State> {
    fn draw(&mut self, area: Area, state: &mut State, app: &mut AppState<State>, visible: bool) {
        let mut anim = app
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
        if app.resizing {
            anim.visible.transition_instantaneous(visible, app.now);
            anim.x.transition_instantaneous(area.x, app.now);
            anim.y.transition_instantaneous(area.y, app.now);
            anim.width.transition_instantaneous(area.width, app.now);
            anim.height.transition_instantaneous(area.height, app.now);
        } else {
            anim.visible.transition(visible, app.now);
            anim.x.transition(area.x, app.now);
            anim.y.transition(area.y, app.now);
            anim.width.transition(area.width, app.now);
            anim.height.transition(area.height, app.now);
        }
        if visible || anim.visible.in_progress(app.now) {
            let visibility = anim.visible.animate_bool(0., 1., app.now);
            let animated_area = Area {
                x: anim.x.animate_wrapped(app.now),
                y: anim.y.animate_wrapped(app.now),
                width: anim.width.animate_wrapped(app.now),
                height: anim.height.animate_wrapped(app.now),
            };
            if !visible || visibility == 0. {
                return;
            }
            let id = self.id();

            // Check if this view is appearing for the first time
            if !app.appeared_views.contains(&id) {
                app.appeared_views.insert(id);
                // Trigger appear handlers
                for handler in &self.gesture_handlers {
                    if handler.interaction_type.appear
                        && let Some(ref interaction_handler) = handler.interaction_handler
                    {
                        interaction_handler(state, app, crate::gestures::Interaction::Appear);
                    }
                }
            }

            app.gesture_handlers.extend(
                self.gesture_handlers
                    .drain(..)
                    .map(|handler| (id, animated_area, handler)),
            );
            match &mut self.view_type {
                ViewType::Text(view) => {
                    view.draw(animated_area, area, state, app, visible, visibility)
                }
                ViewType::Rect(view) => view.draw(animated_area, state, app, visible, visibility),
                ViewType::Svg(view) => view.draw(animated_area, state, app, visible, visibility),
                ViewType::Circle(view) => view.draw(animated_area, state, app, visible, visibility),
                ViewType::Image(view) => view.draw(animated_area, state, app, visible, visibility),
            }
        }
        app.animation_bank.animations.insert(self.id(), anim);
    }
}
