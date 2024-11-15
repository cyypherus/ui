use backer::nodes::column_spaced;
use backer::{
    id,
    models::*,
    nodes::*,
    transitions::{AnimationBank, TransitionDrawable, TransitionState},
    Layout, Node,
};
use femtovg::{renderer::OpenGl, Canvas, Color, FontId, Paint, Path, Renderer};
use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext},
    display::GetGlDisplay,
    prelude::*,
    surface::{Surface, SurfaceAttributesBuilder, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;
use resource::resource;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::{num::NonZeroU32, time::Instant};
use text_view::{text, Text};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton};
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::Key,
    window::Window,
};
use winit::{event_loop::ActiveEventLoop, window::WindowAttributes};

mod text_view;

fn main() {
    App::start(
        600,
        600,
        "Title",
        true,
        UserState {
            hovered: false,
            height: 0.,
            depressed: false,
        },
        Layout::new(|ui: &mut Ui<UserState>| {
            column_spaced(
                10.,
                vec![
                    space(),
                    stack(vec![view(
                        ui,
                        text(id!(), "Lorem ipsum")
                            .fill(Color::rgb(200, 200, 200))
                            .finish(),
                    )
                    .pad(10.)
                    .attach_under(view(
                        ui,
                        rect(id!())
                            .stroke(Color::rgb(100, 100, 100), 2.)
                            .fill(match (ui.state.hovered, ui.state.depressed) {
                                (_, true) => Color::rgb(20, 20, 20),
                                (true, false) => Color::rgb(50, 50, 50),
                                (false, false) => Color::rgb(10, 10, 10),
                            })
                            .corner_radius(5.)
                            .finish()
                            .on_hover(|state: &mut UserState, hovered| state.hovered = hovered)
                            .on_click(|state: &mut UserState, click_state| match click_state {
                                ClickState::Started => state.depressed = true,
                                ClickState::Cancelled => state.depressed = false,
                                ClickState::Completed => {
                                    state.depressed = false;
                                    if state.height > 50. {
                                        state.height = 0.;
                                    } else {
                                        state.height += 10.;
                                    }
                                }
                            }),
                    ))]),
                    view(
                        ui,
                        rect(id!())
                            .stroke(Color::rgb(255, 0, 0), 3.)
                            .corner_radius(20.)
                            .finish(),
                    )
                    .height(ui.state.height)
                    .visible(ui.state.hovered),
                    space(),
                ],
            )
            .pad(10.)
        }),
    );
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

struct UserState {
    hovered: bool,
    depressed: bool,
    height: f32,
}

impl<State> TransitionState for Ui<State> {
    fn bank(&mut self) -> &mut AnimationBank {
        &mut self.animation_bank
    }
}

struct App<State> {
    default_font: FontId,
    ui: Ui<State>,
    view: Layout<Ui<State>>,
}

struct Ui<State> {
    canvas: Canvas<OpenGl>,
    window: Window,
    context: PossiblyCurrentContext,
    surface: Surface<WindowSurface>,
    state: State,
    default_font: FontId,
    drawables: Vec<View<State>>,
    animation_bank: AnimationBank,
    view_state: HashMap<u64, Box<dyn Any>>,
    gesture_handlers: Vec<(u64, Area, GestureHandler<State>)>,
    cursor_position: Option<Point>,
    gesture_state: GestureState,
}

impl<State> ApplicationHandler for App<State> {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(physical_size) => {
                self.ui.surface.resize(
                    &self.ui.context,
                    physical_size.width.try_into().unwrap(),
                    physical_size.height.try_into().unwrap(),
                );
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Character(_),
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            } => {}
            WindowEvent::RedrawRequested => {
                self.ui.gesture_handlers.clear();
                let dpi_factor = self.ui.window.scale_factor();
                let size = self.ui.window.inner_size();
                self.ui
                    .canvas
                    .set_size(size.width, size.height, dpi_factor as f32);
                self.ui.canvas.clear_rect(
                    0,
                    0,
                    size.width,
                    size.height,
                    Color::rgbf(0.9, 0.9, 0.9),
                );
                self.view.draw(
                    Area::new(0., 0., size.width as f32, size.height as f32),
                    &mut self.ui,
                );

                if self.ui.animation_bank.in_progress(Instant::now()) {
                    self.ui.window.request_redraw();
                }

                self.ui.canvas.save();
                self.ui.canvas.reset();
                self.ui.canvas.restore();

                self.ui.canvas.flush();
                self.ui.surface.swap_buffers(&self.ui.context).unwrap();
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if let Some(point) = self.ui.cursor_position {
                    if let Some((capturer, _, handler)) = self
                        .ui
                        .gesture_handlers
                        .iter()
                        .rev()
                        .find(|(_, area, handler)| {
                            area_contains(area, point)
                                && (handler.on_click.is_some() || handler.on_drag.is_some())
                        })
                    {
                        if let Some(ref on_click) = handler.on_click {
                            on_click(&mut self.ui.state, ClickState::Started);
                            self.ui.window.request_redraw();
                        }
                        self.ui.gesture_state = GestureState::Dragging {
                            start: point,
                            capturer: *capturer,
                        }
                    }
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                if let Some(current) = self.ui.cursor_position {
                    if let GestureState::Dragging { start, capturer } = self.ui.gesture_state {
                        let distance = start.distance(current);
                        let mut needs_redraw = false;
                        if let Some((_, area, handler)) = self
                            .ui
                            .gesture_handlers
                            .iter()
                            .find(|(id, _, _)| *id == capturer)
                        {
                            if let Some(ref on_click) = handler.on_click {
                                if area_contains(area, current) {
                                    on_click(&mut self.ui.state, ClickState::Completed);
                                    needs_redraw = true;
                                } else {
                                    on_click(&mut self.ui.state, ClickState::Cancelled);
                                    needs_redraw = true;
                                }
                            }
                            if let Some(ref on_drag) = handler.on_drag {
                                on_drag(
                                    &mut self.ui.state,
                                    DragState::Completed {
                                        start,
                                        current,
                                        distance,
                                    },
                                );
                                needs_redraw = true;
                            }
                        }
                        if needs_redraw {
                            self.ui.window.request_redraw();
                        }
                    }
                }
                self.ui.gesture_state = GestureState::None;
            }
            WindowEvent::CursorMoved { position, .. } => {
                let current_position = Point {
                    x: position.x as f32,
                    y: position.y as f32,
                };
                self.ui.cursor_position = Some(current_position);
                let mut needs_redraw = false;
                self.ui.gesture_handlers.iter().for_each(|(_, area, gh)| {
                    if let Some(on_hover) = &gh.on_hover {
                        on_hover(&mut self.ui.state, area_contains(area, current_position));
                        needs_redraw = true;
                    }
                });
                if let GestureState::Dragging { start, capturer } = self.ui.gesture_state {
                    let distance = start.distance(current_position);
                    if let Some(Some(handler)) = self
                        .ui
                        .gesture_handlers
                        .iter()
                        .find(|(id, _, _)| *id == capturer)
                        .map(|(_, _, gh)| &gh.on_drag)
                    {
                        handler(
                            &mut self.ui.state,
                            DragState::Updated {
                                start,
                                current: current_position,
                                distance,
                            },
                        );
                        needs_redraw = true;
                    }
                }
                if needs_redraw {
                    self.ui.window.request_redraw();
                }
            }
            _ => (),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum GestureState {
    None,
    Dragging { start: Point, capturer: u64 },
}

#[derive(Debug, Clone, Copy)]
struct Point {
    x: f32,
    y: f32,
}

impl Point {
    fn distance(&self, to: Point) -> f32 {
        ((to.x - self.x).powf(2.) + (to.y - self.y).powf(2.)).sqrt()
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

fn view<State: 'static>(ui: &mut Ui<State>, view: View<State>) -> Node<Ui<State>> {
    view.view(ui)
}

struct View<State> {
    view_type: ViewType,
    gesture_handler: GestureHandler<State>,
}

#[derive(Debug, Clone)]
enum ViewType {
    Text(Text),
    Rect(Rect),
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
}

impl<State> TransitionDrawable<Ui<State>> for View<State> {
    fn draw_interpolated(
        &mut self,
        area: Area,
        state: &mut Ui<State>,
        visible: bool,
        visible_amount: f32,
    ) {
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
            ViewType::Text(view) => view.draw_interpolated(area, state, visible, visible_amount),
            ViewType::Rect(view) => view.draw_interpolated(area, state, visible, visible_amount),
        }
    }

    fn id(&self) -> &u64 {
        match &self.view_type {
            ViewType::Text(view) => <Text as TransitionDrawable<Ui<State>>>::id(view),
            ViewType::Rect(view) => <Rect as TransitionDrawable<Ui<State>>>::id(view),
        }
    }

    fn easing(&self) -> backer::Easing {
        match &self.view_type {
            ViewType::Text(view) => <Text as TransitionDrawable<Ui<State>>>::easing(view),
            ViewType::Rect(view) => <Rect as TransitionDrawable<Ui<State>>>::easing(view),
        }
    }

    fn duration(&self) -> f32 {
        match &self.view_type {
            ViewType::Text(view) => <Text as TransitionDrawable<Ui<State>>>::duration(view),
            ViewType::Rect(view) => <Rect as TransitionDrawable<Ui<State>>>::duration(view),
        }
    }
}

impl<State: 'static> View<State> {
    fn view(self, ui: &mut Ui<State>) -> Node<Ui<State>> {
        match self.view_type.clone() {
            ViewType::Text(view) => view.view(ui, draw_object(self)),
            ViewType::Rect(view) => view.view(ui, draw_object(self)),
        }
    }
}

trait ViewTrait<State>: TransitionDrawable<Ui<State>> + Sized {
    fn view(self, ui: &mut Ui<State>, node: Node<Ui<State>>) -> Node<Ui<State>>;
}

#[derive(Debug, Clone)]
struct Rect {
    id: u64,
    fill: Option<Color>,
    corner_radius: f32,
    stroke: Option<(Color, f32)>,
    easing: Option<backer::Easing>,
    duration: Option<f32>,
}

fn rect(id: String) -> Rect {
    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    Rect {
        id: hasher.finish(),
        fill: None,
        corner_radius: 0.,
        stroke: None,
        easing: None,
        duration: None,
    }
}

impl Rect {
    fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }
    fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }
    fn stroke(mut self, color: Color, line_width: f32) -> Self {
        self.stroke = Some((color, line_width));
        self
    }
    fn finish<State>(self) -> View<State> {
        View {
            view_type: ViewType::Rect(self),
            gesture_handler: GestureHandler {
                on_click: None,
                on_drag: None,
                on_hover: None,
            },
        }
    }
}

