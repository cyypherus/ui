use crate::app::AppState;
use crate::circle::{AnimatedCircle, Circle};
use crate::gestures::{ClickLocation, EditInteraction, Interaction, InteractionType, ScrollDelta};
use crate::rect::{AnimatedRect, Rect};
use crate::svg::Svg;
use crate::text::{AnimatedText, Text};
use crate::ui::AnimArea;
use crate::{ClickState, DragState, GestureHandler, Key};
use backer::nodes::{draw_object, dynamic, intermediate};
use backer::traits::Drawable;
use backer::{models::Area, Node};
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

#[macro_export]
macro_rules! binding {
    ($State:ty, $field:ident) => {
        Binding::new(
            |s: &AppState<$State>| s.state.$field.clone(),
            |s: &mut AppState<$State>, value| s.state.$field = value,
        )
    };
}

pub fn clipping<'a, State: 'a>(
    path: fn(Area) -> BezPath,
    node: Node<'a, State, AppState>,
) -> Node<'a, State, AppState> {
    intermediate(
        move |available_area: Area, state: &mut State, app: &mut AppState| {
            app.scene
                .push_layer(Mix::Normal, 1., Affine::IDENTITY, &(path)(available_area));
        },
        move |state: &mut State, app: &mut AppState| {
            app.scene.pop_layer();
        },
        node,
    )
}

pub struct View<State> {
    pub(crate) view_type: ViewType<State>,
    pub(crate) gesture_handlers: Vec<GestureHandler<State, AppState>>,
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
pub(crate) enum ViewType<State> {
    Text(Text<State>),
    Rect(Rect),
    Circle(Circle),
    Svg(Svg),
}

impl<State> Clone for ViewType<State> {
    fn clone(&self) -> Self {
        match self {
            ViewType::Text(text) => ViewType::Text(text.clone()),
            ViewType::Rect(rect) => ViewType::Rect(*rect),
            ViewType::Circle(circle) => ViewType::Circle(circle.clone()),
            ViewType::Svg(svg) => ViewType::Svg(svg.clone()),
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
    pub fn on_edit(
        mut self,
        f: impl Fn(&mut State, &mut AppState, EditInteraction) + 'static,
    ) -> Self {
        self.gesture_handlers.push(GestureHandler {
            interaction_type: InteractionType {
                edit: true,
                ..Default::default()
            },
            interaction_handler: Some(Rc::new(move |state, app_state, interaction| {
                let Interaction::Edit(edit_interaction) = interaction else {
                    return;
                };
                (f)(state, app_state, edit_interaction);
            })),
        });
        self
    }
    pub fn on_click(
        mut self,
        f: impl Fn(&mut State, &mut AppState, ClickState, ClickLocation) + 'static,
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
    pub fn on_drag(mut self, f: impl Fn(&mut State, &mut AppState, DragState) + 'static) -> Self {
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
    pub fn on_hover(mut self, f: impl Fn(&mut State, &mut AppState, bool) + 'static) -> Self {
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
    pub fn on_key(mut self, f: impl Fn(&mut State, &mut AppState, Key) + 'static) -> Self {
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
        f: impl Fn(&mut State, &mut AppState, ScrollDelta) + 'static,
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
    pub fn finish<'a>(self) -> Node<'a, State, AppState>
    where
        State: 'static,
    {
        dynamic(move |state: &mut State, app: &mut AppState| {
            let moved = self.clone();
            if let ViewType::Text(view) = self.view_type.clone() {
                view.create_node(state, app, draw_object(moved))
            } else {
                draw_object(moved)
            }
        })
    }
}

impl<State> Drawable<State, AppState> for View<State> {
    fn draw(&mut self, area: Area, state: &mut State, app: &mut AppState, visible: bool) {
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
        anim.visible.transition(visible, app.now);
        anim.x.transition(area.x, app.now);
        anim.y.transition(area.y, app.now);
        anim.width.transition(area.width, app.now);
        anim.height.transition(area.height, app.now);
        if visible || anim.visible.in_progress(app.now) {
            let visibility = anim.visible.animate_bool(0., 1., app.now);
            let area = Area {
                x: anim.x.animate_wrapped(app.now),
                y: anim.y.animate_wrapped(app.now),
                width: anim.width.animate_wrapped(app.now),
                height: anim.height.animate_wrapped(app.now),
            };
            if !visible || visibility == 0. {
                return;
            }
            let id = self.id();
            app.gesture_handlers.extend(
                self.gesture_handlers
                    .drain(..)
                    .map(|handler| (id, area, handler)),
            );
            match &mut self.view_type {
                ViewType::Text(view) => view.draw(area, state, app, visible, visibility),
                ViewType::Rect(view) => view.draw(area, app, visible, visibility),
                ViewType::Svg(view) => view.draw(area, state, app, visible, visibility),
                ViewType::Circle(view) => view.draw(area, app, visible, visibility),
            }
        }
        app.animation_bank.animations.insert(self.id(), anim);
    }
}
