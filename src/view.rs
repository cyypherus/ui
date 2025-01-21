use std::time::Instant;

use crate::rect::{AnimatedRect, Rect};
use crate::text::Text;
use crate::{ClickState, DragState, GestureHandler, Ui};
use backer::nodes::{draw, draw_object};
use backer::transitions::TransitionDrawable;
use backer::{models::Area, Node};
use vello::kurbo::{Affine, RoundedRect, Shape, Stroke};
use vello::peniko::{Brush, Color, Fill};

pub fn view<'n, 's, State>(ui: &'s mut Ui<State>, view: View<State>) -> Node<'n, Ui<State>> {
    view.view(ui)
}

pub struct View<State> {
    pub(crate) view_type: ViewType,
    pub(crate) gesture_handler: GestureHandler<State>,
}

#[derive(Debug, Clone)]
pub(crate) enum ViewType {
    Text(Text),
    Rect(Rect),
}

pub(crate) trait ViewTrait<'s, State>: TransitionDrawable<Ui<State>> + Sized {
    fn view(self, ui: &mut Ui<State>, node: Node<'s, Ui<State>>) -> Node<'s, Ui<State>>;
}

#[derive(Debug)]
pub(crate) enum AnimatedView {
    Rect(AnimatedRect),
}

impl<State> View<State> {
    pub fn on_click(mut self, f: impl Fn(&mut State, ClickState) + 'static) -> View<State> {
        self.gesture_handler.on_click = Some(Box::new(f));
        self
    }
    pub fn on_drag(mut self, f: impl Fn(&mut State, DragState) + 'static) -> View<State> {
        self.gesture_handler.on_drag = Some(Box::new(f));
        self
    }
    pub fn on_hover(mut self, f: impl Fn(&mut State, bool) + 'static) -> View<State> {
        self.gesture_handler.on_hover = Some(Box::new(f));
        self
    }
    pub fn easing(mut self, easing: backer::Easing) -> Self {
        match self.view_type {
            ViewType::Text(ref mut view) => view.easing = Some(easing),
            ViewType::Rect(ref mut view) => view.easing = Some(easing),
        }
        self
    }
    pub fn transition_duration(mut self, duration_ms: f32) -> Self {
        match self.view_type {
            ViewType::Text(ref mut view) => view.duration = Some(duration_ms),
            ViewType::Rect(ref mut view) => view.duration = Some(duration_ms),
        }
        self
    }
    pub fn transition_delay(mut self, delay_ms: f32) -> Self {
        match self.view_type {
            ViewType::Text(ref mut view) => view.delay = delay_ms,
            ViewType::Rect(ref mut view) => view.delay = delay_ms,
        }
        self
    }
}

impl<State> View<State> {
    fn view<'n, 's>(self, ui: &'s mut Ui<State>) -> Node<'n, Ui<State>>
    where
        State: 's,
    {
        // match self.view_type.clone() {
        //     ViewType::Text(view) => view.view(ui, draw_object(self)),
        //     ViewType::Rect(view) => view.view(ui, draw_object(self)),
        // }
        match self.view_type.clone() {
            ViewType::Text(view) => draw(|_, _| {}),
            ViewType::Rect(mut view) => draw(move |area, state: &mut Ui<State>| {
                state.gesture_handlers.push((
                    *self.id(),
                    area,
                    GestureHandler {
                        on_click: self.gesture_handler.on_click.take(),
                        on_drag: self.gesture_handler.on_drag.take(),
                        on_hover: self.gesture_handler.on_hover.take(),
                    },
                ));
                let AnimatedView::Rect(mut animated) = state
                    .cx()
                    .view_state
                    .remove(&view.id)
                    .unwrap_or(AnimatedView::Rect(AnimatedRect::new_from(&view)));
                AnimatedRect::update(&view, &mut animated);
                let now = Instant::now();
                let path = RoundedRect::from_rect(
                    vello::kurbo::Rect::from_origin_size(
                        vello::kurbo::Point::new(area.x as f64, area.y as f64),
                        vello::kurbo::Size::new(area.width as f64, area.height as f64),
                    ),
                    animated.radius.animate_wrapped(now) as f64,
                )
                .to_path(0.01);
                if animated.fill.is_none() && animated.stroke.is_none() {
                    state.cx().scene.fill(
                        Fill::EvenOdd,
                        Affine::IDENTITY,
                        Color::BLACK.multiply_alpha(1.),
                        None,
                        &path,
                    )
                } else {
                    if let Some(fill) = &animated.fill {
                        state.cx().scene.fill(
                            Fill::EvenOdd,
                            Affine::IDENTITY,
                            Color::rgba8(
                                fill.r.animate_wrapped(now).0,
                                fill.g.animate_wrapped(now).0,
                                fill.b.animate_wrapped(now).0,
                                255,
                            ),
                            None,
                            &path,
                        )
                    }
                    if let Some((stroke, width)) = &animated.stroke {
                        state.cx().scene.stroke(
                            &Stroke::new(width.animate_wrapped(now) as f64),
                            Affine::IDENTITY,
                            &Brush::Solid(Color::rgba8(
                                stroke.r.animate_wrapped(now).0,
                                stroke.g.animate_wrapped(now).0,
                                stroke.b.animate_wrapped(now).0,
                                255,
                            )),
                            None,
                            &path,
                        );
                    }
                }
                state
                    .cx()
                    .view_state
                    .insert(*self.id(), AnimatedView::Rect(animated));
            }),
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
            ViewType::Text(view) => view.easing,
            ViewType::Rect(view) => view.easing,
        }
        .unwrap_or(backer::Easing::EaseOut)
    }

    fn duration(&self) -> f32 {
        match &self.view_type {
            ViewType::Text(view) => view.duration,
            ViewType::Rect(view) => view.duration,
        }
        .unwrap_or(200.)
    }
    fn delay(&self) -> f32 {
        match &self.view_type {
            ViewType::Text(view) => view.delay,
            ViewType::Rect(view) => view.delay,
        }
    }
    fn constraints(
        &self,
        available_area: Area,
        state: &mut Ui<State>,
    ) -> Option<backer::SizeConstraints> {
        match &self.view_type {
            ViewType::Text(view) => view.constraints(available_area, state),
            ViewType::Rect(view) => view.constraints(available_area, state),
        }
    }
}
