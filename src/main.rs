use std::any::Any;
use std::collections::HashMap;
use std::future;
use std::hash::{DefaultHasher, Hash, Hasher};

use backer::models::Area;
use backer::nodes::*;
use backer::transitions::{AnimationBank, TransitionDrawable, TransitionState};
use backer::{id, Layout, Node};
use nannou::app::App as NannouApp;
use nannou::lyon::tessellation::StrokeOptions;
use nannou::prelude::*;
use text_view::{text, Text};

mod text_view;

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
                    view(ui, text(id!(), "Hello").finish()).visible(ui.state.toggle),
                    view(
                        ui,
                        rect(id!())
                            .stroke(srgb(0.4, 0.4, 0.4), 1.)
                            .fill(match (ui.state.hovered, ui.state.depressed) {
                                (_, true) => srgb(0.2, 0.2, 0.2),
                                (true, false) => srgb(0.3, 0.3, 0.3),
                                (false, false) => srgb(0.1, 0.1, 0.1),
                            })
                            .corner_rounding(if ui.state.hovered { 0.1 } else { 0.2 })
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

fn view<State: 'static>(ui: &mut Ui<State>, view: View<State>) -> Node<Ui<State>> {
    view.view(ui)
}

struct UserState {
    hovered: bool,
    depressed: bool,
    toggle: bool,
}

struct App<State> {
    ui: Ui<State>,
    view: Layout<Ui<State>>,
}

struct Ui<State> {
    state: State,
    animation_bank: AnimationBank,
    view_state: HashMap<u64, Box<dyn Any>>,
    gesture_handlers: Vec<(u64, Area, GestureHandler<State>)>,
    cursor_position: Option<Point>,
    gesture_state: GestureState,
    draw: Draw,
    window_size: Area,
}

struct View<State> {
    view_type: ViewType,
    gesture_handler: GestureHandler<State>,
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
}

