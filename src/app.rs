use crate::draw_layout::draw_layout;
use crate::gestures::{ClickLocation, Interaction, ScrollDelta};

use crate::text::TextLayout;
use crate::view::DrawableType;
use crate::{ClickState, DragState, Editor, GestureHandler, Point, area_contains};
use crate::{GestureState, RUBIK_FONT, area_contains_padded, event};
use backer::{Area, Layout};
use parley::fontique::Blob;
use parley::fontique::FontInfoOverride;
use parley::{
    Alignment, FontContext, FontWeight, LayoutContext, LineHeight, OverflowWrap, PlainEditor,
    StyleProperty,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use vello_svg::vello::kurbo::{Affine, BezPath};
use vello_svg::vello::peniko::{Brush, Color, Fill, Mix};
use vello_svg::vello::util::{RenderContext, RenderSurface};
use vello_svg::vello::{Renderer, RendererOptions, Scene};
use winit::event::{Modifiers, MouseScrollDelta};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Fullscreen, Icon};
use winit::{application::ApplicationHandler, event_loop::EventLoop, window::Window};
use winit::{dpi::LogicalSize, event::MouseButton};

#[cfg(target_os = "macos")]
use winit::platform::macos::WindowAttributesExtMacOS;

#[cfg(target_os = "windows")]
use winit::platform::windows::WindowAttributesExtWindows;

type FontEntry = (Arc<Vec<u8>>, Option<String>);

pub struct AppBuilder<State> {
    state: State,
    view: for<'a> fn(&'a State, &mut AppState) -> Layout<'a, View<State>, AppCtx>,
    on_frame: fn(&mut State, &mut AppState) -> (),
    on_start: fn(&mut State, &mut AppState) -> (),
    on_exit: fn(&mut State, &mut AppState) -> (),
    inner_size: Option<(u32, u32)>,
    resizable: Option<bool>,
    title: Option<String>,
    icon: Option<Icon>,
    custom_fonts: Vec<FontEntry>,
}

impl<State: 'static> AppBuilder<State> {
    pub fn new(
        state: State,
        view: for<'a> fn(&'a State, &mut AppState) -> Layout<'a, View<State>, AppCtx>,
    ) -> Self {
        Self {
            state,
            view,
            on_frame: |_, _| {},
            on_start: |_, _| {},
            on_exit: |_, _| {},
            inner_size: None,
            resizable: None,
            title: None,
            icon: None,
            custom_fonts: Vec::new(),
        }
    }
    pub fn add_font_bytes(mut self, bytes: Vec<u8>, family: Option<&str>) -> Self {
        self.custom_fonts
            .push((Arc::new(bytes), family.map(|s| s.to_string())));
        self
    }

    pub fn inner_size(mut self, width: u32, height: u32) -> Self {
        self.inner_size = Some((width, height));
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = Some(resizable);
        self
    }

    pub fn on_frame(mut self, on_frame: fn(&mut State, &mut AppState) -> ()) -> Self {
        self.on_frame = on_frame;
        self
    }

    pub fn on_start(mut self, on_start: fn(&mut State, &mut AppState) -> ()) -> Self {
        self.on_start = on_start;
        self
    }

    pub fn on_exit(mut self, on_exit: fn(&mut State, &mut AppState) -> ()) -> Self {
        self.on_exit = on_exit;
        self
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Does nothing on macOS
    /// 32x32 is a reasonable size
    pub fn icon(mut self, icon: &[u8]) -> Self {
        let img = image::load_from_memory(icon).expect("Invalid icon bytes");
        let rgba_img = img.to_rgba8();
        let (width, height) = rgba_img.dimensions();
        self.icon = Some(
            Icon::from_rgba(rgba_img.into_raw(), width, height).expect("Failed to create icon"),
        );
        self
    }

    pub fn start(self) {
        let event_loop: EventLoop<AppEvent> = EventLoop::with_user_event()
            .build()
            .expect("Could not create event loop");
        #[allow(unused_mut)]
        let mut render_cx = RenderContext::new();
        #[cfg(not(target_arch = "wasm32"))]
        {
            App::run(
                self.state,
                event_loop,
                render_cx,
                self.view,
                self.on_frame,
                self.on_start,
                self.on_exit,
                self.inner_size,
                self.resizable,
                self.title,
                self.icon,
                self.custom_fonts,
            );
        }
    }
}

