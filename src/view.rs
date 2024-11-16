use crate::rect::Rect;
use crate::text_view::Text;
use crate::{ClickState, DragState, GestureHandler, Ui, ViewTrait};
use backer::nodes::draw_object;
use backer::transitions::TransitionDrawable;
use backer::{models::Area, Node};

pub(crate) fn view<'s, State: 'static>(
    ui: &mut Ui<State>,
    view: View<State>,
) -> Node<Ui<'s, State>> {
    view.view(ui)
}

pub(crate) struct View<State> {
    pub(crate) view_type: ViewType,
    pub(crate) gesture_handler: GestureHandler<State>,
    pub(crate) easing: Option<backer::Easing>,
    pub(crate) duration: Option<f32>,
    pub(crate) delay: f32,
}

impl<State> View<State> {
    pub(crate) fn on_click(mut self, f: impl Fn(&mut State, ClickState) + 'static) -> View<State> {
        self.gesture_handler.on_click = Some(Box::new(f));
        self
    }
    pub(crate) fn on_drag(mut self, f: impl Fn(&mut State, DragState) + 'static) -> View<State> {
        self.gesture_handler.on_drag = Some(Box::new(f));
        self
    }
    pub(crate) fn on_hover(mut self, f: impl Fn(&mut State, bool) + 'static) -> View<State> {
        self.gesture_handler.on_hover = Some(Box::new(f));
        self
    }
    pub(crate) fn easing(mut self, easing: backer::Easing) -> Self {
        self.easing = Some(easing);
        self
    }
    pub(crate) fn transition_duration(mut self, duration_ms: f32) -> Self {
        self.duration = Some(duration_ms);
        self
    }
    pub(crate) fn transition_delay(mut self, delay_ms: f32) -> Self {
        self.delay = delay_ms;
        self
    }
}

impl<State: 'static> View<State> {
    fn view<'s>(self, ui: &mut Ui<State>) -> Node<Ui<'s, State>> {
        match self.view_type.clone() {
            ViewType::Text(view) => view.view(ui, draw_object(self)),
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
pub(crate) enum ViewType {
    Text(Text),
    Rect(Rect),
}
