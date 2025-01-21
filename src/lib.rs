mod app;
mod custom_view;
mod event;
mod gestures;
mod primitives;
mod rect;
mod text;
mod ui;
mod view;

pub use app::App;
pub use backer::{
    id,
    models::*,
    nodes::*,
    transitions::{AnimationBank, TransitionDrawable, TransitionState},
    Layout, Node,
};
pub use gestures::{ClickState, DragState, GestureHandler, GestureState};
pub use rect::rect;
pub use text::text;
pub use ui::Ui;
pub use vello::peniko::Color;
pub use view::view;

use primitives::*;

const RUBIK_FONT: &[u8] = include_bytes!("../assets/Rubik-VariableFont_wght.ttf");
