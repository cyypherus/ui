use backer::models::Area;
use backer::nodes::*;
use backer::transitions::{AnimationBank, TransitionDrawable, TransitionState};
use backer::{id, Layout, Node};
use std::any::Any;
use std::collections::HashMap;
use std::future;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::Arc;
use vello::kurbo::Affine;
use vello::low_level::{BumpAllocators, DebugLayers};
use vello::peniko::Color;
use winit::dpi::LogicalSize;
use winit::event::MouseButton;
use winit::event_loop::EventLoop;
// use text_view::{text, Text};
use vello::util::{RenderContext, RenderSurface};
use vello::{AaConfig, Renderer, RendererOptions, Scene};
use winit::application::ApplicationHandler;
use winit::window::{Window, WindowAttributes};

mod event;
mod text_view;

fn main() {
    App::start(
        UserState {
            hovered: false,
            depressed: false,
            toggle: false,
        },
        |ui| column(vec![space(), view(ui, rect(id!()).finish())]),
    );
}

// fn main() {
//     App::start(
//         UserState {
//             hovered: false,
//             depressed: false,
//             toggle: false,
//         },
//         |ui| {
//             column_spaced(
//                 10.,
//                 vec![
//                     view(
//                         ui,
//                         text(id!(), "Hello world")
//                             .font_size(10)
//                             .fill(srgb(1., 1., 1.))
//                             .finish()
//                             .transition_duration(300.),
//                     )
//                     .pad(15.)
//                     .attach_under(view(
//                         ui,
//                         rect(id!())
//                             .fill(srgb(0.2, 0.15, 0.15))
//                             .stroke(srgb(0.3, 0.2, 0.2), 2.)
//                             .corner_rounding(0.3)
//                             .finish()
//                             .transition_duration(200.),
//                     ))
//                     .pad(5.)
//                     .attach_under(view(
//                         ui,
//                         rect(id!())
//                             .stroke(srgb(0.3, 0.2, 0.2), 2.)
//                             .corner_rounding(0.3)
//                             .finish()
//                             .transition_duration(400.),
//                     ))
//                     .pad(5.)
//                     .attach_under(view(
//                         ui,
//                         rect(id!())
//                             .stroke(srgb(0.3, 0.2, 0.2), 2.)
//                             .corner_rounding(0.3)
//                             .finish()
//                             .transition_duration(600.),
//                     ))
//                     .visible(ui.state.toggle),
//                     view(
//                         ui,
//                         rect(id!())
//                             .stroke(srgb(0.4, 0.4, 0.4), 1.)
//                             .fill(match (ui.state.hovered, ui.state.depressed) {
//                                 (_, true) => srgb(0.2, 0.2, 0.2),
//                                 (true, false) => srgb(0.3, 0.3, 0.3),
//                                 (false, false) => srgb(0.1, 0.1, 0.1),
//                             })
//                             .corner_rounding(0.2)
//                             .finish()
//                             .on_hover(|state: &mut UserState, hovered| state.hovered = hovered)
//                             .on_click(|state: &mut UserState, click_state| match click_state {
//                                 ClickState::Started => state.depressed = true,
//                                 ClickState::Cancelled => state.depressed = false,
//                                 ClickState::Completed => {
//                                     state.depressed = false;
//                                     state.toggle = !state.toggle;
//                                 }
//                             }),
//                     )
//                     .width(100.)
//                     .height(40.),
//                 ],
//             )
//         },
//     )
// }

fn view<'s, State: 'static>(ui: &mut Ui<State>, view: View<State>) -> Node<Ui<'s, State>> {
    view.view(ui)
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
    view_state: HashMap<u64, Box<dyn Any>>,
    gesture_handlers: Vec<(u64, Area, GestureHandler<State>)>,
    cursor_position: Option<Point>,
    gesture_state: GestureState,
    scene: Scene,
}

struct View<State> {
    view_type: ViewType,
    gesture_handler: GestureHandler<State>,
    easing: Option<backer::Easing>,
    duration: Option<f32>,
    delay: f32,
}

#[derive(Debug, Clone, Copy)]
enum GestureState {
    None,
    Dragging { start: Point, capturer: u64 },
}

struct GestureHandler<State> {
    on_click: Option<Box<dyn Fn(&mut State, ClickState)>>,
    on_drag: Option<Box<dyn Fn(&mut State, DragState)>>,
    on_hover: Option<Box<dyn Fn(&mut State, bool)>>,
}

