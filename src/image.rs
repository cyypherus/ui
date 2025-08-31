use crate::DEFAULT_CORNER_ROUNDING;
use crate::app::AppState;
use crate::view::{View, ViewType};
use backer::Node;
use backer::models::Area;
use lilt::Easing;
use std::sync::Arc;
use vello_svg::vello::kurbo::{Affine, Point, RoundedRect, Size, Vec2};
use vello_svg::vello::peniko::Mix;
use vello_svg::vello::{Scene, peniko};

#[derive(Debug, Clone)]
pub struct Image {
    pub(crate) id: u64,
    pub(crate) source: ImageSource,
    pub(crate) unlocked_aspect_ratio: bool,
    pub(crate) easing: Option<Easing>,
    pub(crate) duration: Option<f32>,
    pub(crate) delay: f32,
    pub(crate) image_id: Option<String>,
    pub(crate) corner_rounding: f32,
}

#[derive(Debug, Clone)]
pub enum ImageSource {
    Path(String),
    Bytes(Arc<Vec<u8>>),
}

pub fn image(id: u64, source: impl Into<ImageSource>) -> Image {
    Image {
        id,
        source: source.into(),
        easing: None,
        duration: None,
        delay: 0.,
        unlocked_aspect_ratio: false,
        image_id: None,
        corner_rounding: DEFAULT_CORNER_ROUNDING,
    }
}

pub fn image_from_path(id: u64, path: impl AsRef<str>) -> Image {
    image(id, ImageSource::Path(path.as_ref().to_string()))
}

pub fn image_from_bytes(id: u64, bytes: Arc<Vec<u8>>) -> Image {
    image(id, ImageSource::Bytes(bytes))
}

impl From<String> for ImageSource {
    fn from(path: String) -> Self {
        ImageSource::Path(path)
    }
}

impl From<&str> for ImageSource {
    fn from(path: &str) -> Self {
        ImageSource::Path(path.to_string())
    }
}

impl From<Vec<u8>> for ImageSource {
    fn from(bytes: Vec<u8>) -> Self {
        ImageSource::Bytes(Arc::new(bytes))
    }
}

impl From<Arc<Vec<u8>>> for ImageSource {
    fn from(bytes: Arc<Vec<u8>>) -> Self {
        ImageSource::Bytes(bytes)
    }
}

impl Image {
    /// Used to differentiate images when a view with the same id() is passed different image data.
    pub fn image_id(mut self, image_id: impl Into<String>) -> Self {
        self.image_id = Some(image_id.into());
        self
    }

    pub fn corner_rounding(mut self, radius: f32) -> Self {
        self.corner_rounding = radius;
        self
    }

    pub fn view<State>(self) -> View<State> {
        View {
            view_type: ViewType::Image(self),
            z_index: 0,
            gesture_handlers: Vec::new(),
        }
    }

    pub fn finish<'n, State: 'static>(self) -> Node<'n, State, AppState<State>> {
        self.view().finish()
    }
}

impl Image {
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

        let cache_key = if let Some(ref image_id) = self.image_id {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            self.id.hash(&mut hasher);
            image_id.hash(&mut hasher);
            hasher.finish()
        } else {
            self.id
        };

        if !app.image_scenes.contains_key(&cache_key) {
            let peniko_image = match self.load_image() {
                Ok(img) => img,
                Err(err) => {
                    eprintln!("Loading image failed: {err}");
                    app.image_scenes.insert(cache_key, (Scene::new(), 0., 0.));
                    return;
                }
            };

            let width = peniko_image.width as f32;
            let height = peniko_image.height as f32;

            let mut image_scene = Scene::new();
            image_scene.draw_image(&peniko_image, Affine::IDENTITY);

            app.image_scenes
                .insert(cache_key, (image_scene, width, height));
        }

        let AppState {
            image_scenes,
            scene,
            ..
        } = app;

        if let Some((image_scene, width, height)) = image_scenes.get(&cache_key) {
            let width = *width as f64;
            let height = *height as f64;
            let area_x = area.x as f64 * app.scale_factor;
            let area_y = area.y as f64 * app.scale_factor;
            let area_width = area.width as f64 * app.scale_factor;
            let area_height = area.height as f64 * app.scale_factor;
            let mut scale = 1.;

            let transform = if self.unlocked_aspect_ratio {
                Affine::IDENTITY
                    .then_scale_non_uniform(area_width / width, area_height / height)
                    .then_translate(Vec2::new(area_x, area_y))
            } else {
                scale = (area_width / width).min(area_height / height);
                let dx = area_x + (area_width - width * scale) / 2.0;
                let dy = area_y + (area_height - height * scale) / 2.0;
                Affine::IDENTITY
                    .then_scale(scale)
                    .then_translate(Vec2::new(dx, dy))
            };

            scene.push_layer(
                Mix::Normal,
                1.,
                transform,
                &RoundedRect::from_origin_size(
                    Point::ZERO,
                    Size::new(width, height),
                    self.corner_rounding as f64 / scale,
                ),
            );
            scene.append(image_scene, Some(transform));
            app.scene.pop_layer();
        }
    }

    fn load_image(&self) -> Result<peniko::Image, Box<dyn std::error::Error>> {
        let image_data = match &self.source {
            ImageSource::Path(path) => std::fs::read(path)?,
            ImageSource::Bytes(bytes) => bytes.as_ref().clone(),
        };

        let img = image::load_from_memory(&image_data)?;
        let rgba_img = img.to_rgba8();
        let (width, height) = rgba_img.dimensions();

        let blob = peniko::Blob::new(Arc::new(rgba_img.into_raw()));

        Ok(peniko::Image::new(
            blob,
            peniko::ImageFormat::Rgba8,
            width,
            height,
        ))
    }
}