impl<State> TransitionDrawable<Ui<State>> for Rect {
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
        let mut path = Path::new();
        path.rounded_rect(area.x, area.y, area.width, area.height, self.corner_radius);
        if visible_amount < 1. {
            if let Some(ref mut fill) = self.fill {
                fill.set_alphaf(visible_amount)
            }
            if let Some((ref mut stroke, _)) = self.stroke {
                stroke.set_alphaf(visible_amount);
            }
        }
        if let (None, None) = (self.fill, self.stroke) {
            state.canvas.fill_path(&path, &Paint::color(Color::black()));
        }
        if let Some(color) = self.fill {
            state.canvas.fill_path(&path, &Paint::color(color));
        }
        if let Some((color, width)) = self.stroke {
            let mut stroke_paint = Paint::color(color);
            stroke_paint.set_line_width(width);
            state.canvas.stroke_path(&path, &stroke_paint);
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
}

impl<State> ViewTrait<State> for Rect {
    fn view(self, _ui: &mut Ui<State>, node: Node<Ui<State>>) -> Node<Ui<State>> {
        node
    }
}

// fn toggle<State>(id: String, ui: &mut Ui<State>) -> Node<Ui<State>> {
//     let mut hasher = DefaultHasher::new();
//     id.hash(&mut hasher);
//     let vs: &mut ButtonState = ui
//         .view_state
//         .get_mut(&hasher.finish())
//         .unwrap()
//         .downcast_mut()
//         .unwrap();
//     rect(id)
// }

// fn text<State>(id: String, ui: &mut Ui<State>, text: impl AsRef<str> + 'static) -> Node<Ui<State>> {
//     let font_size = 18.;
//     let paint = Paint::color(Color::black())
//         .with_font(&[ui.default_font])
//         .with_font_size(font_size);
//     let text_size = ui
//         .canvas
//         .measure_text(0., 0., text.as_ref(), &paint)
//         .expect("Error measuring font");

//     draw_object(DrawableId {
//         id,
//         draw_type: Drawable::Text {
//             size: font_size,
//             text: text.as_ref().to_owned(),
//         },
//     })
//     .height(text_size.height())
//     .width(text_size.width())
// }

// fn rect<State>(id: String) -> Node<Ui<State>> {
//     draw_object(DrawableId {
//         id,
//         draw_type: Drawable::Rect,
//     })
// }

impl<State> App<State> {
    fn new(
        default_font: FontId,
        canvas: Canvas<OpenGl>,
        window: Window,
        context: PossiblyCurrentContext,
        surface: Surface<WindowSurface>,
        state: State,
        view: Layout<Ui<State>>,
    ) -> Self {
        Self {
            default_font,
            ui: Ui {
                state,
                canvas,
                window,
                context,
                surface,
                drawables: Vec::new(),
                animation_bank: AnimationBank::new(),
                default_font,
                view_state: HashMap::new(),
                gesture_handlers: Vec::new(),
                cursor_position: None,
                gesture_state: GestureState::None,
            },
            view,
        }
    }
    pub fn start(
        width: u32,
        height: u32,
        title: &'static str,
        resizeable: bool,
        state: State,
        view: Layout<Ui<State>>,
    ) {
        let event_loop = EventLoop::new().unwrap();

        let (mut canvas, window, context, surface) = {
            let window_attributes = WindowAttributes::default()
                .with_inner_size(winit::dpi::PhysicalSize::new(width, height))
                .with_resizable(resizeable)
                .with_title(title);

            let template = ConfigTemplateBuilder::new().with_alpha_size(8);

            let display_builder =
                DisplayBuilder::new().with_window_attributes(Some(window_attributes));

            let (window, gl_config) = display_builder
                .build(&event_loop, template, |configs| {
                    configs
                        .reduce(|accum, config| {
                            let transparency_check =
                                config.supports_transparency().unwrap_or(false)
                                    & !accum.supports_transparency().unwrap_or(false);

                            if transparency_check || config.num_samples() < accum.num_samples() {
                                config
                            } else {
                                accum
                            }
                        })
                        .unwrap()
                })
                .unwrap();

            let window = window.unwrap();

            let raw_window_handle = Some(window.raw_window_handle().unwrap());

            let gl_display = gl_config.display();

            let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);
            let fallback_context_attributes = ContextAttributesBuilder::new()
                .with_context_api(ContextApi::Gles(None))
                .build(raw_window_handle);
            let mut not_current_gl_context = Some(unsafe {
                gl_display
                    .create_context(&gl_config, &context_attributes)
                    .unwrap_or_else(|_| {
                        gl_display
                            .create_context(&gl_config, &fallback_context_attributes)
                            .expect("failed to create context")
                    })
            });

            let (width, height): (u32, u32) = window.inner_size().into();
            let raw_window_handle = window.raw_window_handle().unwrap();
            let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
                raw_window_handle,
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            );

