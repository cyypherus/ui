#![allow(clippy::type_complexity, clippy::too_many_arguments)]

mod animated_color;
mod app;
mod button;
mod circle;
mod draw_layout;
mod editor;
mod event;
mod gestures;
mod image;
mod models;
mod rect;
mod scroller;
mod segment_picker;
mod shape;
mod slider;
mod svg;
mod text;
mod text_field;
mod toggle;
mod ui;
mod view;

pub use app::{App, AppBuilder, AppState};
pub use backer::{Layout, Node, nodes::*};
pub use button::*;
pub use circle::circle;
pub use editor::*;
pub use gestures::{ClickState, DragState, EditInteraction, GestureHandler, GestureState};
pub use image::{image, image_from_bytes, image_from_path};
pub use rect::rect;
pub use scroller::*;
pub use segment_picker::*;
pub use slider::*;
pub use svg::svg;
pub use text::*;
pub use text_field::*;
pub use toggle::*;
// pub use ui::scoper;
pub use view::clipping;
pub use view::const_hash;
pub use winit::keyboard::NamedKey;

use lilt::Easing;
use vello_svg::vello::kurbo::*;
use vello_svg::vello::peniko::color::AlphaColor;
use vello_svg::vello::peniko::color::Srgb;

pub use models::*;

pub type Color = AlphaColor<Srgb>;

const RUBIK_FONT: &[u8] = include_bytes!("../assets/Rubik-VariableFont_wght.ttf");
const DEFAULT_FONT_FAMILY: &str = "Rubik";
const DEFAULT_EASING: Easing = Easing::EaseOut;
const DEFAULT_DURATION: f32 = 200.;
const DEFAULT_PADDING: f32 = 10.;
const DEFAULT_CORNER_ROUNDING: f32 = 5.;
const DEFAULT_FONT_SIZE: u32 = 14;
const DEFAULT_FG_COLOR: AlphaColor<Srgb> = AlphaColor::WHITE;
