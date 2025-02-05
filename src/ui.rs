use crate::{
    gestures::{ClickHandler, DragHandler, HoverHandler, KeyHandler},
    view::AnimatedView,
    GestureHandler,
};
pub use backer::models::*;
use backer::Node;
use lilt::Animated;
use parley::{FontContext, LayoutContext};
use std::{cell::Cell, sync::Arc, time::Instant};
use std::{collections::HashMap, rc::Rc};
use vello_svg::vello::util::RenderSurface;
use vello_svg::vello::Scene;
use winit::window::Window;

type ImageData = HashMap<u64, (Area, fn() -> Vec<u8>)>;

pub struct Ui<State> {
    pub state: State,
    pub gesture_handlers: Vec<(u64, Area, GestureHandler<State>)>,
    pub cx: Option<UiCx>,
    pub(crate) images: ImageData,
    pub(crate) now: Instant,
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

pub struct RcUi<State> {
    pub ui: Ui<State>,
}

pub fn scoper<'n, State, Scoped: 'n + 'static>(
    scope: impl Fn(&mut State) -> Scoped + 'static + Copy,
    embed: impl Fn(&mut State, Scoped) + 'static + Copy,
    node: Node<'n, RcUi<Scoped>>,
) -> Node<'n, RcUi<State>> {
    backer::nodes::scope_owned(
        move |ui: &mut RcUi<State>| {
            let child_cx = ui.ui.cx.take();
            RcUi {
                ui: Ui {
                    state: scope(&mut ui.ui.state),
                    gesture_handlers: Vec::new(),
                    cx: child_cx,
                    images: HashMap::new(),
                    now: ui.ui.now,
                },
            }
        },
        move |ui: &mut RcUi<State>, mut embedded: RcUi<Scoped>| {
            ui.ui.cx = embedded.ui.cx.take();
            ui.ui.gesture_handlers.append(
                &mut std::mem::take(&mut embedded.ui.gesture_handlers)
                    .into_iter()
                    .map(|h| {
                        (
                            h.0,
                            h.1,
                            GestureHandler {
                                on_click: h.2.on_click.map(|o_c| {
                                    let r: ClickHandler<State> =
                                        Rc::new(move |state, click_state| {
                                            let mut scoped = scope(state);
                                            (o_c)(&mut scoped, click_state);
                                            embed(state, scoped);
                                        });
                                    r
                                }),
                                on_drag: h.2.on_drag.map(|o_c| {
                                    let r: DragHandler<State> =
                                        Rc::new(move |state, drag_state| {
                                            let mut scoped = scope(state);
                                            (o_c)(&mut scoped, drag_state);
                                            embed(state, scoped);
                                        });
                                    r
                                }),
                                on_hover: h.2.on_hover.map(|o_c| {
                                    let r: HoverHandler<State> = Rc::new(move |state, on_hover| {
                                        let mut scoped = scope(state);
                                        (o_c)(&mut scoped, on_hover);
                                        embed(state, scoped);
                                    });
                                    r
                                }),
                                on_key: h.2.on_key.map(|o_c| {
                                    let r: KeyHandler<State> = Rc::new(move |state, on_hover| {
                                        let mut scoped = scope(state);
                                        (o_c)(&mut scoped, on_hover);
                                        embed(state, scoped);
                                    });
                                    r
                                }),
                            },
                        )
                    })
                    .collect::<Vec<_>>(),
            );
            ui.ui.images.extend(embedded.ui.images);
            embed(&mut ui.ui.state, embedded.ui.state)
        },
        node,
    )
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
    pub(crate) fn animations_in_progress(&self, now: Instant) -> bool {
        self.view_state.values().any(|v| match v {
            AnimatedView::Rect(animated_rect) => animated_rect.shape.in_progress(now),
            AnimatedView::Text(animated_text) => animated_text.fill.in_progress(now),
            AnimatedView::Circle(animated_circle) => animated_circle.shape.in_progress(now),
        })
    }
}

pub(crate) struct RenderState<'surface> {
    // SAFETY: We MUST drop the surface before the `window`, so the fields
    // must be in this order
    pub(crate) surface: RenderSurface<'surface>,
    pub(crate) window: Arc<Window>,
}
