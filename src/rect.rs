use crate::view::{View, ViewType};
use crate::{GestureHandler, Ui, ViewTrait};
use backer::transitions::TransitionDrawable;
use backer::{models::Area, Node};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::process::id;
use vello::kurbo::{BezPath, RoundedRect, Shape, Stroke};
use vello::peniko::{Brush, Fill};
use vello::{kurbo::Affine, peniko::Color};

#[derive(Debug, Clone)]
pub(crate) struct Rect {
    id: u64,
    fill: Option<Color>,
    radius: f32,
    stroke: Option<(Color, f32)>,
    easing: Option<backer::Easing>,
    duration: Option<f32>,
    delay: f32,
}

pub(crate) fn rect(id: String) -> Rect {
    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    Rect {
        id: hasher.finish(),
        fill: None,
        radius: 0.,
        stroke: None,
        easing: None,
        duration: None,
        delay: 0.,
    }
}

impl Rect {
    pub(crate) fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }
    pub(crate) fn corner_rounding(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }
    pub(crate) fn stroke(mut self, color: Color, line_width: f32) -> Self {
        self.stroke = Some((color, line_width));
        self
    }
    pub(crate) fn finish<State>(self) -> View<State> {
        View {
            view_type: ViewType::Rect(self),
            gesture_handler: GestureHandler {
                on_click: None,
                on_drag: None,
                on_hover: None,
            },
            easing: None,
            duration: None,
            delay: 0.,
        }
    }
}

impl<'s, State> TransitionDrawable<Ui<'s, State>> for Rect {
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
        let path = RoundedRect::from_rect(
            vello::kurbo::Rect::from_origin_size(
                vello::kurbo::Point::new(area.x as f64, area.y as f64),
                vello::kurbo::Size::new(area.width as f64, area.height as f64),
            ),
            self.radius as f64,
        )
        .to_path(0.01);
        if self.fill.is_none() && self.stroke.is_none() {
            state.scene.fill(
                Fill::EvenOdd,
                Affine::IDENTITY,
                Color::BLACK.multiply_alpha(visible_amount),
                None,
                &path,
            )
        } else {
            if let Some(fill) = self.fill {
                state.scene.fill(
                    Fill::EvenOdd,
                    Affine::IDENTITY,
                    fill.multiply_alpha(visible_amount),
                    None,
                    &path,
                )
            }
            if let Some((stroke, width)) = self.stroke {
                state.scene.stroke(
                    &Stroke::new(width as f64),
                    Affine::IDENTITY,
                    &Brush::Solid(stroke.multiply_alpha(visible_amount)),
                    None,
                    &path,
                );
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
    fn delay(&self) -> f32 {
        self.delay
    }
}

impl<'s, State> ViewTrait<'s, State> for Rect {
    fn view(self, _ui: &mut Ui<State>, node: Node<Ui<'s, State>>) -> Node<Ui<'s, State>> {
        node
    }
}
