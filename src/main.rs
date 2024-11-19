use backer::models::Area;
use backer::nodes::*;
use backer::transitions::{AnimationBank, TransitionDrawable, TransitionState};
use backer::{id, Layout, Node};
use parley::{FontContext, LayoutContext};
use rect::rect;
use std::any::Any;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;
use text_view::text;
use vello::peniko::Color;
use vello::util::{RenderContext, RenderSurface};
use vello::{Renderer, RendererOptions, Scene};
use view::{view, AnimatedView};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::MouseButton;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowAttributes};

mod event;
mod rect;
mod text_view;
mod view;

fn main() {
    App::start(
        UserState {
            hovered: false,
            depressed: false,
            toggle: false,
        },
        |ui| {
            column_spaced(
                10.,
                vec![
                    // view(
                    //     ui,
                    //     text(id!(), "Hello world")
                    //         .font_size(10)
                    //         .fill(Color::rgb(1., 1., 1.))
                    //         .finish()
                    //         .transition_duration(1700.),
                    // )
                    space()
                        .height(30.)
                        .width(100.)
                        .pad(15.)
                        .attach_under(view(
                            ui,
                            rect(id!())
                                .fill(Color::rgb(0.2, 0.15, 0.15))
                                .stroke(stroke(ui.state.toggle), stroke_width(ui.state.toggle))
                                .corner_rounding(rounding(ui.state.toggle, 30.))
                                .finish()
                                .transition_duration(1700.),
                        ))
                        .pad(10.)
                        .attach_under(view(
                            ui,
                            rect(id!())
                                .stroke(stroke(ui.state.toggle), stroke_width(ui.state.toggle))
                                .corner_rounding(rounding(ui.state.toggle, 30.))
                                .finish()
                                .transition_duration(1900.),
                        ))
                        .pad(10.)
                        .attach_under(view(
                            ui,
                            rect(id!())
                                .stroke(stroke(ui.state.toggle), stroke_width(ui.state.toggle))
                                .corner_rounding(rounding(ui.state.toggle, 30.))
                                .finish()
                                .transition_duration(2100.),
                        ))
                        .visible(ui.state.toggle)
                        .pad(10.)
                        .attach_under(view(
                            ui,
                            rect(id!())
                                .stroke(stroke(ui.state.toggle), stroke_width(ui.state.toggle))
                                .corner_rounding(rounding(ui.state.toggle, 30.))
                                .finish()
                                .transition_duration(2300.),
                        ))
                        .visible(ui.state.toggle)
                        .pad(10.)
                        .attach_under(view(
                            ui,
                            rect(id!())
                                .stroke(stroke(ui.state.toggle), stroke_width(ui.state.toggle))
                                .corner_rounding(rounding(ui.state.toggle, 30.))
                                .finish()
                                .transition_duration(2500.),
                        ))
                        .visible(ui.state.toggle),
                    space().height(150.).width(0.).visible(!ui.state.toggle),
                    view(
                        ui,
                        rect(id!())
                            .stroke(Color::rgb(0.4, 0.4, 0.4), 4.)
                            .fill(match (ui.state.hovered, ui.state.depressed) {
                                (_, true) => Color::rgb(0.2, 0.2, 0.2),
                                (true, false) => Color::rgb(0.3, 0.3, 0.3),
                                (false, false) => Color::rgb(0.1, 0.1, 0.1),
                            })
                            .corner_rounding(20.)
                            .finish()
                            .on_hover(|state: &mut UserState, hovered| state.hovered = hovered)
                            .on_click(|state: &mut UserState, click_state| match click_state {
                                ClickState::Started => state.depressed = true,
                                ClickState::Cancelled => state.depressed = false,
                                ClickState::Completed => {
                                    state.depressed = false;
                                    state.toggle = !state.toggle;
                                }
                            }),
                    )
                    .width(100.)
                    .height(40.),
                ],
            )
        },
    )
}

fn stroke(toggle: bool) -> Color {
    if toggle {
        Color::rgb(1., 0.2, 0.2)
    } else {
        Color::rgb(1., 0.2, 1.)
    }
}

fn rounding(toggle: bool, basis: f32) -> f32 {
    if toggle {
        0.
    } else {
        basis
    }
}

