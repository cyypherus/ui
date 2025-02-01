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

pub(crate) type ClickHandler<State> = Box<dyn Fn(&mut State, ClickState)>;
pub(crate) type DragHandler<State> = Box<dyn Fn(&mut State, DragState)>;
pub(crate) type HoverHandler<State> = Box<dyn Fn(&mut State, bool)>;

pub struct GestureHandler<State> {
    pub on_click: Option<ClickHandler<State>>,
    pub on_drag: Option<DragHandler<State>>,
    pub on_hover: Option<HoverHandler<State>>,
}
