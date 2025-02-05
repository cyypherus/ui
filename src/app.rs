use crate::ui::{AnimationBank, Ui, UiCx};
use crate::{area_contains, ClickState, DragState, GestureHandler, Point, RcUi};
use crate::{event, ui::RenderState, Area, GestureState, Layout, RUBIK_FONT};
use backer::Node;
use parley::{FontContext, LayoutContext};
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;
use std::{num::NonZeroUsize, sync::Arc};
use vello_svg::vello::kurbo::{Affine, Vec2};
use vello_svg::vello::peniko::Color;
use vello_svg::vello::util::RenderContext;
use vello_svg::vello::{Renderer, RendererOptions, Scene};
use winit::{application::ApplicationHandler, event_loop::EventLoop, window::Window};
use winit::{dpi::LogicalSize, event::MouseButton};

pub struct App<'s, 'n, State: Clone> {
    state: State,
    view: Layout<'n, RcUi<State>>,
    pub(crate) cursor_position: Option<Point>,
    pub(crate) gesture_state: GestureState,
    pub gesture_handlers: Option<Vec<(u64, Area, GestureHandler<State>)>>,
    pub(crate) context: RenderContext,
    pub(crate) renderers: Vec<Option<Renderer>>,
    pub(crate) render_state: Option<RenderState<'s>>,
    pub(crate) cached_window: Option<Arc<Window>>,
    pub(crate) cx: Option<UiCx>,
    pub(crate) images: HashMap<u64, Vec<u8>>,
    pub(crate) image_scenes: HashMap<u64, (Scene, f32, f32)>,
}

impl<State: Clone> App<'_, '_, State> {
    fn request_redraw(&self) {
        let Some(RenderState { window, .. }) = &self.render_state else {
            return;
        };
        window.request_redraw();
    }
}

impl<'n, State: Clone> App<'_, 'n, State> {
    pub fn start(state: State, view: Node<'n, RcUi<State>>) {
        let event_loop = EventLoop::new().expect("Could not create event loop");
        #[allow(unused_mut)]
        let mut render_cx = RenderContext::new();
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::run(state, event_loop, render_cx, Layout::new(view));
        }
    }
    fn run(
        state: State,
        event_loop: EventLoop<()>,
        render_cx: RenderContext,
        #[cfg(target_arch = "wasm32")] render_state: RenderState,
        view: Layout<'n, RcUi<State>>,
    ) {
        #[allow(unused_mut)]
        let mut renderers: Vec<Option<Renderer>> = vec![];

        let mut font_cx = FontContext::new();
        font_cx.collection.register_fonts(RUBIK_FONT.to_vec());

        let render_state = None::<RenderState>;
        let mut app = Self {
            state,
            context: render_cx,
            renderers,
            render_state,
            cached_window: None,
            cursor_position: None,
            gesture_state: GestureState::None,
            cx: Some(crate::ui::UiCx {
                view_state: HashMap::new(),
                animation_bank: AnimationBank::new(),
                scene: Scene::new(),
                layout_cx: Rc::new(Cell::new(Some(LayoutContext::new()))),
                font_cx: Rc::new(Cell::new(Some(font_cx))),
            }),
            view,
            gesture_handlers: Some(Vec::new()),
            images: HashMap::new(),
            image_scenes: HashMap::new(),
        };
        event_loop.run_app(&mut app).expect("run to completion");
    }
    fn redraw(&mut self) {
        let now = Instant::now();
        self.gesture_handlers.as_mut().unwrap().clear();
        if let Self {
            context,
            render_state: Some(RenderState { surface, window }),
            ..
        } = self
        {
            let size = window.inner_size();
            let width = size.width;
            let height = size.height;
            if surface.config.width != width || surface.config.height != height {
                context.resize_surface(surface, width, height);
            }
            let mut ui = RcUi {
                ui: Ui {
                    state: self.state.clone(),
                    gesture_handlers: self.gesture_handlers.take().unwrap(),
                    cx: self.cx.take(),
                    images: HashMap::new(),
                    now,
                },
            };

            self.view.draw(
                Area {
                    x: 0.,
                    y: 0.,
                    width: surface.config.width as f32,
                    height: surface.config.height as f32,
                },
                &mut ui,
            );
            self.state = ui.ui.state.clone();
            self.gesture_handlers = Some(std::mem::take(&mut ui.ui.gesture_handlers));
            self.cx = ui.ui.cx.take();

            for (id, (area, source)) in ui.ui.images {
                if self.images.get_mut(&id).is_none() {
                    self.images.insert(id, source());
                }
                let image_data = self.images.get(&id).unwrap();
                if self.image_scenes.get(&id).is_none() {
                    match vello_svg::usvg::Tree::from_data(
                        image_data,
                        &vello_svg::usvg::Options::default(),
                    ) {
                        Err(err) => {
                            eprintln!("Loading svg failed: {err}");
                            self.image_scenes.insert(id, (Scene::new(), 0., 0.));
                        }
                        Ok(svg) => {
                            let svg_scene = vello_svg::render_tree(&svg);
                            let size = svg.size();
                            self.image_scenes
                                .insert(id, (svg_scene, size.width(), size.height()));
                        }
                    }
                }
                if let Some((svg_scene, width, height)) = self.image_scenes.get(&id) {
                    self.cx.as_mut().unwrap().scene.append(
                        svg_scene,
                        Some(
                            Affine::IDENTITY
                                .then_scale_non_uniform(
                                    (area.width / width) as f64,
                                    (area.height / height) as f64,
                                )
                                .then_translate(Vec2::new(area.x as f64, area.y as f64)),
                        ),
                    );
                }
            }
        }
        let Self {
            context,
            renderers,
            render_state: Some(RenderState { surface, window }),
            cx: Some(UiCx { scene, .. }),
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
        self.cx.as_mut().unwrap().scene.reset();
        if self.cx.as_mut().unwrap().animation_bank.in_progress(now)
            || self.cx.as_ref().unwrap().animations_in_progress(now)
        {
            self.request_redraw();
        }
    }
}

