use backer::models::Area;

use crate::{Key, Point};
use std::rc::Rc;

#[derive(Debug, Clone, Copy)]
pub enum GestureState {
    None,
    Dragging { start: Point, capturer: u64 },
}

#[derive(Debug, Clone, Copy)]
pub enum DragState {
    Began(Point),
    Updated {
        start: Point,
        current: Point,
        distance: f32,
    },
    Completed {
        start: Point,
        current: Point,
        distance: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickState {
    Started,
    Cancelled,
    Completed,
}

#[derive(Debug, Clone, Copy)]
pub struct ClickLocation {
    global: Point,
    area: Area,
}

impl ClickLocation {
    pub(crate) fn new(global: Point, area: Area) -> Self {
        ClickLocation { global, area }
    }
    pub fn global(&self) -> Point {
        self.global
    }
    pub fn local(&self) -> Point {
        Point {
            x: self.global.x - self.area.x as f64,
            y: self.global.y - self.area.y as f64,
        }
    }
}

pub(crate) enum Interaction {
    Edit(EditInteraction),
    Click(ClickState, ClickLocation),
    Drag(DragState),
    Hover(bool),
    Key(Key),
    Scroll(ScrollDelta),
    Appear,
}

#[derive(Debug, Clone)]
pub enum EditInteraction {
    Update(String),
    End,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct InteractionType {
    pub(crate) edit: bool,
    pub(crate) click: bool,
    pub(crate) drag: bool,
    pub(crate) hover: bool,
    pub(crate) key: bool,
    pub(crate) scroll: bool,
    pub(crate) appear: bool,
}

impl Default for InteractionType {
    fn default() -> Self {
        Self {
            edit: false,
            click: false,
            drag: false,
            hover: false,
            key: false,
            scroll: false,
            appear: false,
        }
    }
}

pub struct ScrollDelta {
    pub x: f32,
    pub y: f32,
}
pub(crate) type InteractionHandler<T, U> = Rc<dyn Fn(&mut T, &mut U, Interaction)>;
pub struct GestureHandler<T, U> {
    pub(crate) interaction_type: InteractionType,
    pub(crate) interaction_handler: Option<InteractionHandler<T, U>>,
}

impl<T, U> Default for GestureHandler<T, U> {
    fn default() -> Self {
        GestureHandler {
            interaction_type: InteractionType::default(),
            interaction_handler: None,
        }
    }
}

impl<T, U> Clone for GestureHandler<T, U> {
    fn clone(&self) -> Self {
        Self {
            interaction_type: self.interaction_type,
            interaction_handler: self.interaction_handler.clone(),
        }
    }
}
