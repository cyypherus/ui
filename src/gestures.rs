use crate::Point;

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

pub struct GestureHandler<State> {
    pub on_click: Option<Box<dyn Fn(&mut State, ClickState)>>,
    pub on_drag: Option<Box<dyn Fn(&mut State, DragState)>>,
    pub on_hover: Option<Box<dyn Fn(&mut State, bool)>>,
}
