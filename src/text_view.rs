use backer::{models::*, transitions::TransitionDrawable, Node};
use nannou::{
    color::{rgb, rgba, Rgba},
    lyon::math::rect,
    text::font,
};
// use femtovg::{Rgba, FontId, Paint};
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{ui_to_draw, GestureHandler, Ui, View, ViewTrait, ViewType};

pub(crate) fn text(id: String, text: impl AsRef<str> + 'static) -> Text {
    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    Text {
        id: hasher.finish(),
        text: text.as_ref().to_owned(),
        font_size: 40,
        font: None,
        easing: None,
        duration: None,
        fill: None,
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Text {
    id: u64,
    text: String,
    fill: Option<Rgba<f32>>,
    font_size: u32,
    font: Option<font::Id>,
    easing: Option<backer::Easing>,
    duration: Option<f32>,
}

impl Text {
    pub(crate) fn fill(mut self, color: Rgba) -> Self {
        self.fill = Some(color);
        self
    }
    pub(crate) fn font_size(mut self, size: u32) -> Self {
        self.font_size = size;
        self
    }
    pub(crate) fn font(mut self, font_id: font::Id) -> Self {
        self.font = Some(font_id);
        self
    }
    pub(crate) fn finish<State>(self) -> View<State> {
        View {
            view_type: ViewType::Text(self),
            gesture_handler: GestureHandler {
                on_click: None,
                on_drag: None,
                on_hover: None,
            },
        }
    }
}

impl<State> TransitionDrawable<Ui<State>> for Text {
    fn draw_interpolated(
        &mut self,
        area: Area,
        state: &mut Ui<State>,
        visible: bool,
        visible_amount: f32,
    ) {
        let area = ui_to_draw(area, state.window_size);
        let fill = self.fill.unwrap_or(rgba(0., 0., 0., 1.));
        state
            .draw
            .text(&self.text)
            .x(area.x)
            .y(area.y)
            .h(area.height)
            .w(area.width)
            .font_size(self.font_size)
            .color(rgba(
                fill.red,
                fill.green,
                fill.blue,
                visible_amount * fill.alpha,
            ))
            .align_text_bottom()
            .finish()
    }

    fn id(&self) -> &u64 {
        &self.id
    }

    fn easing(&self) -> backer::Easing {
        self.easing.unwrap_or(backer::Easing::EaseOut)
    }
    fn duration(&self) -> f32 {
        self.duration.unwrap_or(200.)
    }
}

impl<State> ViewTrait<State> for Text {
    fn view(self, ui: &mut Ui<State>, node: Node<Ui<State>>) -> Node<Ui<State>> {
        let layout = nannou::text::Builder::from(self.text)
            .font_size(self.font_size)
            .build(nannou::geom::Rect {
                x: nannou::geom::Range {
                    start: 0.,
                    end: 400.,
                },
                y: nannou::geom::Range {
                    start: 0.,
                    end: 400.,
                },
            })
            .bounding_rect();

        node.height(layout.h()).width(layout.w())
    }
}