pub struct App<'s, State> {
    pub(crate) context: RenderContext,
    pub(crate) renderers: Vec<Option<Renderer>>,
    pub(crate) render_state: Option<RenderState<'s>>,
    pub(crate) cached_window: Option<Arc<Window>>,
    pub(crate) window_inner_size: Option<(u32, u32)>,
    pub(crate) window_resizable: Option<bool>,
    pub(crate) window_title: Option<String>,
    pub(crate) window_icon: Option<Icon>,
    pub(crate) app_state: AppState,
    pub(crate) gesture_handlers: Vec<(u64, Area, GestureHandler<State, AppState>)>,
    pub state: State,
    pub(crate) view: for<'a> fn(&'a State, &mut AppState) -> Layout<'a, View<State>, AppCtx>,
    pub(crate) on_frame: fn(&mut State, &mut AppState) -> (),
    pub(crate) on_start: fn(&mut State, &mut AppState) -> (),
    pub(crate) on_exit: fn(&mut State, &mut AppState) -> (),
    pub(crate) started: bool,
    pub(crate) last_window_size: Option<winit::dpi::PhysicalSize<u32>>,
}

pub(crate) struct RenderState<'surface> {
    // SAFETY: We MUST drop the surface before the `window`, so the fields
    // must be in this order
    pub(crate) surface: RenderSurface<'surface>,
    pub(crate) window: Arc<Window>,
}

pub(crate) type LayoutCache = HashMap<u64, Vec<(String, f32, parley::Layout<Brush>)>>;

pub struct AppCtx {
    pub(crate) text_layout: TextLayout,
    pub(crate) font_cx: FontContext,
    pub(crate) layout_cx: LayoutContext<Brush>,
    pub(crate) scale_factor: f64,
    pub(crate) editor: Option<EditState>,
    pub(crate) editor_areas: HashMap<u64, Area>,
}

pub struct AppState {
    pub cursor_position: Option<Point>,
    pub(crate) gesture_state: GestureState,
    pub(crate) runtime: Runtime,
    pub(crate) cancellation_token: CancellationToken,
    pub(crate) task_tracker: TaskTracker,
    pub(crate) scene: Scene,
    pub(crate) app_context: AppCtx,
    pub(crate) layout_cache: LayoutCache,
    pub(crate) svg_scenes: HashMap<String, (Scene, f32, f32)>,
    pub(crate) image_scenes: HashMap<u64, (Scene, f32, f32)>,
    pub(crate) modifiers: Option<Modifiers>,
    pub(crate) redraw: Sender<()>,
    pub(crate) fullscreen_requested: bool,
}

pub enum View<State> {
    Draw {
        view: Box<DrawableType>,
        gesture_handlers: Vec<GestureHandler<State, AppState>>,
        area: Area,
    },
    PushClip {
        path: BezPath,
    },
    PopClip,
    EditorArea(u64, Area),
    Empty,
}

pub(crate) struct EditState {
    pub(crate) id: u64,
    pub(crate) editor: Editor,
    pub(crate) editing: bool,
    pub(crate) cursor_color: Brush,
    pub(crate) highlight_color: Brush,
}

impl Clone for EditState {
    fn clone(&self) -> Self {
        EditState {
            id: self.id,
            editor: self.editor.clone(),
            editing: self.editing,
            cursor_color: self.cursor_color.clone(),
            highlight_color: self.highlight_color.clone(),
        }
    }
}

impl AppState {
    pub fn ctx(&mut self) -> &mut AppCtx {
        &mut self.app_context
    }

    pub fn end_editing(&mut self) {
        if self.app_context.editor.is_some() {
            self.app_context.editor = None;
        }
    }

    pub fn set_fullscreen(&mut self) {
        self.fullscreen_requested = true;
    }

    pub fn exit_fullscreen(&mut self) {
        self.fullscreen_requested = false;
    }