#[derive(Debug, Clone, Copy)]
enum DragState {
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

impl<State> View<State> {
    fn on_click(mut self, f: impl Fn(&mut State, ClickState) + 'static) -> View<State> {
        self.gesture_handler.on_click = Some(Box::new(f));
        self
    }
    fn on_drag(mut self, f: impl Fn(&mut State, DragState) + 'static) -> View<State> {
        self.gesture_handler.on_drag = Some(Box::new(f));
        self
    }
    fn on_hover(mut self, f: impl Fn(&mut State, bool) + 'static) -> View<State> {
        self.gesture_handler.on_hover = Some(Box::new(f));
        self
    }
    fn easing(mut self, easing: backer::Easing) -> Self {
        self.easing = Some(easing);
        self
    }
    fn transition_duration(mut self, duration_ms: f32) -> Self {
        self.duration = Some(duration_ms);
        self
    }
    fn transition_delay(mut self, delay_ms: f32) -> Self {
        self.delay = delay_ms;
        self
    }
}

impl<State: 'static> View<State> {
    fn view<'s>(self, ui: &mut Ui<State>) -> Node<Ui<'s, State>> {
        match self.view_type.clone() {
            // ViewType::Text(view) => view.view(ui, draw_object(self)),
            ViewType::Rect(view) => view.view(ui, draw_object(self)),
        }
    }
}

impl<'s, State> TransitionDrawable<Ui<'s, State>> for View<State> {
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
        state.gesture_handlers.push((
            *self.id(),
            area,
            GestureHandler {
                on_click: self.gesture_handler.on_click.take(),
                on_drag: self.gesture_handler.on_drag.take(),
                on_hover: self.gesture_handler.on_hover.take(),
            },
        ));
        match &mut self.view_type {
            // ViewType::Text(view) => view.draw_interpolated(area, state, visible, visible_amount),
            ViewType::Rect(view) => view.draw_interpolated(area, state, visible, visible_amount),
        }
    }

    fn id(&self) -> &u64 {
        match &self.view_type {
            // ViewType::Text(view) => <Text as TransitionDrawable<Ui<State>>>::id(view),
            ViewType::Rect(view) => <Rect as TransitionDrawable<Ui<State>>>::id(view),
        }
    }

    fn easing(&self) -> backer::Easing {
        self.easing.unwrap_or(backer::Easing::EaseOut)
    }

    fn duration(&self) -> f32 {
        self.duration.unwrap_or(100.)
    }
    fn delay(&self) -> f32 {
        self.delay
    }
}

#[derive(Debug, Clone)]
enum ViewType {
    // Text(Text),
    Rect(Rect),
}

fn window_attributes() -> WindowAttributes {
    Window::default_attributes()
        .with_inner_size(LogicalSize::new(1044, 800))
        .with_resizable(true)
        .with_title("????")
}

const AA_CONFIGS: [AaConfig; 3] = [AaConfig::Area, AaConfig::Msaa8, AaConfig::Msaa16];

