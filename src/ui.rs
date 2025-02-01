use crate::{
    gestures::{ClickHandler, DragHandler, HoverHandler},
    view::AnimatedView,
    ClickState, DragState, GestureHandler,
};
pub use backer::models::*;
use lilt::Animated;
use parley::{FontContext, LayoutContext};
use std::{
    cell::{Cell, Ref, RefCell},
    sync::Arc,
    time::Instant,
};
use std::{collections::HashMap, rc::Rc};
use vello::{util::RenderSurface, Scene};
use winit::window::Window;

pub struct Ui<State> {
    pub state: State,
    pub gesture_handlers: Vec<(u64, Area, GestureHandler<State>)>,
    pub cx: Option<UiCx>,
}

#[derive(Debug, Clone)]
/// State storage for animation state
pub(crate) struct AnimationBank {
    pub(crate) animations: HashMap<u64, AnimArea>,
}
impl Default for AnimationBank {
    fn default() -> Self {
        Self::new()
    }
}
impl AnimationBank {
    /// Initialize an empty `AnimationBank`
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
        }
    }
    /// Checks if any animations are currently in progress
    pub(crate) fn in_progress(&self, time: Instant) -> bool {
        for value in self.animations.values() {
            if value.visible.in_progress(time)
                || value.x.in_progress(time)
                || value.y.in_progress(time)
                || value.width.in_progress(time)
                || value.height.in_progress(time)
            {
                return true;
            }
        }
        false
    }
}
#[derive(Debug, Clone)]
pub(crate) struct AnimArea {
    pub(crate) visible: Animated<bool, Instant>,
    pub(crate) x: Animated<f32, Instant>,
    pub(crate) y: Animated<f32, Instant>,
    pub(crate) width: Animated<f32, Instant>,
    pub(crate) height: Animated<f32, Instant>,
}

#[derive(Clone)]
pub struct RcUi<State> {
    pub ui: Rc<RefCell<Ui<State>>>,
}

impl<State> RcUi<State> {
    pub fn borrow_state(&self) -> Ref<State> {
        Ref::map(RefCell::borrow(&self.ui), |ui| &ui.state)
    }
}

// pub fn scoper<'a, State: 'a + 'static, T: Clone + 'a + 'static>(
//     scope: impl Fn(&mut State) -> T + 'static + Copy,
//     embed: impl Fn(&mut State, T) + 'static + Copy,
//     tree: impl Fn(&mut RcUi<T>) -> Node<'a, RcUi<T>> + 'static,
// ) -> Node<'a, RcUi<State>> {
//     backer::nodes::scope(
//         move |ui: &mut RcUi<State>| ui.scope_ui(scope),
//         move |ui: &mut RcUi<State>, embed_ui: RcUi<T>| ui.embed_ui(scope, embed, embed_ui),
//         tree,
//     )
// }

impl<State: 'static> RcUi<State> {
    pub fn embed_ui<T: Clone + 'static>(
        &mut self,
        scope: impl Fn(&mut State) -> &mut T + 'static + Copy,
        embed: RcUi<T>,
    ) {
        RefCell::borrow_mut(&self.ui).cx = RefCell::borrow_mut(&embed.ui).cx.take();
        RefCell::borrow_mut(&self.ui).gesture_handlers.append(
            &mut std::mem::take(&mut RefCell::borrow_mut(&embed.ui).gesture_handlers)
                .into_iter()
                .map(|h| {
                    (
                        h.0,
                        h.1,
                        GestureHandler {
                            on_click: h.2.on_click.map(|o_c| {
                                let r: ClickHandler<State> = Box::new(move |state, click_state| {
                                    let scoped = scope(state);
                                    (o_c)(scoped, click_state);
                                });
                                r
                            }),

                            on_drag: h.2.on_drag.map(|o_c| {
                                let r: DragHandler<State> = Box::new(move |state, drag_state| {
                                    let scoped = scope(state);
                                    (o_c)(scoped, drag_state);
                                });
                                r
                            }),
                            on_hover: h.2.on_hover.map(|o_c| {
                                let r: HoverHandler<State> = Box::new(move |state, on_hover| {
                                    let scoped = scope(state);
                                    (o_c)(scoped, on_hover);
                                });
                                r
                            }),
                        },
                    )
                })
                .collect::<Vec<_>>(),
        );
    }
    pub fn scope_ui<T>(&mut self, scope: impl Fn(&mut State) -> T + 'static + Copy) -> RcUi<T> {
        let child_cx = RefCell::borrow_mut(&self.ui).cx.take();
        RcUi {
            ui: Rc::new(RefCell::new(Ui {
                state: scope(&mut RefCell::borrow_mut(&self.ui).state),
                gesture_handlers: Vec::new(),
                cx: child_cx,
            })),
        }
    }
}

impl<State> Ui<State> {
    pub(crate) fn cx(&mut self) -> &mut UiCx {
        self.cx.as_mut().unwrap()
    }
}

pub struct UiCx {
    pub(crate) animation_bank: AnimationBank,
    pub(crate) scene: Scene,
    pub(crate) font_cx: Rc<Cell<Option<FontContext>>>,
    pub(crate) layout_cx: Rc<Cell<Option<LayoutContext>>>,
    pub(crate) view_state: HashMap<u64, AnimatedView>,
}

impl UiCx {
    pub(crate) fn with_font_layout_ctx<T>(
        &mut self,
        f: impl Fn(&mut LayoutContext, &mut FontContext) -> T,
    ) -> T {
        let mut layout_ctx = self.layout_cx.take().unwrap();
        let mut font_cx = self.font_cx.take().unwrap();
        let result = f(&mut layout_ctx, &mut font_cx);
        self.layout_cx.set(Some(layout_ctx));
        self.font_cx.set(Some(font_cx));
        result
    }
}

pub(crate) struct RenderState<'surface> {
    // SAFETY: We MUST drop the surface before the `window`, so the fields
    // must be in this order
    pub(crate) surface: RenderSurface<'surface>,
    pub(crate) window: Arc<Window>,
}
