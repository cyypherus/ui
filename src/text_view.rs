use backer::{models::*, nodes::*, transitions::TransitionDrawable, Node};
use femtovg::{Color, FontId, Paint};
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{Ui, View};

pub(crate) fn text(id: String, text: impl AsRef<str> + 'static) -> Text {
    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    Text {
        id: hasher.finish(),
        text: text.as_ref().to_owned(),
        font_size: 16.,
        font: None,
        easing: None,
        duration: None,
    }
}

struct Text {
    id: u64,
    text: String,
    font_size: f32,
    font: Option<FontId>,
    easing: Option<backer::Easing>,
    duration: Option<f32>,
}

impl<State> TransitionDrawable<Ui<State>> for Text {
    fn draw_interpolated(
        &mut self,
        area: Area,
        state: &mut Ui<State>,
        visible: bool,
        visible_amount: f32,
    ) {
        if !visible && visible_amount == 0. {
            return;
        }
        let mut color = Color::black();
        color.set_alphaf(visible_amount);
        let paint = Paint::color(Color::black())
            .with_font(&[state.default_font])
            .with_font_size(self.font_size);

        let font_metrics = state
            .canvas
            .measure_font(&paint)
            .expect("Error measuring font");

        let width = state.canvas.width() as f32;
        let mut y = area.y + area.height;

        let lines = state
            .canvas
            .break_text_vec(width, self.text.clone(), &paint)
            .expect("Error while breaking text");

        for line_range in lines {
            if let Ok(_res) = state
                .canvas
                .fill_text(area.x, y, &self.text[line_range], &paint)
            {
                y += font_metrics.height();
            }
        }
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

impl<State> View<State> for Text {
    fn view(self, ui: &mut Ui<State>) -> Node<Ui<State>> {
        let font_size = self.font_size;
        let paint = Paint::color(Color::black())
            .with_font(&[ui.default_font])
            .with_font_size(font_size);
        let text_size = ui
            .canvas
            .measure_text(0., 0., self.text.clone(), &paint)
            .expect("Error measuring font");

        draw_object(self)
            .height(text_size.height())
            .width(text_size.width())
    }
}
