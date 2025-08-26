use crate::gestures::{ClickLocation, EditInteraction, Interaction, ScrollDelta};
use crate::ui::AnimationBank;
use crate::view::AnimatedView;
use crate::{Area, GestureState, RUBIK_FONT, TextState, event};
use crate::{Binding, ClickState, DragState, Editor, GestureHandler, Point, area_contains};
use backer::{Layout, Node};
use parley::fontique::Blob;
use parley::fontique::FontInfoOverride;
use parley::{FontContext, LayoutContext};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use vello_svg::vello::peniko::{Brush, Color};
use vello_svg::vello::util::{RenderContext, RenderSurface};
use vello_svg::vello::{Renderer, RendererOptions, Scene};
use winit::event::{Modifiers, MouseScrollDelta};
use winit::{application::ApplicationHandler, event_loop::EventLoop, window::Window};
use winit::{dpi::LogicalSize, event::MouseButton};

#[cfg(target_os = "macos")]
use winit::platform::macos::WindowAttributesExtMacOS;

type FontEntry = (Arc<Vec<u8>>, Option<String>);

pub struct AppBuilder<State> {
    state: State,
    view: fn() -> Node<'static, State, AppState<State>>,
    on_frame: fn(&mut State, &mut AppState<State>) -> (),
    on_start: fn(&mut State, &mut AppState<State>) -> (),
    on_exit: fn(&mut State, &mut AppState<State>) -> (),
    inner_size: Option<(u32, u32)>,
    resizable: Option<bool>,
    title: Option<String>,
    custom_fonts: Vec<FontEntry>,
}