fn stroke_width(toggle: bool) -> f32 {
    if toggle {
        4.
    } else {
        0.
    }
}

struct UserState {
    hovered: bool,
    depressed: bool,
    toggle: bool,
}

struct RenderState<'s> {
    // SAFETY: We MUST drop the surface before the `window`, so the fields
    // must be in this order
    surface: RenderSurface<'s>,
    window: Arc<Window>,
}

struct App<'s, State> {
    ui: Ui<'s, State>,
    view: Layout<Ui<'s, State>>,
}

struct Ui<'s, State> {
    context: RenderContext,
    renderers: Vec<Option<Renderer>>,
    render_state: Option<RenderState<'s>>,
    cached_window: Option<Arc<Window>>,
    state: State,
    animation_bank: AnimationBank,
    // system_state: HashMap<u64, Box<dyn Any>>,
    view_state: HashMap<u64, AnimatedView>,
    gesture_handlers: Vec<(u64, Area, GestureHandler<State>)>,
    cursor_position: Option<Point>,
    gesture_state: GestureState,
    scene: Scene,
    font_cx: FontContext,
    layout_cx: LayoutContext,
}

#[derive(Debug, Clone, Copy)]
enum GestureState {
    None,
    Dragging { start: Point, capturer: u64 },
}

type ClickHandler<State> = Option<Box<dyn Fn(&mut State, ClickState)>>;
type DragHandler<State> = Option<Box<dyn Fn(&mut State, DragState)>>;
type HoverHandler<State> = Option<Box<dyn Fn(&mut State, bool)>>;

struct GestureHandler<State> {
    on_click: ClickHandler<State>,
    on_drag: DragHandler<State>,
    on_hover: HoverHandler<State>,
}

#[derive(Debug, Clone, Copy)]
pub enum DragState {
    Began(Point),
    Updated {
        start: Point,
        current: Point,
        distance: f32,
    },
    Completed {
        start: Point,
        current: Point,
        distance: f32,
    },
}

#[derive(Debug, Clone, Copy)]
enum ClickState {
    Started,
    Cancelled,
    Completed,
}

