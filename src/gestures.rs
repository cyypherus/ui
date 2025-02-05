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

#[derive(Debug, Clone, Copy)]
pub enum ClickState {
    Started,
    Cancelled,
    Completed,
}

pub(crate) type ClickHandler<State> = Rc<dyn Fn(&mut State, ClickState)>;
pub(crate) type DragHandler<State> = Rc<dyn Fn(&mut State, DragState)>;
pub(crate) type HoverHandler<State> = Rc<dyn Fn(&mut State, bool)>;
pub(crate) type KeyHandler<State> = Rc<dyn Fn(&mut State, Key)>;

pub struct GestureHandler<State> {
    pub on_click: Option<ClickHandler<State>>,
    pub on_drag: Option<DragHandler<State>>,
    pub on_hover: Option<HoverHandler<State>>,
    pub on_key: Option<KeyHandler<State>>,
}

impl<State> Default for GestureHandler<State> {
    fn default() -> Self {
        GestureHandler {
            on_click: None,
            on_drag: None,
            on_hover: None,
            on_key: None,
        }
    }
}

impl<State> Clone for GestureHandler<State> {
    fn clone(&self) -> Self {
        Self {
            on_click: self.on_click.clone(),
            on_drag: self.on_drag.clone(),
            on_hover: self.on_hover.clone(),
            on_key: self.on_key.clone(),
        }
    }
}