            let surface = unsafe {
                gl_config
                    .display()
                    .create_window_surface(&gl_config, &attrs)
                    .unwrap()
            };

            let gl_context = not_current_gl_context
                .take()
                .unwrap()
                .make_current(&surface)
                .unwrap();

            let renderer = unsafe {
                OpenGl::new_from_function_cstr(|s| gl_display.get_proc_address(s).cast())
            }
            .expect("Cannot create renderer");
            let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");
            canvas.set_size(width, height, window.scale_factor() as f32);
            (canvas, window, gl_context, surface)
        };

        event_loop
            .run_app(&mut App::new(
                canvas
                    .add_font_mem(&resource!("/assets/Rubik-VariableFont_wght.ttf"))
                    .expect("Cannot add font"),
                canvas,
                window,
                context,
                surface,
                state,
                view,
            ))
            .unwrap();
    }
}

fn draw_paragraph<T: Renderer>(
    canvas: &mut Canvas<T>,
    font: FontId,
    area: Area,
    font_size: f32,
    text: &str,
) {
    let paint = Paint::color(Color::black())
        .with_font(&[font])
        .with_font_size(font_size);

    let font_metrics = canvas.measure_font(&paint).expect("Error measuring font");

    let width = canvas.width() as f32;
    let mut y = area.y + area.height;

    let lines = canvas
        .break_text_vec(width, text, &paint)
        .expect("Error while breaking text");

    for line_range in lines {
        if let Ok(_res) = canvas.fill_text(area.x, y, &text[line_range], &paint) {
            y += font_metrics.height();
        }
    }
}

fn draw_rect<T: Renderer>(canvas: &mut Canvas<T>, area: Area) {
    let paint = Paint::color(Color::rgb(255, 0, 0));
    let mut path = Path::new();
    path.rounded_rect(area.x, area.y, area.width, area.height, 10.);
    canvas.stroke_path(&path, &paint);
}
