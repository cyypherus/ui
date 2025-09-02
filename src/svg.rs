use crate::app::AppState;
use crate::view::{View, ViewType};
use backer::Node;
use backer::models::Area;
use lilt::Easing;
use vello_svg::vello::kurbo::{self, Affine, Vec2};
use vello_svg::vello::peniko::{Color, Compose, Fill, Mix};
use vello_svg::vello::{Scene, peniko};

#[derive(Debug, Clone)]
pub struct Svg {
    pub(crate) id: u64,
    pub(crate) content: String,
    pub(crate) unlocked_aspect_ratio: bool,
    pub(crate) fill: Option<Color>,
    pub(crate) easing: Option<Easing>,
    pub(crate) duration: Option<f32>,
    pub(crate) delay: f32,
}

pub fn svg(id: u64, content: impl AsRef<str>) -> Svg {
    Svg {
        id,
        content: content.as_ref().to_string(),
        easing: None,
        duration: None,
        delay: 0.,
        unlocked_aspect_ratio: false,
        fill: None,
    }
}

impl Svg {
    pub fn unlock_aspect_ratio(mut self) -> Self {
        self.unlocked_aspect_ratio = true;
        self
    }
    pub fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }
    pub fn view<State>(self) -> View<State> {
        View {
            view_type: ViewType::Svg(self),
            z_index: 0,
            gesture_handlers: Vec::new(),
        }
    }
    pub fn finish<'n, State: 'static>(self) -> Node<'n, State, AppState<State>> {
        self.view().finish()
    }
}

impl Svg {
    pub(crate) fn draw<State>(
        &mut self,
        area: Area,
        _state: &mut State,
        app: &mut AppState<State>,
        visible: bool,
        visible_amount: f32,
    ) {
        if !visible && visible_amount == 0. {
            return;
        }
        #[allow(clippy::map_entry)]
        if !app.svg_scenes.contains_key(&self.content) {
            match vello_svg::usvg::Tree::from_data(
                self.content.as_bytes(),
                &vello_svg::usvg::Options::default(),
            ) {
                Err(err) => {
                    eprintln!("Loading svg failed: {err}");
                    app.svg_scenes
                        .insert(self.content.clone(), (Scene::new(), 0., 0.));
                }
                Ok(svg) => {
                    let svg_scene = vello_svg::render_tree(&svg);
                    let size = svg.size();
                    app.svg_scenes.insert(
                        self.content.clone(),
                        (svg_scene, size.width(), size.height()),
                    );
                }
            }
        }
        let AppState {
            svg_scenes, scene, ..
        } = app;
        if let Some((svg_scene, width, height)) = svg_scenes.get(&self.content) {
            let width = *width as f64;
            let height = *height as f64;
            let area_x = area.x as f64 * app.scale_factor;
            let area_y = area.y as f64 * app.scale_factor;
            let area_width = area.width as f64 * app.scale_factor;
            let area_height = area.height as f64 * app.scale_factor;
            if self.fill.is_some() {
                scene.push_layer(
                    peniko::BlendMode {
                        mix: Mix::Normal,
                        compose: Compose::SrcOver,
                    },
                    1.0,
                    Affine::IDENTITY,
                    &kurbo::Rect::from_origin_size(
                        kurbo::Point::new(area_x, area_y),
                        kurbo::Size::new(area_width, area_height),
                    ),
                );
            }
            scene.append(
                svg_scene,
                Some(if self.unlocked_aspect_ratio {
                    Affine::IDENTITY
                        .then_scale_non_uniform(area_width / width, area_height / height)
                        .then_translate(Vec2::new(area_x, area_y))
                } else {
                    let scale = (area_width / width).min(area_height / height);
                    let dx = area_x + (area_width - width * scale) / 2.0;
                    let dy = area_y + (area_height - height * scale) / 2.0;
                    Affine::IDENTITY
                        .then_scale(scale)
                        .then_translate(Vec2::new(dx, dy))
                }),
            );
            if let Some(fill) = self.fill {
                scene.push_layer(
                    peniko::BlendMode {
                        mix: Mix::Clip,
                        compose: Compose::SrcIn,
                    },
                    1.0,
                    Affine::IDENTITY,
                    &kurbo::Rect::from_origin_size(
                        kurbo::Point::new(area_x, area_y),
                        kurbo::Size::new(area_width, area_height),
                    ),
                );

                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    fill,
                    None,
                    &kurbo::Rect::from_origin_size(
                        kurbo::Point::new(area_x, area_y),
                        kurbo::Size::new(area_width, area_height),
                    ),
                );
                scene.pop_layer();
                scene.pop_layer();
            }
        }
    }
}