    pub(crate) fn begin_editing(
        &mut self,
        id: u64,
        text: String,
        fill: Brush,
        font_family: String,
        font_weight: FontWeight,
        line_height: f32,
        font_size: f32,
        overflow_wrap: OverflowWrap,
        alignment: Alignment,
        cursor_fill: Brush,
        highlight_fill: Brush,
        wrap: bool,
    ) {
        if self.app_context.editor.is_some() {
            return;
        }
        let Some(area) = self.app_context.editor_areas.get(&id) else {
            return;
        };
        let mut editor = PlainEditor::new(font_size);
        editor.set_text(&text);
        let styles = editor.edit_styles();

        styles.insert(parley::StyleProperty::Brush(fill));
        styles.insert(parley::FontFamily::Named(font_family.into()).into());
        styles.insert(StyleProperty::FontWeight(font_weight));
        styles.insert(StyleProperty::LineHeight(LineHeight::FontSizeRelative(
            line_height,
        )));
        styles.insert(StyleProperty::FontSize(font_size));
        styles.insert(StyleProperty::OverflowWrap(overflow_wrap));

        editor.set_alignment(alignment);
        if wrap {
            editor.set_width(Some(area.width));
        }
        let mut editor = Editor {
            editor,
            last_click_time: Default::default(),
            click_count: Default::default(),
            pointer_down: Default::default(),
            cursor_pos: Default::default(),
            cursor_visible: Default::default(),
            modifiers: Default::default(),
            start_time: Default::default(),
            blink_period: Default::default(),
        };

        if let Some(pos) = self.cursor_position {
            editor.mouse_moved(
                Point::new(pos.x - area.x as f64, pos.y - area.y as f64),
                &mut self.app_context.layout_cx,
                &mut self.app_context.font_cx,
            );
        }
        self.app_context.editor = Some(EditState {
            id,
            editor,
            editing: true,
            cursor_color: cursor_fill,
            highlight_color: highlight_fill,
        });
    }

    pub fn spawn(&self, task: impl std::future::Future<Output = ()> + Send + 'static) {
        self.task_tracker.spawn_on(task, self.runtime.handle());
    }

    pub fn redraw_trigger(&self) -> RedrawTrigger {
        RedrawTrigger::new(self.redraw.clone())
    }

    pub fn redraw(&self) {
        let _ = self.redraw.blocking_send(());
    }
}

#[derive(Debug, Clone)]
pub struct RedrawTrigger {
    sender: Sender<()>,
}

impl RedrawTrigger {
    pub(crate) fn new(sender: Sender<()>) -> Self {
        Self { sender }
    }

    pub async fn trigger(&self) {
        self.sender.send(()).await.ok();
    }
}

