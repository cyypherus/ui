mod animated_color;
mod app;
mod circle;
mod event;
mod gestures;
mod models;
mod rect;
mod shape;
mod svg;
mod text;
mod ui;
mod view;

pub use app::App;
pub use backer::{models::*, nodes::*, Layout, Node};
pub use circle::circle;
pub use gestures::{ClickState, DragState, GestureHandler, GestureState};
use lilt::Easing;
pub use rect::rect;
pub use svg::svg;
pub use text::{text, TextAlign};
pub use ui::{scoper, RcUi, Ui};
pub use vello_svg::vello::peniko::Color;
pub use view::const_hash;
pub use view::custom_view;
pub use view::dynamic_node;
pub use winit::keyboard::NamedKey;

pub use models::*;

const RUBIK_FONT: &[u8] = include_bytes!("../assets/Rubik-VariableFont_wght.ttf");
const DEFAULT_EASING: Easing = Easing::EaseOut;
const DEFAULT_DURATION: f32 = 200.;
