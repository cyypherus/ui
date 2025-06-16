use crate::gestures::{ClickLocation, EditInteraction, Interaction, ScrollDelta};
use crate::ui::AnimationBank;
use crate::view::AnimatedView;
use crate::{area_contains, ClickState, DragState, Editor, GestureHandler, Point};
use crate::{event, Area, GestureState, RUBIK_FONT};
use backer::{Layout, Node};
use parley::fontique::Blob;
use parley::{FontContext, LayoutContext};
use std::collections::HashMap;
use std::time::Instant;
use std::{num::NonZeroUsize, sync::Arc};
use vello_svg::vello::peniko::{Brush, Color};
use vello_svg::vello::util::{RenderContext, RenderSurface};
use vello_svg::vello::{Renderer, RendererOptions, Scene};
use winit::event::{Modifiers, MouseScrollDelta};
use winit::{application::ApplicationHandler, event_loop::EventLoop, window::Window};
use winit::{dpi::LogicalSize, event::MouseButton};

pub struct App<'s, State> {
    pub(crate) context: RenderContext,
    pub(crate) renderers: Vec<Option<Renderer>>,
    pub(crate) render_state: Option<RenderState<'s>>,
    pub(crate) cached_window: Option<Arc<Window>>,
    pub(crate) app_state: AppState,
    pub state: State,
    pub(crate) view: fn() -> Node<'static, State, AppState>,
}

pub(crate) struct RenderState<'surface> {
    // SAFETY: We MUST drop the surface before the `window`, so the fields
    // must be in this order
    pub(crate) surface: RenderSurface<'surface>,
    pub(crate) window: Arc<Window>,
}

type TextLayoutCache = HashMap<u64, Vec<(String, f32, parley::Layout<Brush>)>>;
pub struct AppState {
    pub(crate) cursor_position: Option<Point>,
    pub(crate) gesture_state: GestureState,
    pub gesture_handlers: Vec<(u64, Area, GestureHandler<State, Self>)>,
    // pub(crate) background_scheduler: BackgroundScheduler<State>,
    pub(crate) scale_factor: f64,
    pub(crate) editor: Option<(u64, Area, Editor, bool)>,
    pub(crate) animation_bank: AnimationBank,
    pub(crate) scene: Scene,
    pub(crate) font_cx: FontContext,
    pub(crate) layout_cx: LayoutContext<Brush>,
    pub(crate) view_state: HashMap<u64, AnimatedView>,
    pub(crate) layout_cache: TextLayoutCache,
    pub(crate) image_scenes: HashMap<String, (Scene, f32, f32)>,
    pub(crate) modifiers: Option<Modifiers>,
    pub(crate) now: Instant,
}

impl AppState {
    pub(crate) fn animations_in_progress(&self, now: Instant) -> bool {
        self.view_state.values().any(|v| match v {
            AnimatedView::Rect(animated_rect) => animated_rect.shape.in_progress(now),
            AnimatedView::Text(animated_text) => animated_text.fill.in_progress(now),
            AnimatedView::Circle(animated_circle) => animated_circle.shape.in_progress(now),
        })
    }
}

use std::thread;
use tokio::{runtime::Runtime, sync::mpsc, task};

type BackgroundTask<State> = Box<dyn FnOnce() -> BackgroundTaskCompletion<State> + Send + 'static>;
type BackgroundTaskCompletion<State> = Box<dyn FnOnce(&mut State) + Send>;

pub struct BackgroundScheduler<State> {
    sender: mpsc::Sender<BackgroundTask<State>>,
}

impl<State: 'static> BackgroundScheduler<State> {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel::<BackgroundTask<State>>(100);
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                while let Some(task) = rx.recv().await {
                    task::spawn_blocking(move || task());
                }
            });
        });
        Self { sender: tx }
    }
}