impl<State: 'static> App<'_, State> {
    pub fn start(
        state: State,
        view: for<'a> fn(&'a State, &mut AppState) -> Layout<'a, View<State>, AppCtx>,
    ) {
        AppBuilder::new(state, view).start();
    }

    pub fn builder(
        state: State,
        view: for<'a> fn(&'a State, &mut AppState) -> Layout<'a, View<State>, AppCtx>,
    ) -> AppBuilder<State> {
        AppBuilder::new(state, view)
    }

    fn request_redraw(&self) {
        let Some(RenderState { window, .. }) = &self.render_state else {
            return;
        };
        window.request_redraw();
    }

    fn gesture_handlers(&self) -> Vec<(u64, Area, GestureHandler<State, AppState>)> {
        self.gesture_handlers.clone()
    }

    fn run(
        state: State,
        event_loop: EventLoop<AppEvent>,
        render_cx: RenderContext,
        #[cfg(target_arch = "wasm32")] render_state: RenderState,
        view: for<'a> fn(&'a State, &mut AppState) -> Layout<'a, View<State>, AppCtx>,
        on_frame: fn(&mut State, &mut AppState) -> (),
        on_start: fn(&mut State, &mut AppState) -> (),
        on_exit: fn(&mut State, &mut AppState) -> (),
        inner_size: Option<(u32, u32)>,
        resizable: Option<bool>,
        title: Option<String>,
        icon: Option<Icon>,
        custom_fonts: Vec<FontEntry>,
    ) {
        #[allow(unused_mut)]
        let mut renderers: Vec<Option<Renderer>> = vec![];

        let mut font_cx = FontContext::new();

        font_cx
            .collection
            .register_fonts(Blob::new(Arc::new(RUBIK_FONT)), None);

        for (font_bytes, family_opt) in custom_fonts.into_iter() {
            font_cx.collection.register_fonts(
                Blob::new(font_bytes),
                Some(FontInfoOverride {
                    family_name: family_opt.as_deref(),
                    ..Default::default()
                }),
            );
        }

        let render_state = None::<RenderState>;
        let runtime = Runtime::new().expect("Failed to create runtime");

        let redraw_proxy = event_loop.create_proxy();
        let (redraw_sender, mut redraw_receiver) = tokio::sync::mpsc::channel::<()>(10);
        runtime.spawn(async move {
            loop {
                if redraw_receiver.recv().await.is_some() {
                    redraw_proxy
                        .send_event(AppEvent::RequestRedraw)
                        .expect("Event send failed");
                }
            }
        });

        let layout_cache = HashMap::new();
        let layout_cx = LayoutContext::new();
        let font_cx_inner = FontContext::new();

        let mut app = Self {
            context: render_cx,
            renderers,
            render_state,
            cached_window: None,
            window_inner_size: inner_size,
            window_resizable: resizable,
            window_title: title,
            window_icon: icon,
            state,
            view,
            gesture_handlers: Vec::new(),

            app_state: AppState {
                cursor_position: None,
                gesture_state: GestureState::None,
                runtime,
                cancellation_token: CancellationToken::new(),
                task_tracker: TaskTracker::new(),
                scene: Scene::new(),
                app_context: AppCtx {
                    text_layout: TextLayout::new(layout_cache, font_cx_inner, layout_cx),
                    font_cx: FontContext::new(),
                    layout_cx: LayoutContext::new(),
                    scale_factor: 1.,
                    editor: None,
                    editor_areas: HashMap::new(),
                },
                layout_cache: HashMap::new(),
                image_scenes: HashMap::new(),
                svg_scenes: HashMap::new(),
                modifiers: None,
                redraw: redraw_sender,
                fullscreen_requested: false,
            },
            on_frame,
            on_start,
            on_exit,
            started: false,
            last_window_size: None,
        };

        event_loop.run_app(&mut app).expect("run to completion");
        (app.on_exit)(&mut app.state, &mut app.app_state);

        app.app_state.cancellation_token.cancel();

        app.app_state.task_tracker.close();

        app.app_state
            .runtime
            .shutdown_timeout(Duration::from_secs(5));
    }

    fn redraw(&mut self) {
        if !self.started {
            self.started = true;
            (self.on_start)(&mut self.state, &mut self.app_state);
        }

        self.gesture_handlers.clear();
        if let Self {
            context,
            render_state: Some(RenderState { surface, window }),
            ..
        } = self
        {
            let size = window.inner_size();
            self.last_window_size = Some(size);
            self.app_state.app_context.scale_factor = window.scale_factor();
            let size = window.inner_size();
            let width = size.width;
            let height = size.height;
            if surface.config.width != width || surface.config.height != height {
                context.resize_surface(surface, width, height);
            }

            let view = self.view;
            let mut layout = view(&self.state, &mut self.app_state);
            let draw_items = layout.draw(
                Area {
                    x: 0.,
                    y: 0.,
                    width: ((width as f64) / self.app_state.app_context.scale_factor) as f32,
                    height: ((height as f64) / self.app_state.app_context.scale_factor) as f32,
                },
                &mut self.app_state.app_context,
            );

            for item in draw_items {
                match item {
                    View::PushClip { path } => {
                        self.app_state.scene.push_layer(
                            Fill::NonZero,
                            Mix::Normal,
                            1.,
                            Affine::scale(self.app_state.app_context.scale_factor),
                            &path,
                        );
                    }
                    View::PopClip => {
                        self.app_state.scene.pop_layer();
                    }
                    View::EditorArea(id, area) => {
                        self.app_state.app_context.editor_areas.insert(id, area);
                    }
                    View::Draw {
                        mut view,
                        gesture_handlers,
                        area,
                    } => {
                        let id = view.id();
                        let draw_area = area;

                        self.gesture_handlers.extend(
                            gesture_handlers
                                .into_iter()
                                .map(|handler| (id, draw_area, handler)),
                        );

                        match &mut *view {
                            DrawableType::Text(v) => v.draw(draw_area, area, &mut self.app_state),
                            DrawableType::Layout(boxed) => {
                                let (layout, transform) = boxed.as_mut();
                                draw_layout(None, *transform, layout, &mut self.app_state.scene)
                            }
                            DrawableType::Path(v) => v.draw(
                                &mut self.app_state.scene,
                                draw_area,
                                self.app_state.app_context.scale_factor,
                            ),
                            DrawableType::Svg(v) => v.draw(draw_area, &mut self.app_state),
                            DrawableType::Image(v) => v.draw(draw_area, &mut self.app_state),
                        }
                    }
                    View::Empty => (),
                }
            }
        }

        (self.on_frame)(&mut self.state, &mut self.app_state);
        let Self {
            context,
            render_state: Some(RenderState { surface, window }),
            app_state: AppState { scene, .. },
            ..
        } = self
        else {
            return;
        };

        let size = window.inner_size();
        let width = size.width;
        let height = size.height;

        let device_handle = &context.devices[surface.dev_id];

        let render_params = vello_svg::vello::RenderParams {
            base_color: Color::BLACK,
            width,
            height,
            antialiasing_method: vello_svg::vello::AaConfig::Area,
        };

        window.pre_present_notify();

        self.renderers[surface.dev_id]
            .as_mut()
            .unwrap()
            .render_to_texture(
                &device_handle.device,
                &device_handle.queue,
                scene,
                &surface.target_view,
                &render_params,
            )
            .expect("failed to render to texture");

        let surface_texture = surface
            .surface
            .get_current_texture()
            .expect("failed to get surface texture");

        let mut encoder =
            device_handle
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Surface Blit"),
                });
        surface.blitter.copy(
            &device_handle.device,
            &mut encoder,
            &surface.target_view,
            &surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
        );
        device_handle.queue.submit([encoder.finish()]);
        surface_texture.present();

        scene.reset();
    }
}