impl<'s, State> ApplicationHandler for App<'s, State> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let Option::None = self.ui.render_state else {
            return;
        };
        let window = self.ui.cached_window.take().unwrap_or_else(|| {
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
        let surface_future = self.ui.context.create_surface(
            window.clone(),
            size.width,
            size.height,
            vello::wgpu::PresentMode::Immediate,
        );
        let surface = pollster::block_on(surface_future).expect("Error creating surface");
        self.ui.render_state = {
            let render_state = RenderState { window, surface };
            self.ui
                .renderers
                .resize_with(self.ui.context.devices.len(), || None);
            let id = render_state.surface.dev_id;
            self.ui.renderers[id].get_or_insert_with(|| {
                #[allow(unused_mut)]
                let mut renderer = Renderer::new(
                    &self.ui.context.devices[id].device,
                    RendererOptions {
                        surface_format: Some(render_state.surface.format),
                        use_cpu: false,
                        antialiasing_support: vello::AaSupport {
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
        }
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
                event::WindowEvent::KeyPressed(_) => {}
                event::WindowEvent::KeyReleased(_) => {}
                event::WindowEvent::MouseMoved(pos) => self.ui.mouse_moved(pos),
                event::WindowEvent::MousePressed(MouseButton::Left) => self.ui.mouse_pressed(),
                event::WindowEvent::MousePressed(_) => {}
                event::WindowEvent::MouseReleased(MouseButton::Left) => self.ui.mouse_released(),
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
                    let Some(RenderState { window, .. }) = &self.ui.render_state else {
                        return;
                    };
                    window.request_redraw();
                }
                event::WindowEvent::Unfocused => {}
                event::WindowEvent::Closed => event_loop.exit(),
                event::WindowEvent::RedrawRequested => self.redraw(),
            }
        }
    }
}
impl<'s, State> Ui<'s, State> {
    fn mouse_moved(&mut self, pos: Point) {
        self.cursor_position = Some(pos);
        self.gesture_handlers.iter().for_each(|(_, area, gh)| {
            if let Some(on_hover) = &gh.on_hover {
                on_hover(&mut self.state, area_contains(area, pos));
            }
        });
        if let GestureState::Dragging { start, capturer } = self.gesture_state {
            let distance = start.distance(pos);
            if let Some(Some(handler)) = self
                .gesture_handlers
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
    fn mouse_pressed(&mut self) {
        if let Some(point) = self.cursor_position {
            if let Some((capturer, _, handler)) =
                self.gesture_handlers
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
    fn mouse_released(&mut self) {
        if let Some(current) = self.cursor_position {
            if let GestureState::Dragging { start, capturer } = self.gesture_state {
                let distance = start.distance(current);
                if let Some((_, area, handler)) = self
                    .gesture_handlers
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
impl<'s, State> App<'s, State> {
    pub fn start(state: State, view: impl Fn(&mut Ui<State>) -> Node<Ui<'s, State>> + 'static) {
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
        view: Layout<Ui<'s, State>>,
    ) {
        #[allow(unused_mut)]
        let mut renderers: Vec<Option<Renderer>> = vec![];

        let mut font_cx = FontContext::new();
        font_cx.collection.register_fonts(RUBIK_FONT.to_vec());

        let render_state = None::<RenderState>;
        let mut app = Self {
            ui: Ui {
                context: render_cx,
                renderers,
                render_state,
                cached_window: None,
                state,
                animation_bank: AnimationBank::new(),
                view_state: HashMap::new(),
                gesture_handlers: Vec::new(),
                cursor_position: None,
                gesture_state: GestureState::None,
                scene: Scene::new(),
                layout_cx: LayoutContext::new(),
                font_cx,
            },
            view,
        };
        event_loop.run_app(&mut app).expect("run to completion");
    }
    fn redraw(&mut self) {
        if let Some(RenderState { surface, window }) = &mut self.ui.render_state {
            let size = window.inner_size();
            let width = size.width;
            let height = size.height;
            if surface.config.width != width || surface.config.height != height {
                self.ui.context.resize_surface(surface, width, height);
            }
            self.ui.gesture_handlers.clear();
            self.view.draw(
                Area {
                    x: 0.,
                    y: 0.,
                    width: surface.config.width as f32,
                    height: surface.config.height as f32,
                },
                &mut self.ui,
            );
        }
        let Some(RenderState { surface, window }) = &mut self.ui.render_state else {
            return;
        };
        window.request_redraw();

        let size = window.inner_size();
        let width = size.width;
        let height = size.height;

        let device_handle = &self.ui.context.devices[surface.dev_id];

        window.set_title("haven-ui");

        let render_params = vello::RenderParams {
            base_color: Color::BLACK,
            width,
            height,
            antialiasing_method: vello::AaConfig::Area,
        };

        let surface_texture = surface
            .surface
            .get_current_texture()
            .expect("failed to get surface texture");

        window.pre_present_notify();

        self.ui.renderers[surface.dev_id]
            .as_mut()
            .unwrap()
            .render_to_surface(
                &device_handle.device,
                &device_handle.queue,
                &self.ui.scene,
                &surface_texture,
                &render_params,
            )
            .expect("failed to render to surface");

        surface_texture.present();
        device_handle.device.poll(vello::wgpu::Maintain::Wait);
        self.ui.scene.reset();
    }
}
const RUBIK_FONT: &[u8] = include_bytes!("../assets/Rubik-VariableFont_wght.ttf");

fn area_contains(area: &Area, point: Point) -> bool {
    let x = point.x;
    let y = point.y;
    if x > area.x && y > area.y && y < area.y + area.height && x < area.x + area.width {
        return true;
    }
    false
}

impl Point {
    fn distance(&self, to: Point) -> f32 {
        ((to.x - self.x).powf(2.) + (to.y - self.y).powf(2.)).sqrt()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    x: f32,
    y: f32,
}

trait ViewTrait<'s, State>: TransitionDrawable<Ui<'s, State>> + Sized {
    fn view(self, ui: &mut Ui<State>, node: Node<Ui<'s, State>>) -> Node<Ui<'s, State>>;
}

impl<'s, State> TransitionState for Ui<'s, State> {
    fn bank(&mut self) -> &mut AnimationBank {
        &mut self.animation_bank
    }
}
