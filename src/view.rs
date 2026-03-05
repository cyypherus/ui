use crate::app::{AppCtx, AppState, View};
use crate::gestures::{ClickLocation, Interaction, InteractionType, ScrollDelta};
use crate::image::Image;
use crate::shader::Shader;
use crate::shape::PathData;
use crate::svg::Svg;
use crate::text::Text;
use crate::{ClickState, DragState, GestureHandler, Key};
use backer::{Area, Layout, nodes::*};
use parley::Layout as TextLayout;
use std::rc::Rc;
use vello_svg::vello::kurbo::{Affine, BezPath};
use vello_svg::vello::peniko::Brush;

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
    // It would be good to explore using a TypeId for uniqueness instead of
    // the caller location. Currently we can't hash TypeId values at
    // compile time / in const contexts so the ids throughout the crate would
    // have to be changed to some Hashable struct with the unique token.
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
    ($state_var:ident, $State:ty, $field:ident) => {
        (
            $state_var.$field.clone(),
            Binding::new(
                |s: &$State| s.$field.clone(),
                |s: &mut $State, value| s.$field = value,
            ),
        )
    };
}

pub fn clipping<State: 'static>(
    path: fn(Area) -> BezPath,
    content: Layout<View<State>, AppCtx>,
) -> Layout<View<State>, AppCtx> {
    stack(vec![
        draw(move |area, _| View::PushClip { path: path(area) }),
        content,
        draw(|_, _| View::PopClip),
    ])
}

pub struct Drawable<State> {
    pub(crate) view_type: DrawableType,
    pub(crate) gesture_handlers: Vec<GestureHandler<State, AppState<State>>>,
}

impl<State> Clone for Drawable<State> {
    fn clone(&self) -> Self {
        Self {
            view_type: self.view_type.clone(),
            gesture_handlers: self.gesture_handlers.clone(),
        }
    }
}

pub(crate) enum DrawableType {
    Text(Text),
    Layout(Box<(TextLayout<Brush>, Affine)>),
    Path(Box<PathData>),
    Svg(Svg),
    Image(Image),
    Shader(Shader),
}

impl Clone for DrawableType {
    fn clone(&self) -> Self {
        match self {
            DrawableType::Text(text) => DrawableType::Text(text.clone()),
            DrawableType::Layout(boxed) => DrawableType::Layout(boxed.clone()),
            DrawableType::Path(path) => DrawableType::Path(path.clone()),
            DrawableType::Svg(svg) => DrawableType::Svg(svg.clone()),
            DrawableType::Image(image) => DrawableType::Image(image.clone()),
            DrawableType::Shader(shader) => DrawableType::Shader(shader.clone()),
        }
    }
}

impl<State> Drawable<State> {
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
    pub(crate) fn id(&self) -> u64 {
        match &self.view_type {
            DrawableType::Text(view) => view.id,
            DrawableType::Layout(_) => 0,
            DrawableType::Path(view) => view.id,
            DrawableType::Svg(view) => view.id,
            DrawableType::Image(view) => view.id,
            DrawableType::Shader(view) => view.id,
        }
    }
}

impl<State> Drawable<State> {
    pub fn finish(self, ctx: &mut AppCtx) -> Layout<View<State>, AppCtx>
    where
        State: 'static,
    {
        let view_type = self.view_type.clone();

        let node = draw(move |area, _| View::Draw {
            view: Box::new(self.clone()),
            area,
        });

        if let DrawableType::Text(text_view) = view_type {
            text_view.with_text_constraints(ctx, node)
        } else {
            node
        }
    }
}