impl<'n, State: Clone + 'static> App<'_, State> {
    // pub fn spawn<F, R>(&self, task: F)
    // where
    //     F: FnOnce() -> R + Send + 'static,
    //     R: FnOnce(&mut State) + Send + 'static,
    // {
    //     _ = self
    //         .app_state
    //         .background_scheduler
    //         .sender
    //         .blocking_send(Box::new(|| Box::new(task())));
    // }
    fn request_redraw(&self) {
        let Some(RenderState { window, .. }) = &self.render_state else {
            return;
        };
        window.request_redraw();
    }
    pub fn start(state: State, view: fn() -> Node<'static, State, AppState>) {
        let event_loop = EventLoop::new().expect("Could not create event loop");
        #[allow(unused_mut)]
        let mut render_cx = RenderContext::new();
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::run(state, event_loop, render_cx, view);
        }
    }
    fn run(
        state: State,
        event_loop: EventLoop<()>,
        render_cx: RenderContext,
        #[cfg(target_arch = "wasm32")] render_state: RenderState,
        view: fn() -> Node<'static, State, AppState>,
    ) {
        #[allow(unused_mut)]
        let mut renderers: Vec<Option<Renderer>> = vec![];

        let mut font_cx = FontContext::new();

        font_cx
            .collection
            .register_fonts(Blob::new(Arc::new(RUBIK_FONT)), None);

        let render_state = None::<RenderState>;
        let mut app = Self {
            context: render_cx,
            renderers,
            render_state,
            cached_window: None,
            state,
            view,
            app_state: AppState {
                cursor_position: None,
                gesture_state: GestureState::None,
                gesture_handlers: Vec::new(),
                // background_scheduler: BackgroundScheduler::new(),
                scale_factor: 1.,
                editor: None,
                view_state: HashMap::new(),
                animation_bank: AnimationBank::new(),
                scene: Scene::new(),
                layout_cx: LayoutContext::new(),
                font_cx,
                layout_cache: HashMap::new(),
                image_scenes: HashMap::new(),
                modifiers: None,
                now: Instant::now(),
            },
        };
        event_loop.run_app(&mut app).expect("run to completion");
    }

    fn redraw(&mut self) {
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
                    width: width as f32,
                    height: height as f32,
                },
                &mut self.state,
                &mut self.app_state,
            );
            if let Some((_, area, ref mut editor, _)) = self.app_state.editor {
                editor.draw(
                    area,
                    &mut self.app_state.scene,
                    &mut self.app_state.layout_cx,
                    &mut self.app_state.font_cx,
                    true,
                    1.0,
                );
            }
        }
        let Self {
            context,
            renderers,
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

        window.set_title("haven-ui");

        let render_params = vello_svg::vello::RenderParams {
            base_color: Color::BLACK,
            width,
            height,
            antialiasing_method: vello_svg::vello::AaConfig::Area,
        };

        let surface_texture = surface
            .surface
            .get_current_texture()
            .expect("failed to get surface texture");

        window.pre_present_notify();

        renderers[surface.dev_id]
            .as_mut()
            .unwrap()
            .render_to_surface(
                &device_handle.device,
                &device_handle.queue,
                scene,
                &surface_texture,
                &render_params,
            )
            .expect("failed to render to surface");

        surface_texture.present();
        device_handle
            .device
            .poll(vello_svg::vello::wgpu::Maintain::Wait);
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

impl<State: Clone + 'static> ApplicationHandler for App<'_, State> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let Option::None = self.render_state else {
            return;
        };
        let window = self.cached_window.take().unwrap_or_else(|| {
            Arc::new(
                event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_inner_size(LogicalSize::new(1044, 800))
                            .with_resizable(true),
                    )
                    .unwrap(),
            )
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
                let mut renderer = Renderer::new(
                    &context.devices[id].device,
                    RendererOptions {
                        surface_format: Some(render_state.surface.format),
                        use_cpu: false,
                        antialiasing_support: vello_svg::vello::AaSupport {
                            area: true,
                            msaa8: false,
                            msaa16: false,
                        },
                        num_init_threads: NonZeroUsize::new(1),
                    },
                )
                .expect("Failed to create renderer");
                renderer
            });
            Some(render_state)
        };
        self.render_state = render_state;
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
                    if let Some((_, _, editor, _)) = editor {
                        editor.handle_key(key.clone(), layout_cx, font_cx, modifiers.clone());
                    }
                    for handler in self.app_state.gesture_handlers.clone().iter() {
                        if let Some(ref interaction_handler) = handler.2.interaction_handler {
                            if handler.2.interaction_type.key {
                                needs_redraw = true;
                                interaction_handler(
                                    &mut self.app_state,
                                    Interaction::Key(key.clone()),
                                );
                            } else if handler.2.interaction_type.edit {
                                needs_redraw = true;
                                if let Some((_, _, editor, _)) = &self.app_state.editor.clone() {
                                    (interaction_handler)(
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
                    self.request_redraw();
                }
                event::WindowEvent::ModifiersChanged(modifiers) => {
                    self.app_state.modifiers = Some(modifiers);
                }
            }
        }
    }
}
impl<State: Clone + 'static> App<'_, State> {
    pub(crate) fn mouse_moved(&mut self, pos: Point) {
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
        if let Some((_, area, editor, _)) = editor.as_mut() {
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
                if gh.interaction_type.hover {
                    if let Some(ref on_hover) = gh.interaction_handler {
                        needs_redraw = true;
                        (on_hover)(
                            &mut self.app_state,
                            Interaction::Hover(area_contains(area, pos)),
                        );
                    }
                }
            });
        if let GestureState::Dragging { start, capturer } = self.app_state.gesture_state {
            let distance = start.distance(pos);
            self.app_state
                .gesture_handlers
                .clone()
                .iter()
                .filter(|(id, _, gh)| *id == capturer && gh.interaction_type.drag)
                .for_each(|(_, _, gh)| {
                    needs_redraw = true;
                    if let Some(handler) = &gh.interaction_handler {
                        (handler)(
                            &mut self.app_state,
                            Interaction::Drag(DragState::Updated {
                                start,
                                current: pos,
                                distance: distance as f32,
                            }),
                        );
                    }
                });
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
            if let Some((_, area, editor, _)) = editor.as_mut() {
                if area_contains(area, point) {
                    editor.mouse_pressed(layout_cx, font_cx);
                }
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
            {
                needs_redraw = true;
                if let Some(ref on_click) = handler.interaction_handler {
                    on_click(
                        &mut self.app_state,
                        Interaction::Click(ClickState::Started, ClickLocation::new(point, *area)),
                    );
                }
                self.app_state.gesture_state = GestureState::Dragging {
                    start: point,
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
        if let Some((_, _, editor, _)) = self.app_state.editor.as_mut() {
            needs_redraw = true;
            editor.mouse_released();
        }
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
            if let GestureState::Dragging { start, capturer } = self.app_state.gesture_state {
                let distance = start.distance(current);
                self.app_state
                    .gesture_handlers
                    .clone()
                    .iter()
                    .filter(|(id, _, _)| *id == capturer)
                    .for_each(|(_, area, gh)| {
                        if let (Some(ref on_click), true) =
                            (&gh.interaction_handler, gh.interaction_type.click)
                        {
                            needs_redraw = true;
                            if area_contains(area, current) {
                                on_click(
                                    &mut self.app_state,
                                    Interaction::Click(
                                        ClickState::Completed,
                                        ClickLocation::new(current, *area),
                                    ),
                                );
                            } else {
                                on_click(
                                    &mut self.app_state,
                                    Interaction::Click(
                                        ClickState::Cancelled,
                                        ClickLocation::new(current, *area),
                                    ),
                                );
                            }
                        }
                        if let (Some(ref on_drag), true) =
                            (&gh.interaction_handler, gh.interaction_type.drag)
                        {
                            needs_redraw = true;
                            on_drag(
                                &mut self.app_state,
                                Interaction::Drag(DragState::Completed {
                                    start,
                                    current,
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
        if let Some(current) = self.app_state.cursor_position {
            if let Some((_, _, handler)) = self
                .app_state
                .gesture_handlers
                .clone()
                .iter()
                .rev()
                .find(|(_, area, handler)| {
                    area_contains(area, current) && (handler.interaction_type.scroll)
                })
            {
                if let Some(ref on_scroll) = handler.interaction_handler {
                    needs_redraw = true;
                    on_scroll(
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
            }
        }
        if needs_redraw {
            self.request_redraw();
        }
    }
}
