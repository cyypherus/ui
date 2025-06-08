use crate::{gestures::InteractionHandler, view::AnimatedView, Editor, GestureHandler};
pub use backer::models::*;
use backer::Node;
use lilt::Animated;
use parley::{FontContext, Layout, LayoutContext};
use std::{cell::Cell, sync::Arc, time::Instant};
use std::{collections::HashMap, rc::Rc};
use vello_svg::vello::Scene;
use vello_svg::vello::{peniko::Brush, util::RenderSurface};
use winit::{event::Modifiers, window::Window};

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

// pub fn scoper<'n, State, Scoped: 'n + 'static>(
//     scope: impl Fn(&mut State) -> Scoped + 'static + Copy,
//     embed: impl Fn(&mut State, Scoped) + 'static + Copy,
//     node: Node<'n, RcUi<Scoped>>,
// ) -> Node<'n, RcUi<State>> {
//     backer::nodes::scope_owned(
//         move |ui: &mut RcUi<State>| RcUi {
//             ui: Ui {
//                 state: scope(&mut ui.ui.state),
//                 gesture_handlers: Vec::new(),
//                 cx: ui.ui.cx.take(),
//                 now: ui.ui.now,
//                 editor: ui.ui.editor.take(),
//             },
//         },
//         move |ui: &mut RcUi<State>, mut embedded: RcUi<Scoped>| {
//             ui.ui.cx = embedded.ui.cx.take();
//             ui.ui.gesture_handlers.append(
//                 &mut std::mem::take(&mut embedded.ui.gesture_handlers)
//                     .into_iter()
//                     .map(|h| {
//                         (
//                             h.0,
//                             h.1,
//                             GestureHandler {
//                                 interaction_type: h.2.interaction_type,
//                                 interaction_handler: h.2.interaction_handler.map(|o_c| {
//                                     let r: InteractionHandler<State> =
//                                         Rc::new(move |ui, interaction| {
//                                             let child_cx = ui.ui.cx.take();
//                                             let mut scoped = RcUi {
//                                                 ui: Ui {
//                                                     state: scope(&mut ui.ui.state),
//                                                     gesture_handlers: Vec::new(),
//                                                     cx: child_cx,
//                                                     now: ui.ui.now,
//                                                     editor: ui.ui.editor.take(),
//                                                 },
//                                             };
//                                             (o_c)(&mut scoped, interaction);
//                                             ui.ui.cx = scoped.ui.cx.take();
//                                             ui.ui.editor = scoped.ui.editor.take();
//                                             embed(&mut ui.ui.state, scoped.ui.state);
//                                         });
//                                     r
//                                 }),
//                             },
//                         )
//                     })
//                     .collect::<Vec<_>>(),
//             );
//             embed(&mut ui.ui.state, embedded.ui.state)
//         },
//         node,
//     )
// }

// impl<State> Ui<State> {
//     pub(crate) fn cx(&mut self) -> &mut UiCx {
//         self.cx.as_mut().unwrap()
//     }
// }
