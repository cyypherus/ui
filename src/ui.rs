use crate::{view::AnimatedView, ClickState, DragState, GestureHandler};
use backer::Node;
pub use backer::{
    models::*,
    transitions::{AnimationBank, TransitionState},
};
use parley::{FontContext, LayoutContext};
use std::{cell::Cell, sync::Arc};
use std::{collections::HashMap, rc::Rc};
use vello::{util::RenderSurface, Scene};
use winit::window::Window;

pub struct Ui<State> {
    pub state: State,
    pub gesture_handlers: Vec<(u64, Area, GestureHandler<State>)>,
    pub cx: Option<UiCx>,
}

pub fn scoper<State: 'static, T: 'static>(
    scope: impl Fn(&mut State) -> T + 'static + Copy,
    embed: impl Fn(&mut State, T) + 'static + Copy,
    tree: impl Fn(&mut Ui<T>) -> Node<Ui<T>> + 'static,
) -> Node<Ui<State>> {
    backer::nodes::scope(
        move |ui: &mut Ui<State>| ui.scope_ui(scope),
        move |embed_ui: Ui<T>, ui: &mut Ui<State>| ui.embed_ui(scope, embed, embed_ui),
        tree,
    )
}

impl<'s, State: 'static> Ui<State> {
    pub fn embed_ui<T: 'static>(
        &mut self,
        scope: impl Fn(&mut State) -> T + 'static + Copy,
        embed_state: impl Fn(&mut State, T) + 'static + Copy,
        mut embed: Ui<T>,
    ) {
        embed_state(&mut self.state, embed.state);
        self.cx = embed.cx.take();
        self.gesture_handlers.append(
            &mut embed
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
                                        let mut scoped = scope(state);
                                        (o_c)(&mut scoped, click_state);
                                        embed_state(state, scoped);
                                    });
                                r
                            }),

                            on_drag: h.2.on_drag.map(|o_c| {
                                let r: Box<dyn Fn(&mut State, DragState)> =
                                    Box::new(move |state, drag_state| {
                                        let mut scoped = scope(state);
                                        (o_c)(&mut scoped, drag_state);
                                        embed_state(state, scoped);
                                    });
                                r
                            }),
                            on_hover: h.2.on_hover.map(|o_c| {
                                let r: Box<dyn Fn(&mut State, bool)> =
                                    Box::new(move |state, on_hover| {
                                        let mut scoped = scope(state);
                                        (o_c)(&mut scoped, on_hover);
                                        embed_state(state, scoped);
                                    });
                                r
                            }),
                        },
                    )
                })
                .collect(),
        );
    }
    pub fn scope_ui<T: 'static>(
        &mut self,
        scope: impl Fn(&mut State) -> T + 'static + Copy,
    ) -> Ui<T> {
        let child_cx = self.cx.take();
        Ui {
            state: scope(&mut self.state),
            gesture_handlers: Vec::new(),
            cx: child_cx,
        }
        // self.cx = child_ui.cx.take();
        // self.gesture_handlers.append(
        //     &mut child_ui
        //         .gesture_handlers
        //         .into_iter()
        //         .map(|h| {
        //             (
        //                 h.0,
        //                 h.1,
        //                 GestureHandler {
        //                     on_click: h.2.on_click.map(|o_c| {
        //                         let r: Box<dyn Fn(&mut State, ClickState)> =
        //                             Box::new(move |state, click_state| {
        //                                 (o_c)(&mut scope(state), click_state)
        //                             });
        //                         return r;
        //                     }),

        //                     on_drag: h.2.on_drag.map(|o_c| {
        //                         let r: Box<dyn Fn(&mut State, DragState)> =
        //                             Box::new(move |state, drag_state| {
        //                                 (o_c)(&mut scope(state), drag_state)
        //                             });
        //                         return r;
        //                     }),
        //                     on_hover: h.2.on_hover.map(|o_c| {
        //                         let r: Box<dyn Fn(&mut State, bool)> =
        //                             Box::new(move |state, on_hover| (o_c)(scope(state), on_hover));
        //                         return r;
        //                     }),
        //                 },
        //             )
        //         })
        //         .collect(),
        // );
    }
}

impl<State> TransitionState for Ui<State> {
    fn bank(&mut self) -> &mut AnimationBank {
        &mut self.cx().animation_bank
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
