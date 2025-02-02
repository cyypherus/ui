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
pub use backer::{models::*, nodes::*, Layout, Node};
pub use gestures::{ClickState, DragState, GestureHandler, GestureState};
use lilt::Easing;
pub use rect::rect;
pub use text::text;
pub use ui::{scoper, RcUi, Ui};
pub use vello::peniko::Color;
pub use view::const_hash;
pub use view::{dynamic_node, dynamic_view, view};

use primitives::*;

const RUBIK_FONT: &[u8] = include_bytes!("../assets/Rubik-VariableFont_wght.ttf");
const DEFAULT_EASING: Easing = Easing::EaseOut;
const DEFAULT_DURATION: f32 = 200.;
