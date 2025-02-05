use crate::ui::RcUi;
use crate::view::{View, ViewType};
use crate::GestureHandler;
use backer::models::Area;
use backer::Node;
use lilt::Easing;

#[derive(Debug, Clone, Copy)]
pub struct Svg {
    pub(crate) id: u64,
    pub(crate) source: fn() -> Vec<u8>,
    pub(crate) easing: Option<Easing>,
    pub(crate) duration: Option<f32>,
    pub(crate) delay: f32,
}

pub fn svg(id: u64, source: fn() -> Vec<u8>) -> Svg {
    Svg {
        id,
        source,
        easing: None,
        duration: None,
        delay: 0.,
    }
}

impl Svg {
    pub fn view<State>(self) -> View<State, ()> {
        View {
            view_type: ViewType::Svg(self),
            gesture_handler: GestureHandler {
                on_click: None,
                on_drag: None,
                on_hover: None,
                on_key: None,
            },
        }
    }
    pub fn finish<'n, State: 'n>(self) -> Node<'n, RcUi<State>> {
        self.view().finish()
    }
}

impl Svg {
    pub(crate) fn draw<State>(
        &mut self,
        area: Area,
        state: &mut RcUi<State>,
        visible: bool,
        visible_amount: f32,
    ) {
        if !visible && visible_amount == 0. {
            return;
        }
        state.ui.images.insert(self.id, (area, self.source));
    }
}