impl<State: Clone> ApplicationHandler for App<'_, '_, State> {
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
                    self.request_redraw();
                    let Some(key) = crate::Key::from(key) else {
                        return;
                    };
                    for handler in self.gesture_handlers.as_ref().unwrap().iter() {
                        if let Some(ref on_key) = handler.2.on_key {
                            (on_key)(&mut self.state, key.clone());
                        }
                    }
                }
                event::WindowEvent::KeyReleased(_) => {}
                event::WindowEvent::MouseMoved(pos) => {
                    self.request_redraw();
                    self.mouse_moved(pos)
                }
                event::WindowEvent::MousePressed(MouseButton::Left) => {
                    self.request_redraw();
                    self.mouse_pressed()
                }
                event::WindowEvent::MouseReleased(MouseButton::Left) => {
                    self.request_redraw();
                    self.mouse_released()
                }
                event::WindowEvent::MousePressed(_) => {}
                event::WindowEvent::MouseReleased(_) => {}
                event::WindowEvent::MouseEntered => {}
                event::WindowEvent::MouseExited => {}
                event::WindowEvent::MouseWheel(_, _) => {}
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
            }
        }
    }
}
impl<State: Clone> App<'_, '_, State> {
    pub(crate) fn mouse_moved(&mut self, pos: Point) {
        self.cursor_position = Some(pos);
        self.gesture_handlers
            .as_ref()
            .unwrap()
            .iter()
            .for_each(|(_, area, gh)| {
                if let Some(on_hover) = &gh.on_hover {
                    on_hover(&mut self.state, area_contains(area, pos));
                }
            });
        if let GestureState::Dragging { start, capturer } = self.gesture_state {
            let distance = start.distance(pos);
            if let Some(Some(handler)) = self
                .gesture_handlers
                .as_ref()
                .unwrap()
                .iter()
                .find(|(id, _, _)| *id == capturer)
                .map(|(_, _, gh)| &gh.on_drag)
            {
                handler(
                    &mut self.state,
                    DragState::Updated {
                        start,
                        current: pos,
                        distance,
                    },
                );
            }
        }
    }
    pub(crate) fn mouse_pressed(&mut self) {
        if let Some(point) = self.cursor_position {
            if let Some((capturer, _, handler)) = self
                .gesture_handlers
                .as_ref()
                .unwrap()
                .iter()
                .rev()
                .find(|(_, area, handler)| {
                    area_contains(area, point)
                        && (handler.on_click.is_some() || handler.on_drag.is_some())
                })
            {
                if let Some(ref on_click) = handler.on_click {
                    on_click(&mut self.state, ClickState::Started);
                }
                self.gesture_state = GestureState::Dragging {
                    start: point,
                    capturer: *capturer,
                }
            }
        }
    }
    pub(crate) fn mouse_released(&mut self) {
        if let Some(current) = self.cursor_position {
            if let GestureState::Dragging { start, capturer } = self.gesture_state {
                let distance = start.distance(current);
                if let Some((_, area, handler)) = self
                    .gesture_handlers
                    .as_ref()
                    .unwrap()
                    .iter()
                    .find(|(id, _, _)| *id == capturer)
                {
                    if let Some(ref on_click) = handler.on_click {
                        if area_contains(area, current) {
                            on_click(&mut self.state, ClickState::Completed);
                        } else {
                            on_click(&mut self.state, ClickState::Cancelled);
                        }
                    }
                    if let Some(ref on_drag) = handler.on_drag {
                        on_drag(
                            &mut self.state,
                            DragState::Completed {
                                start,
                                current,
                                distance,
                            },
                        );
                    }
                }
            }
        }
        self.gesture_state = GestureState::None;
    }
}
