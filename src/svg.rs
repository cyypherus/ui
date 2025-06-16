use crate::app::AppState;
use crate::view::{View, ViewType};
use backer::models::Area;
use backer::Node;
use lilt::Easing;
use vello_svg::vello::kurbo::{self, Affine, Vec2};
use vello_svg::vello::peniko::{Color, Compose, Fill, Mix};
use vello_svg::vello::{peniko, Scene};

#[derive(Debug, Clone)]
pub struct Svg {
    pub(crate) id: u64,
    pub(crate) source: String,
    pub(crate) unlocked_aspect_ratio: bool,
    pub(crate) fill: Option<Color>,
    pub(crate) easing: Option<Easing>,
    pub(crate) duration: Option<f32>,
    pub(crate) delay: f32,
}

pub fn svg(id: u64, source: impl AsRef<str>) -> Svg {
    Svg {
        id,
        source: source.as_ref().to_string(),
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
            gesture_handlers: Vec::new(),
        }
    }
    pub fn finish<'n, State: 'static>(self) -> Node<'n, State, AppState> {
        self.view().finish()
    }
}

impl Svg {
    pub(crate) fn draw<State>(
        &mut self,
        area: Area,
        state: &mut State,
        app: &mut AppState,
        visible: bool,
        visible_amount: f32,
    ) {
        if !visible && visible_amount == 0. {
            return;
        }
        #[allow(clippy::map_entry)]
        if !app.image_scenes.contains_key(&self.source) {
            match std::fs::read(self.source.clone()) {
                Err(err) => eprintln!("Loading svg failed: {err}"),
                Ok(image_data) => match vello_svg::usvg::Tree::from_data(
                    image_data.as_slice(),
                    &vello_svg::usvg::Options::default(),
                ) {
                    Err(err) => {
                        eprintln!("Loading svg failed: {err}");
                        app.image_scenes
                            .insert(self.source.clone(), (Scene::new(), 0., 0.));
                    }
                    Ok(svg) => {
                        let svg_scene = vello_svg::render_tree(&svg);
                        let size = svg.size();
                        app.image_scenes.insert(
                            self.source.clone(),
                            (svg_scene, size.width(), size.height()),
                        );
                    }
                },
            }
        }
        let AppState {
            image_scenes,
            scene,
            ..
        } = app;
        if let Some((svg_scene, width, height)) = image_scenes.get(&self.source) {
            if self.fill.is_some() {
                scene.push_layer(
                    peniko::BlendMode {
                        mix: Mix::Normal,
                        compose: Compose::SrcOver,
                    },
                    1.0,
                    Affine::IDENTITY,
                    &kurbo::Rect::from_origin_size(
                        kurbo::Point::new(area.x as f64, area.y as f64),
                        kurbo::Size::new(area.width as f64, area.height as f64),
                    ),
                );
            }
            scene.append(
                svg_scene,
                Some(if self.unlocked_aspect_ratio {
                    Affine::IDENTITY
                        .then_scale_non_uniform(
                            (area.width / width) as f64,
                            (area.height / height) as f64,
                        )
                        .then_translate(Vec2::new(area.x as f64, area.y as f64))
                } else {
                    let scale = (area.width / width).min(area.height / height) as f64;
                    let dx = area.x as f64 + (area.width as f64 - *width as f64 * scale) / 2.0;
                    let dy = area.y as f64 + (area.height as f64 - *height as f64 * scale) / 2.0;
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
                        kurbo::Point::new(area.x as f64, area.y as f64),
                        kurbo::Size::new(area.width as f64, area.height as f64),
                    ),
                );

                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    fill,
                    None,
                    &kurbo::Rect::from_origin_size(
                        kurbo::Point::new(area.x as f64, area.y as f64),
                        kurbo::Size::new(area.width as f64, area.height as f64),
                    ),
                );
                scene.pop_layer();
                scene.pop_layer();
            }
        }
    }
}
