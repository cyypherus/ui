use crate::{
    area_contains, view::AnimatedView, ClickState, DragState, GestureHandler, GestureState, Point,
};
pub use backer::{
    models::*,
    transitions::{AnimationBank, TransitionState},
};
use backer::{
    nodes::scoper,
    traits::{ScopeCtx, ScopeCtxResult},
    Node,
};
use parley::{FontContext, LayoutContext};
use std::{
    cell::{Cell, RefCell},
    sync::Arc,
};
use std::{collections::HashMap, rc::Rc};
use vello::{util::RenderSurface, Scene};
use winit::window::Window;

pub struct Ui<State: Clone> {
    pub state: State,
    pub gesture_handlers: Vec<(u64, Area, GestureHandler<State>)>,
    pub cx: Option<UiCx>,
}

impl<'s, State: Clone + 'static> Ui<State> {
    pub fn scope_ui<ScopedState: 'static + Clone>(
        &mut self,
        scope_ctx: ScopeCtx<Ui<ScopedState>>,
        scope: fn(&mut State) -> &mut ScopedState,
    ) -> ScopeCtxResult {
        let child_cx = self.cx.take();
        let mut child_ui = Ui {
            state: scope(&mut self.state).clone(),
            gesture_handlers: Vec::new(),
            cx: child_cx,
        };
        let result = scope_ctx.with_scoped(&mut child_ui);
        self.cx = child_ui.cx.take();
        // embed(&mut self.state, child_ui.state);
        self.gesture_handlers.append(
            &mut child_ui
                .gesture_handlers
                .into_iter()
                .map(|h| {
                    (
                        h.0,
                        h.1,
                        GestureHandler {
                            on_click: h.2.on_click.map(|o_c| {
                                let r: Box<dyn Fn(&mut State, ClickState)> =
                                    Box::new(move |state, click_state| {
                                        (o_c)(&mut scope(state), click_state)
                                    });
                                return r;
                            }),

                            on_drag: h.2.on_drag.map(|o_c| {
                                let r: Box<dyn Fn(&mut State, DragState)> =
                                    Box::new(move |state, drag_state| {
                                        (o_c)(&mut scope(state), drag_state)
                                    });
                                return r;
                            }),
                            on_hover: h.2.on_hover.map(|o_c| {
                                let r: Box<dyn Fn(&mut State, bool)> =
                                    Box::new(move |state, on_hover| (o_c)(scope(state), on_hover));
                                return r;
                            }),
                        },
                    )
                })
                .collect(),
        );
        return result;
    }
}

impl<State: Clone> TransitionState for Ui<State> {
    fn bank(&mut self) -> &mut AnimationBank {
        &mut self.cx().animation_bank
    }
}

impl<State: Clone> Ui<State> {
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