#[derive(Debug, Clone, Copy)]
enum AppEvent {
    RequestRedraw,
}

impl<State: 'static> ApplicationHandler<AppEvent> for App<'_, State> {
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::RequestRedraw => {
                self.request_redraw();
            }
        }
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let Option::None = self.render_state else {
            return;
        };
        let window = self.cached_window.take().unwrap_or_else(|| {
            let inner_size = self.window_inner_size.take().unwrap_or((1044, 800));
            let resizable = self.window_resizable.take().unwrap_or(true);

            #[cfg(target_os = "macos")]
            let mut attributes = Window::default_attributes()
                .with_inner_size(LogicalSize::new(inner_size.0, inner_size.1))
                .with_resizable(resizable)
                .with_decorations(true)
                .with_titlebar_hidden(false)
                .with_titlebar_transparent(true)
                .with_title_hidden(true)
                .with_fullsize_content_view(true)
                .with_window_icon(self.window_icon.clone());

            #[cfg(target_os = "windows")]
            let mut attributes = Window::default_attributes()
                .with_inner_size(LogicalSize::new(inner_size.0, inner_size.1))
                .with_resizable(resizable)
                .with_decorations(true)
                .with_visible(false)
                .with_window_icon(self.window_icon.clone())
                .with_taskbar_icon(self.window_icon.clone());

            if let Some(ref title) = self.window_title {
                attributes = attributes.with_title(title.clone());
            }

            Arc::new(event_loop.create_window(attributes).unwrap())
        });
        let size = window.inner_size();
        let surface_future = self.context.create_surface(
            window.clone(),
            size.width,
            size.height,
            vello_svg::vello::wgpu::PresentMode::Immediate,
        );
        let surface = pollster::block_on(surface_future).expect("Error creating surface");
        let render_state = RenderState { window, surface };
        let devices_len = self.context.devices.len();
        self.renderers.resize_with(devices_len, || None);
        let render_state = {
            let Self {
                context, renderers, ..
            } = self;
            let id = render_state.surface.dev_id;
            renderers[id].get_or_insert_with(|| {
                #[allow(unused_mut)]
                let mut renderer =
                    Renderer::new(&context.devices[id].device, RendererOptions::default())
                        .expect("Failed to create renderer");
                renderer
            });
            Some(render_state)
        };
        self.render_state = render_state;

        #[cfg(target_os = "windows")]
        if let Self {
            render_state: Some(RenderState { window, .. }),
            ..
        } = self
        {
            // Windows flashes white on startup so we delay display until the renderer is configured
            window.set_visible(true);
        }
        self.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some(event) = crate::event::WindowEvent::from_winit_window_event(event) {
            match event {
                event::WindowEvent::Moved(_) => {}
                event::WindowEvent::KeyPressed(key) => {
                    let Some(key) = crate::Key::from(key) else {
                        return;
                    };
                    let mut needs_redraw = false;
                    for (_id, _area, handler) in self.gesture_handlers() {
                        if let Some(ref interaction_handler) = handler.interaction_handler
                            && handler.interaction_type.key
                        {
                            needs_redraw = true;
                            interaction_handler(
                                &mut self.state,
                                &mut self.app_state,
                                Interaction::Key(key.clone()),
                            );
                        }
                    }
                    if needs_redraw {
                        self.request_redraw();
                    }
                }
                event::WindowEvent::KeyReleased(_) => {}
                event::WindowEvent::MouseMoved(pos) => self.mouse_moved(pos),
                event::WindowEvent::MousePressed(MouseButton::Left) => self.mouse_pressed(),
                event::WindowEvent::MouseReleased(MouseButton::Left) => self.mouse_released(),
                event::WindowEvent::MousePressed(_) => {}
                event::WindowEvent::MouseReleased(_) => {}
                event::WindowEvent::MouseEntered => {}
                event::WindowEvent::MouseExited => {}
                event::WindowEvent::MouseWheel(delta, _phase) => {
                    self.scrolled(delta);
                }
                event::WindowEvent::Resized(_) => {}
                event::WindowEvent::HoveredFile(_) => {}
                event::WindowEvent::DroppedFile(_) => {}
                event::WindowEvent::HoveredFileCancelled => {}
                event::WindowEvent::Touch(_) => {}
                event::WindowEvent::TouchPressure(_) => {}
                event::WindowEvent::Focused => {
                    self.request_redraw();
                }
                event::WindowEvent::Unfocused => {}
                event::WindowEvent::Closed => event_loop.exit(),
                event::WindowEvent::RedrawRequested => self.redraw(),
                event::WindowEvent::ScaleFactorChanged(scale_factor) => {
                    self.app_state.app_context.scale_factor = scale_factor;
                    self.app_state.layout_cache.clear();
                    self.request_redraw();
                }
                event::WindowEvent::ModifiersChanged(modifiers) => {
                    self.app_state.modifiers = Some(modifiers);
                }
            }
        }
    }
}
impl<State: 'static> App<'_, State> {
    pub(crate) fn mouse_moved(&mut self, pos: Point) {
        let pos = Point::new(
            pos.x / self.app_state.app_context.scale_factor,
            pos.y / self.app_state.app_context.scale_factor,
        );
        let mut needs_redraw = false;
        self.app_state.cursor_position = Some(pos);
        if let Some(EditState { id, editor, .. }) = self.app_state.app_context.editor.as_mut()
            && let Some(area) = self.app_state.app_context.editor_areas.get(id).copied()
        {
            needs_redraw = true;
            editor.mouse_moved(
                Point::new(pos.x - area.x as f64, pos.y - area.y as f64),
                &mut self.app_state.app_context.layout_cx,
                &mut self.app_state.app_context.font_cx,
            );
        }
        self.gesture_handlers().iter().for_each(|(_, area, gh)| {
            if gh.interaction_type.hover
                && let Some(ref on_hover) = gh.interaction_handler
            {
                needs_redraw = true;
                (on_hover)(
                    &mut self.state,
                    &mut self.app_state,
                    Interaction::Hover(area_contains(area, pos)),
                );
            }
        });
        if let GestureState::Dragging {
            start,
            last_position,
            capturer,
        } = self.app_state.gesture_state
        {
            let distance = start.distance(pos);
            let delta = Point {
                x: pos.x - last_position.x,
                y: pos.y - last_position.y,
            };
            self.gesture_handlers()
                .iter()
                .filter(|(id, _, gh)| *id == capturer && gh.interaction_type.drag)
                .for_each(|(_, area, gh)| {
                    needs_redraw = true;
                    if let Some(handler) = &gh.interaction_handler {
                        (handler)(
                            &mut self.state,
                            &mut self.app_state,
                            Interaction::Drag(DragState::Updated {
                                start: Point {
                                    x: start.x - area.x as f64,
                                    y: start.y - area.y as f64,
                                },
                                current: Point {
                                    x: pos.x - area.x as f64,
                                    y: pos.y - area.y as f64,
                                },
                                start_global: start,
                                current_global: pos,
                                delta,
                                distance: distance as f32,
                            }),
                        );
                    }
                });
            // Update last_position for next delta calculation
            self.app_state.gesture_state = GestureState::Dragging {
                start,
                last_position: pos,
                capturer,
            };
        }
        if needs_redraw {
            self.request_redraw();
        }
    }
    pub(crate) fn mouse_pressed(&mut self) {
        let mut needs_redraw = false;
        if let Some(point) = self.app_state.cursor_position {
            for (_, area, handler) in
                self.gesture_handlers()
                    .iter()
                    .rev()
                    .filter(|(_, area, handler)| {
                        handler.interaction_type.click_outside
                            && !area_contains_padded(area, point, 10.)
                    })
            {
                if handler.interaction_type.click_outside
                    && let Some(ref on_click_outside) = handler.interaction_handler
                {
                    on_click_outside(
                        &mut self.state,
                        &mut self.app_state,
                        Interaction::ClickOutside(
                            ClickState::Started,
                            ClickLocation::new(point, *area),
                        ),
                    );
                }
            }
            if let Some((capturer, area, handler)) = self
                .gesture_handlers()
                .iter()
                .rev()
                .find(|(_, area, handler)| {
                    area_contains(area, point)
                        && (handler.interaction_type.click || handler.interaction_type.drag)
                })
                .or(self
                    .gesture_handlers()
                    .iter()
                    .rev()
                    .find(|(_, area, handler)| {
                        area_contains(
                            &Area {
                                x: area.x - 10.,
                                y: area.y - 10.,
                                width: area.width + 20.,
                                height: area.height + 20.,
                            },
                            point,
                        ) && (handler.interaction_type.click || handler.interaction_type.drag)
                    }))
            {
                needs_redraw = true;
                if handler.interaction_type.click
                    && let Some(ref on_click) = handler.interaction_handler
                {
                    on_click(
                        &mut self.state,
                        &mut self.app_state,
                        Interaction::Click(ClickState::Started, ClickLocation::new(point, *area)),
                    );
                } else if handler.interaction_type.drag
                    && let Some(ref on_drag) = handler.interaction_handler
                {
                    on_drag(
                        &mut self.state,
                        &mut self.app_state,
                        Interaction::Drag(DragState::Began {
                            start: Point {
                                x: point.x - area.x as f64,
                                y: point.y - area.y as f64,
                            },
                            start_global: point,
                        }),
                    );
                }
                self.app_state.gesture_state = GestureState::Dragging {
                    start: point,
                    last_position: point,
                    capturer: *capturer,
                }
            }
            // Once all click handlers are run, text fields will have set up an editor if they have been clicked, so we can send the mouse press to the editor
            if let Some(EditState { id, editor, .. }) = self.app_state.app_context.editor.as_mut()
                && let Some(area) = self.app_state.app_context.editor_areas.get(id).cloned()
                && area_contains_padded(&area, point, 10.)
            {
                editor.mouse_pressed(
                    &mut self.app_state.app_context.layout_cx,
                    &mut self.app_state.app_context.font_cx,
                );
            }
        }

        if needs_redraw {
            self.request_redraw();
        }
    }
    pub(crate) fn mouse_released(&mut self) {
        let mut needs_redraw = false;
        if let Some(current) = self.app_state.cursor_position {
            if let Some(EditState { id, editor, .. }) = self.app_state.app_context.editor.as_mut()
                && let Some(area) = self.app_state.app_context.editor_areas.get(id)
            {
                editor.mouse_released();
                needs_redraw = true;
                if !area_contains(area, current)
                    && (!matches!(self.app_state.gesture_state, GestureState::Dragging { .. })
                        || match self.app_state.gesture_state {
                            GestureState::Dragging { capturer, .. } => capturer != *id,
                            _ => false,
                        })
                {
                    self.app_state.end_editing();
                }
            }
            if let GestureState::Dragging {
                start,
                last_position,
                capturer,
            } = self.app_state.gesture_state
            {
                let distance = start.distance(current);
                let delta = Point {
                    x: current.x - last_position.x,
                    y: current.y - last_position.y,
                };
                self.gesture_handlers()
                    .iter()
                    .filter(|(id, _, _)| *id == capturer)
                    .for_each(|(_, area, gh)| {
                        if let (Some(on_click), true) =
                            (&gh.interaction_handler, gh.interaction_type.click)
                        {
                            needs_redraw = true;
                            if area_contains(area, current) {
                                on_click(
                                    &mut self.state,
                                    &mut self.app_state,
                                    Interaction::Click(
                                        ClickState::Completed,
                                        ClickLocation::new(current, *area),
                                    ),
                                );
                            } else {
                                on_click(
                                    &mut self.state,
                                    &mut self.app_state,
                                    Interaction::Click(
                                        ClickState::Cancelled,
                                        ClickLocation::new(current, *area),
                                    ),
                                );
                            }
                        }
                        if let (Some(on_drag), true) =
                            (&gh.interaction_handler, gh.interaction_type.drag)
                        {
                            needs_redraw = true;
                            on_drag(
                                &mut self.state,
                                &mut self.app_state,
                                Interaction::Drag(DragState::Completed {
                                    start: Point {
                                        x: start.x - area.x as f64,
                                        y: start.y - area.y as f64,
                                    },
                                    current: Point {
                                        x: current.x - area.x as f64,
                                        y: current.y - area.y as f64,
                                    },
                                    start_global: start,
                                    current_global: current,
                                    delta,
                                    distance: distance as f32,
                                }),
                            );
                        }
                    });
            }
            let press_start = match self.app_state.gesture_state {
                GestureState::Dragging { start, .. } => Some(start),
                _ => None,
            };
            for (_, area, handler) in self
                .gesture_handlers()
                .iter()
                .filter(|(_, _, h)| h.interaction_type.click_outside)
            {
                if !area_contains_padded(area, current, 10.)
                    && press_start.is_some_and(|s| !area_contains_padded(area, s, 10.))
                    && let Some(ref handler) = handler.interaction_handler
                {
                    needs_redraw = true;
                    handler(
                        &mut self.state,
                        &mut self.app_state,
                        Interaction::ClickOutside(
                            ClickState::Completed,
                            ClickLocation::new(current, *area),
                        ),
                    );
                }
            }
        }
        self.app_state.gesture_state = GestureState::None;
        if needs_redraw {
            self.request_redraw();
        }

        if self.app_state.fullscreen_requested
            && let Some(render_state) = &self.render_state
            && render_state.window.fullscreen() != Some(Fullscreen::Borderless(None))
        {
            render_state
                .window
                .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        } else if let Some(render_state) = &self.render_state
            && render_state.window.fullscreen().is_some()
        {
            render_state.window.set_fullscreen(None);
        }
    }

    pub(crate) fn scrolled(&mut self, delta: MouseScrollDelta) {
        let mut needs_redraw = false;
        if let Some(current) = self.app_state.cursor_position
            && let Some((_, _, handler)) =
                self.gesture_handlers()
                    .iter()
                    .rev()
                    .find(|(_, area, handler)| {
                        area_contains(area, current) && (handler.interaction_type.scroll)
                    })
            && let Some(ref on_scroll) = handler.interaction_handler
        {
            needs_redraw = true;
            on_scroll(
                &mut self.state,
                &mut self.app_state,
                Interaction::Scroll(match delta {
                    MouseScrollDelta::LineDelta(x, y) => ScrollDelta {
                        x: x * 10.,
                        y: y * 10.,
                    },
                    MouseScrollDelta::PixelDelta(physical_position) => ScrollDelta {
                        x: physical_position.x as f32,
                        y: physical_position.y as f32,
                    },
                }),
            );
        }
        if needs_redraw {
            self.request_redraw();
        }
    }
}