impl<'s, State> ApplicationHandler for App<'s, State> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let Option::None = self.ui.render_state else {
            return;
        };
        let window =
            self.ui.cached_window.take().unwrap_or_else(|| {
                Arc::new(event_loop.create_window(window_attributes()).unwrap())
            });
        let size = window.inner_size();
        let surface_future = self.ui.context.create_surface(
            window.clone(),
            size.width,
            size.height,
            vello::wgpu::PresentMode::AutoNoVsync,
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
                        antialiasing_support: AA_CONFIGS.iter().copied().collect(),
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
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some(event) = crate::event::WindowEvent::from_winit_window_event(event) {
            match event {
                event::WindowEvent::Moved(_) => {}
                event::WindowEvent::KeyPressed(_) => {}
                event::WindowEvent::KeyReleased(_) => {}
                event::WindowEvent::ReceivedCharacter(_) => {}
                event::WindowEvent::MouseMoved(pos) => self.ui.mouse_moved(pos),
                event::WindowEvent::MousePressed(MouseButton::Left) => self.ui.mouse_pressed(),
                event::WindowEvent::MousePressed(_) => {}
                event::WindowEvent::MouseReleased(MouseButton::Left) => self.ui.mouse_released(),
                event::WindowEvent::MouseReleased(_) => {}
                event::WindowEvent::MouseEntered => {}
                event::WindowEvent::MouseExited => {}
                event::WindowEvent::MouseWheel(_, _) => {}
                event::WindowEvent::Resized(size) => {
                    if let Some(RenderState { surface, window }) = &mut self.ui.render_state {
                        self.ui
                            .context
                            .resize_surface(surface, size.x as u32, size.y as u32);
                        window.request_redraw();
                    };
                }
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
                event::WindowEvent::RedrawRequested => {
                    if let Some((width, height)) = &self
                        .ui
                        .render_state
                        .as_ref()
                        .map(|r| (r.surface.config.width, r.surface.config.height))
                    {
                        self.view.draw(
                            Area {
                                x: 0.,
                                y: 0.,
                                width: *width as f32,
                                height: *height as f32,
                            },
                            &mut self.ui,
                        );
                    }
                    let Some(RenderState { surface, window }) = &self.ui.render_state else {
                        return;
                    };
                    window.request_redraw();

                    let width = surface.config.width;
                    let height = surface.config.height;
                    let device_handle = &self.ui.context.devices[surface.dev_id];

                    window.set_title("haven-ui");

                    let render_params = vello::RenderParams {
                        base_color: Color::BLACK,
                        width,
                        height,
                        antialiasing_method: vello::AaConfig::Msaa8,
                    };

                    let surface_texture = surface
                        .surface
                        .get_current_texture()
                        .expect("failed to get surface texture");

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
                    self.ui.scene.reset();
                }
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

        #[cfg(not(target_arch = "wasm32"))]
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
            },
            view,
        };
        event_loop.run_app(&mut app).expect("run to completion");
    }
}

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
struct Point {
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

#[derive(Debug, Clone)]
struct Rect {
    id: u64,
    // fill: Option<Srgb<f32>>,
    rounding: f32,
    // stroke: Option<(Srgb<f32>, f32)>,
    easing: Option<backer::Easing>,
    duration: Option<f32>,
    delay: f32,
}

fn rect(id: String) -> Rect {
    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    Rect {
        id: hasher.finish(),
        // fill: None,
        rounding: 0.,
        // stroke: None,
        easing: None,
        duration: None,
        delay: 0.,
    }
}

impl Rect {
    // fn fill(mut self, color: Srgb<f32>) -> Self {
    //     self.fill = Some(color);
    //     self
    // }
    fn corner_rounding(mut self, radius: f32) -> Self {
        self.rounding = radius;
        self
    }
    // fn stroke(mut self, color: Srgb<f32>, line_width: f32) -> Self {
    //     self.stroke = Some((color, line_width));
    //     self
    // }
    fn finish<State>(self) -> View<State> {
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
        state.scene.draw_blurred_rounded_rect(
            Affine::IDENTITY,
            vello::kurbo::Rect {
                x0: area.x as f64,
                y0: area.y as f64,
                x1: (area.x + area.width) as f64,
                y1: (area.y + area.height) as f64,
            },
            Color::rgba(1., 0., 0., 1.),
            10.,
            0.1,
        );
        // let area = ui_to_draw(area, state.window_size);
        // fn generate_squircle(
        //     x: f32,
        //     y: f32,
        //     width: f32,
        //     height: f32,
        //     radius: f32,
        // ) -> impl Iterator<Item = (f32, f32)> {
        //     let a = width / 2.0;
        //     let b = height / 2.0;
        //     let aspect = width / height;
        //     let x_exponent = 1.0 / radius;
        //     let y_exponent = (1.0 / radius) * aspect;
        //     let steps = (((width + height) * 2.) / 10.) as usize;
        //     (0..steps).map(move |i| {
        //         let t = (i as f32 / steps as f32) * std::f32::consts::TAU;
        //         let cos_t = t.cos();
        //         let sin_t = t.sin();
        //         let px = x + a * cos_t.signum() * cos_t.abs().powf(x_exponent);
        //         let py = y + b * sin_t.signum() * sin_t.abs().powf(y_exponent);

        //         (px, py)
        //     })
        // }

        // let points = generate_squircle(area.x, area.y, area.width, area.height, 1. / self.rounding);
        // let polygon = state.draw.polygon();
        // if self.stroke.is_none() && self.fill.is_none() {
        //     polygon
        //         .color(srgba(0., 0., 0., visible_amount))
        //         .points(points)
        //         .finish();
        // } else {
        //     polygon
        //         .stroke_opts(if let Some((_, width)) = self.stroke {
        //             StrokeOptions::default()
        //                 .with_line_width(width)
        //                 .with_end_cap(nannou::lyon::tessellation::LineCap::Square)
        //         } else {
        //             StrokeOptions::default()
        //         })
        //         .color(if let Some(color) = self.fill {
        //             srgba(color.red, color.green, color.blue, visible_amount)
        //         } else {
        //             srgba(0., 0., 0., 0.)
        //         })
        //         .stroke_color(if let Some((color, _)) = self.stroke {
        //             srgba(color.red, color.green, color.blue, visible_amount)
        //         } else {
        //             srgba(0., 0., 0., 0.)
        //         })
        //         .points(points)
        //         .finish();
        // }
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