impl<State: 'static> View<State> {
    fn view(self, ui: &mut Ui<State>) -> Node<Ui<State>> {
        match self.view_type.clone() {
            ViewType::Text(view) => view.view(ui, draw_object(self)),
            ViewType::Rect(view) => view.view(ui, draw_object(self)),
        }
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

#[derive(Debug, Clone)]
enum ViewType {
    Text(Text),
    Rect(Rect),
}

impl<State: 'static> App<State> {
    pub fn start(state: State, view: impl Fn(&mut Ui<State>) -> Node<Ui<State>> + 'static) {
        nannou::app::Builder::new_async(move |app| {
            Box::new(future::ready({
                app.new_window()
                    .size(1024, 1024)
                    .view(Self::view)
                    .event(Self::window_event)
                    .build()
                    .unwrap();
                Self {
                    ui: Ui {
                        state,
                        animation_bank: AnimationBank::new(),
                        view_state: HashMap::new(),
                        gesture_handlers: Vec::new(),
                        cursor_position: None,
                        gesture_state: GestureState::None,
                        draw: app.draw(),
                        window_size: Area::default(),
                    },
                    view: Layout::new(view),
                }
            }))
        })
        .update(Self::update)
        .run();
    }
    fn view(app: &NannouApp, state: &Self, frame: Frame) {
        state.ui.draw.to_frame(app, &frame).unwrap();
    }
    fn update(app: &NannouApp, state: &mut Self, _update: Update) {
        let size_rect = app.window_rect();
        let size = Area {
            x: size_rect.x(),
            y: size_rect.y(),
            width: size_rect.w(),
            height: size_rect.h(),
        };
        state.ui.window_size = size;
        state.ui.gesture_handlers.clear();
        state.ui.draw.background().color(BLUE);
        state.view.draw(size, &mut state.ui);
    }
    fn window_event(_app: &NannouApp, state: &mut Self, event: WindowEvent) {
        match event {
            KeyPressed(_key) => {}
            KeyReleased(_key) => {}
            ReceivedCharacter(_char) => {}
            MouseMoved(pos) => {
                let current_position =
                    ui_to_draw_point(Point { x: pos.x, y: pos.y }, state.ui.window_size);
                state.ui.cursor_position = Some(current_position);
                state.ui.gesture_handlers.iter().for_each(|(_, area, gh)| {
                    if let Some(on_hover) = &gh.on_hover {
                        on_hover(&mut state.ui.state, area_contains(area, current_position));
                    }
                });
                if let GestureState::Dragging { start, capturer } = state.ui.gesture_state {
                    let distance = start.distance(current_position);
                    if let Some(Some(handler)) = state
                        .ui
                        .gesture_handlers
                        .iter()
                        .find(|(id, _, _)| *id == capturer)
                        .map(|(_, _, gh)| &gh.on_drag)
                    {
                        handler(
                            &mut state.ui.state,
                            DragState::Updated {
                                start,
                                current: current_position,
                                distance,
                            },
                        );
                    }
                }
            }
            MousePressed(MouseButton::Left) => {
                if let Some(point) = state.ui.cursor_position {
                    if let Some((capturer, _, handler)) = state
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
                            on_click(&mut state.ui.state, ClickState::Started);
                        }
                        state.ui.gesture_state = GestureState::Dragging {
                            start: point,
                            capturer: *capturer,
                        }
                    }
                }
            }
            MousePressed(_) => {}
            MouseReleased(MouseButton::Left) => {
                if let Some(current) = state.ui.cursor_position {
                    if let GestureState::Dragging { start, capturer } = state.ui.gesture_state {
                        let distance = start.distance(current);
                        if let Some((_, area, handler)) = state
                            .ui
                            .gesture_handlers
                            .iter()
                            .find(|(id, _, _)| *id == capturer)
                        {
                            if let Some(ref on_click) = handler.on_click {
                                if area_contains(area, current) {
                                    on_click(&mut state.ui.state, ClickState::Completed);
                                } else {
                                    on_click(&mut state.ui.state, ClickState::Cancelled);
                                }
                            }
                            if let Some(ref on_drag) = handler.on_drag {
                                on_drag(
                                    &mut state.ui.state,
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
                state.ui.gesture_state = GestureState::None;
            }
            MouseReleased(_) => {}
            MouseEntered => {}
            MouseExited => {}
            MouseWheel(_amount, _phase) => {}
            Moved(_pos) => {}
            Resized(_size) => {}
            Touch(_touch) => {}
            TouchPressure(_pressure) => {}
            HoveredFile(_path) => {}
            DroppedFile(_path) => {}
            HoveredFileCancelled => {}
            Focused => {}
            Unfocused => {}
            Closed => {}
        }
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

#[derive(Debug, Clone, Copy)]
struct Point {
    x: f32,
    y: f32,
}

trait ViewTrait<State>: TransitionDrawable<Ui<State>> + Sized {
    fn view(self, ui: &mut Ui<State>, node: Node<Ui<State>>) -> Node<Ui<State>>;
}

impl<State> TransitionState for Ui<State> {
    fn bank(&mut self) -> &mut AnimationBank {
        &mut self.animation_bank
    }
}

#[derive(Debug, Clone)]
struct Rect {
    id: u64,
    fill: Option<Srgb<f32>>,
    rounding: f32,
    stroke: Option<(Srgb<f32>, f32)>,
    easing: Option<backer::Easing>,
    duration: Option<f32>,
}

fn rect(id: String) -> Rect {
    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    Rect {
        id: hasher.finish(),
        fill: None,
        rounding: 0.,
        stroke: None,
        easing: None,
        duration: None,
    }
}

impl Rect {
    fn fill(mut self, color: Srgb<f32>) -> Self {
        self.fill = Some(color);
        self
    }
    fn corner_rounding(mut self, radius: f32) -> Self {
        self.rounding = radius;
        self
    }
    fn stroke(mut self, color: Srgb<f32>, line_width: f32) -> Self {
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
        let area = ui_to_draw(area, state.window_size);
        fn generate_squircle(
            x: f32,
            y: f32,
            width: f32,
            height: f32,
            radius: f32,
        ) -> impl Iterator<Item = (f32, f32)> {
            let a = width / 2.0;
            let b = height / 2.0;
            let aspect = width / height;
            let x_exponent = 1.0 / radius;
            let y_exponent = (1.0 / radius) * aspect;
            let steps = (((width + height) * 2.) / 10.) as usize;
            (0..steps).map(move |i| {
                let t = (i as f32 / steps as f32) * std::f32::consts::TAU;
                let cos_t = t.cos();
                let sin_t = t.sin();
                let px = x + a * cos_t.signum() * cos_t.abs().powf(x_exponent);
                let py = y + b * sin_t.signum() * sin_t.abs().powf(y_exponent);

                (px, py)
            })
        }

        let points = generate_squircle(area.x, area.y, area.width, area.height, 1. / self.rounding);
        let polygon = state.draw.polygon();
        if self.stroke.is_none() && self.fill.is_none() {
            polygon
                .color(srgba(0., 0., 0., visible_amount))
                .points(points)
                .finish();
        } else {
            polygon
                .stroke_opts(if let Some((_, width)) = self.stroke {
                    StrokeOptions::default()
                        .with_line_width(width)
                        .with_end_cap(nannou::lyon::tessellation::LineCap::Square)
                } else {
                    StrokeOptions::default()
                })
                .color(if let Some(color) = self.fill {
                    srgba(color.red, color.green, color.blue, visible_amount)
                } else {
                    srgba(0., 0., 0., 0.)
                })
                .stroke_color(if let Some((color, _)) = self.stroke {
                    srgba(color.red, color.green, color.blue, visible_amount)
                } else {
                    srgba(0., 0., 0., 0.)
                })
                .points(points)
                .finish();
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

fn ui_to_draw(area: Area, window_size: Area) -> Area {
    Area {
        x: ((area.width * 0.5) - (window_size.width * 0.5)) + area.x,
        y: ((window_size.height * 0.5) - (area.height * 0.5)) - area.y,
        width: area.width,
        height: area.height,
    }
}

fn ui_to_draw_point(p: Point, window_size: Area) -> Point {
    Point {
        x: (window_size.width * 0.5) + p.x,
        y: (window_size.height * 0.5) - p.y,
    }
}