impl<State: 'static> AppBuilder<State> {
    pub fn new(state: State, view: fn() -> Node<'static, State, AppState<State>>) -> Self {
        Self {
            state,
            view,
            on_frame: |_, _| {},
            on_start: |_, _| {},
            on_exit: |_, _| {},
            inner_size: None,
            resizable: None,
            title: None,
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

    pub fn on_frame(mut self, on_frame: fn(&mut State, &mut AppState<State>) -> ()) -> Self {
        self.on_frame = on_frame;
        self
    }

    pub fn on_start(mut self, on_start: fn(&mut State, &mut AppState<State>) -> ()) -> Self {
        self.on_start = on_start;
        self
    }

    pub fn on_exit(mut self, on_exit: fn(&mut State, &mut AppState<State>) -> ()) -> Self {
        self.on_exit = on_exit;
        self
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn start(self) {
        let event_loop = EventLoop::new().expect("Could not create event loop");
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
    pub(crate) app_state: AppState<State>,
    pub state: State,
    pub(crate) view: fn() -> Node<'static, State, AppState<State>>,
    pub(crate) on_frame: fn(&mut State, &mut AppState<State>) -> (),
    pub(crate) on_start: fn(&mut State, &mut AppState<State>) -> (),
    pub(crate) on_exit: fn(&mut State, &mut AppState<State>) -> (),
    pub(crate) started: bool,
}

pub(crate) struct RenderState<'surface> {
    // SAFETY: We MUST drop the surface before the `window`, so the fields
    // must be in this order
    pub(crate) surface: RenderSurface<'surface>,
    pub(crate) window: Arc<Window>,
}

type TextLayoutCache = HashMap<u64, Vec<(String, f32, parley::Layout<Brush>)>>;
pub struct AppState<State> {
    pub(crate) cursor_position: Option<Point>,
    pub(crate) gesture_state: GestureState,
    pub gesture_handlers: Vec<(u64, Area, GestureHandler<State, Self>)>,
    // pub(crate) background_scheduler: BackgroundScheduler<State>,
    pub(crate) runtime: Runtime,
    pub(crate) cancellation_token: CancellationToken,
    pub(crate) task_tracker: TaskTracker,
    pub(crate) scale_factor: f64,
    pub(crate) editor: Option<EditState<State>>,
    pub(crate) animation_bank: AnimationBank,
    pub(crate) scene: Scene,
    pub(crate) font_cx: FontContext,
    pub(crate) layout_cx: LayoutContext<Brush>,
    pub(crate) view_state: HashMap<u64, AnimatedView>,
    pub(crate) layout_cache: TextLayoutCache,
    pub(crate) svg_scenes: HashMap<String, (Scene, f32, f32)>,
    pub(crate) image_scenes: HashMap<u64, (Scene, f32, f32)>,
    pub(crate) modifiers: Option<Modifiers>,
    pub(crate) now: Instant,
    pub(crate) appeared_views: std::collections::HashSet<u64>,
}

pub(crate) struct EditState<State> {
    pub(crate) id: u64,
    pub(crate) area: Area,
    pub(crate) editor: Editor,
    pub(crate) editing: bool,
    pub(crate) binding: Binding<State, TextState>,
    pub(crate) cursor_color: Color,
    pub(crate) highlight_color: Color,
}

impl<State> Clone for EditState<State> {
    fn clone(&self) -> Self {
        EditState {
            id: self.id,
            area: self.area,
            editor: self.editor.clone(),
            editing: self.editing,
            binding: self.binding.clone(),
            cursor_color: self.cursor_color,
            highlight_color: self.highlight_color,
        }
    }
}

impl<State> AppState<State> {
    pub fn end_editing(&mut self, state: &mut State) {
        if let Some(EditState { id, binding, .. }) = self.editor.as_mut() {
            let current = binding.get(state);
            binding.set(
                state,
                TextState {
                    text: current.text,
                    editing: false,
                },
            );
            if let Some((_, _, handler)) = self
                .gesture_handlers
                .clone()
                .iter()
                .find(|(handler_id, _, handler)| handler_id == id && handler.interaction_type.edit)
                && let Some(ref handler) = handler.interaction_handler
            {
                (handler)(state, self, Interaction::Edit(EditInteraction::End));
            }
            self.editor = None;
        }
    }

    pub(crate) fn animations_in_progress(&self, now: Instant) -> bool {
        self.view_state.values().any(|v| match v {
            AnimatedView::Rect(animated_rect) => animated_rect.shape.in_progress(now),
            AnimatedView::Text(animated_text) => animated_text.fill.in_progress(now),
            AnimatedView::Circle(animated_circle) => animated_circle.shape.in_progress(now),
        })
    }

    pub fn spawn(&self, task: impl std::future::Future<Output = ()> + Send + 'static) {
        self.task_tracker.spawn_on(task, self.runtime.handle());
    }
}

impl<State: 'static> App<'_, State> {
    fn request_redraw(&self) {
        let Some(RenderState { window, .. }) = &self.render_state else {
            return;
        };
        window.request_redraw();
    }
    pub fn start(state: State, view: fn() -> Node<'static, State, AppState<State>>) {
        AppBuilder::new(state, view).start();
    }

    pub fn builder(
        state: State,
        view: fn() -> Node<'static, State, AppState<State>>,
    ) -> AppBuilder<State> {
        AppBuilder::new(state, view)
    }
    fn run(
        state: State,
        event_loop: EventLoop<()>,
        render_cx: RenderContext,
        #[cfg(target_arch = "wasm32")] render_state: RenderState,
        view: fn() -> Node<'static, State, AppState<State>>,
        on_frame: fn(&mut State, &mut AppState<State>) -> (),
        on_start: fn(&mut State, &mut AppState<State>) -> (),
        on_exit: fn(&mut State, &mut AppState<State>) -> (),
        inner_size: Option<(u32, u32)>,
        resizable: Option<bool>,
        title: Option<String>,
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
        let mut app = Self {
            context: render_cx,
            renderers,
            render_state,
            cached_window: None,
            window_inner_size: inner_size,
            window_resizable: resizable,
            window_title: title,
            state,
            view,

            app_state: AppState {
                cursor_position: None,
                gesture_state: GestureState::None,
                gesture_handlers: Vec::new(),
                runtime: Runtime::new().expect("Failed to create runtime"),
                cancellation_token: CancellationToken::new(),
                task_tracker: TaskTracker::new(),
                scale_factor: 1.,
                editor: None,
                view_state: HashMap::new(),
                animation_bank: AnimationBank::new(),
                scene: Scene::new(),
                layout_cx: LayoutContext::new(),
                font_cx,
                layout_cache: HashMap::new(),
                image_scenes: HashMap::new(),
                svg_scenes: HashMap::new(),
                modifiers: None,
                now: Instant::now(),
                appeared_views: std::collections::HashSet::new(),
            },
            on_frame,
            on_start,
            on_exit,
            started: false,
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

        self.app_state.now = Instant::now();
        self.app_state.gesture_handlers.clear();
        if let Self {
            context,
            render_state: Some(RenderState { surface, window }),
            ..
        } = self
        {
            self.app_state.scale_factor = window.scale_factor();
            let size = window.inner_size();
            let width = size.width;
            let height = size.height;
            if surface.config.width != width || surface.config.height != height {
                context.resize_surface(surface, width, height);
            }

            let view = self.view;
            Layout::new(view()).draw(
                Area {
                    x: 0.,
                    y: 0.,
                    width: ((width as f64) / self.app_state.scale_factor) as f32,
                    height: ((height as f64) / self.app_state.scale_factor) as f32,
                },
                &mut self.state,
                &mut self.app_state,
            );
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
        if self
            .app_state
            .animation_bank
            .in_progress(self.app_state.now)
            || self.app_state.animations_in_progress(self.app_state.now)
        {
            self.request_redraw();
        }
    }
}

impl<State: 'static> ApplicationHandler for App<'_, State> {
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
                .with_fullsize_content_view(true);

            #[cfg(not(target_os = "macos"))]
            let mut attributes = Window::default_attributes()
                .with_inner_size(LogicalSize::new(inner_size.0, inner_size.1))
                .with_resizable(resizable)
                .with_decorations(true)
                // .with_transparent(true)
                .with_maximized(true);

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
            vello_svg::vello::wgpu::PresentMode::AutoVsync,
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
        self.request_redraw();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
            std::time::Instant::now() + std::time::Duration::from_millis(500),
        ));
    }

    // fn new_events(
    //     &mut self,
    //     event_loop: &winit::event_loop::ActiveEventLoop,
    //     cause: winit::event::StartCause,
    // ) {
    //     if let StartCause::ResumeTimeReached { .. } = cause {
    //         self.request_redraw();
    //         event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
    //             std::time::Instant::now() + std::time::Duration::from_millis(500),
    //         ));
    //     }
    // }
    // }

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
                    let App {
                        app_state:
                            AppState {
                                editor,
                                layout_cx,
                                font_cx,
                                modifiers,
                                ..
                            },
                        ..
                    } = self;
                    let mut needs_redraw = false;
                    if let Some(EditState { editor, .. }) = editor {
                        editor.handle_key(key.clone(), layout_cx, font_cx, *modifiers);
                    }
                    for (id, _area, handler) in self.app_state.gesture_handlers.clone().iter() {
                        if let Some(ref interaction_handler) = handler.interaction_handler {
                            if handler.interaction_type.key {
                                needs_redraw = true;
                                interaction_handler(
                                    &mut self.state,
                                    &mut self.app_state,
                                    Interaction::Key(key.clone()),
                                );
                            } else if handler.interaction_type.edit {
                                needs_redraw = true;
                                if let Some(EditState {
                                    id: edit_id,
                                    editor,
                                    ..
                                }) = &self.app_state.editor.clone()
                                    && edit_id == id
                                {
                                    (interaction_handler)(
                                        &mut self.state,
                                        &mut self.app_state,
                                        Interaction::Edit(EditInteraction::Update(
                                            editor.text().to_string(),
                                        )),
                                    );
                                }
                            }
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
                    self.app_state.scale_factor = scale_factor;
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
            pos.x / self.app_state.scale_factor,
            pos.y / self.app_state.scale_factor,
        );
        let mut needs_redraw = false;
        self.app_state.cursor_position = Some(pos);
        let App {
            app_state:
                AppState {
                    editor,
                    font_cx,
                    layout_cx,
                    ..
                },
            ..
        } = self;
        if let Some(EditState { editor, area, .. }) = editor.as_mut() {
            needs_redraw = true;
            editor.mouse_moved(
                Point::new(pos.x - area.x as f64, pos.y - area.y as f64),
                layout_cx,
                font_cx,
            );
        }
        self.app_state
            .gesture_handlers
            .clone()
            .iter()
            .for_each(|(_, area, gh)| {
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
            self.app_state
                .gesture_handlers
                .clone()
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
            let App {
                app_state:
                    AppState {
                        editor,
                        font_cx,
                        layout_cx,
                        ..
                    },
                ..
            } = self;
            if let Some(EditState { editor, area, .. }) = editor.as_mut()
                && area_contains(area, point)
            {
                editor.mouse_pressed(layout_cx, font_cx);
            }

            if let Some((capturer, area, handler)) = self
                .app_state
                .gesture_handlers
                .clone()
                .iter()
                .rev()
                .find(|(_, area, handler)| {
                    area_contains(area, point)
                        && (handler.interaction_type.click || handler.interaction_type.drag)
                })
                .or(self.app_state.gesture_handlers.clone().iter().rev().find(
                    |(_, area, handler)| {
                        area_contains(
                            &Area {
                                x: area.x - 10.,
                                y: area.y - 10.,
                                width: area.width + 20.,
                                height: area.height + 20.,
                            },
                            point,
                        ) && (handler.interaction_type.click || handler.interaction_type.drag)
                    },
                ))
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
        }
        if needs_redraw {
            self.request_redraw();
        }
    }
    pub(crate) fn mouse_released(&mut self) {
        let mut needs_redraw = false;
        // if end_editing {
        //     self.editor = None;
        //     for handler in self.gesture_handlers.clone().unwrap().iter() {
        //         if let Some(ref interaction_handler) = handler.2.interaction_handler {
        //             if handler.2.interaction_type.edit {
        //                 needs_redraw = true;
        //                 self.with_ui(|ui| {
        //                     (interaction_handler)(ui, Interaction::Edit(EditInteraction::End));
        //                 });
        //             }
        //         }
        //     }
        // }
        if let Some(current) = self.app_state.cursor_position {
            if let Some(EditState {
                id, editor, area, ..
            }) = self.app_state.editor.as_mut()
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
                    self.app_state.end_editing(&mut self.state);
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
                self.app_state
                    .gesture_handlers
                    .clone()
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
        }
        self.app_state.gesture_state = GestureState::None;
        if needs_redraw {
            self.request_redraw();
        }
    }
    pub(crate) fn scrolled(&mut self, delta: MouseScrollDelta) {
        let mut needs_redraw = false;
        if let Some(current) = self.app_state.cursor_position
            && let Some((_, _, handler)) = self
                .app_state
                .gesture_handlers
                .clone()
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
