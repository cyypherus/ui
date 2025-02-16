use backer::models::Area;

use crate::{ui::UiCx, Key, Point, RcUi};
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
    pub fn relative(&self) -> Point {
        Point {
            x: self.global.x - self.area.x as f64,
            y: self.global.y - self.area.y as f64,
        }
    }
}

pub(crate) type ClickHandler<State> = Rc<dyn Fn(&mut State, ClickState, ClickLocation)>;
pub(crate) type DragHandler<State> = Rc<dyn Fn(&mut State, DragState)>;
pub(crate) type HoverHandler<State> = Rc<dyn Fn(&mut State, bool)>;
pub(crate) type KeyHandler<State> = Rc<dyn Fn(&mut State, Key)>;
pub(crate) type SystemKeyHandler<State> = Rc<dyn Fn(&mut State, &mut UiCx, Key)>;
pub(crate) type ScrollHandler<State> = Rc<dyn Fn(&mut State, ScrollDelta)>;

pub struct ScrollDelta {
    pub x: f32,
    pub y: f32,
}

pub struct GestureHandler<State> {
    pub on_click: Option<ClickHandler<State>>,
    pub on_drag: Option<DragHandler<State>>,
    pub on_hover: Option<HoverHandler<State>>,
    pub on_key: Option<KeyHandler<State>>,
    pub on_system_key: Option<SystemKeyHandler<State>>,
    pub on_scroll: Option<ScrollHandler<State>>,
}

impl<State> Default for GestureHandler<State> {
    fn default() -> Self {
        GestureHandler {
            on_click: None,
            on_drag: None,
            on_hover: None,
            on_key: None,
            on_system_key: None,
            on_scroll: None,
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
            on_system_key: self.on_system_key.clone(),
            on_scroll: self.on_scroll.clone(),
        }
    }
}
