use std::{cell::RefCell, rc::Rc};
use ui::*;

#[derive(Clone)]
struct State {
    texts: Vec<String>,
    scroller: Rc<RefCell<ScrollerState>>,
}

fn main() {
    App::start(
        State {
            texts: vec![
                "Rendering must not wait for logic resolution unless explicitly requested.".to_string(),
                "Event dispatch latency is a function of input queuing and frame scheduling.".to_string(),
                "A critical but often overlooked aspect of UI performance is how input events interact with ongoing animations and updates. Consider a scenario where a user is scrolling through a feed while asynchronous content loads in the background. If the rendering pipeline does not correctly prioritize the scroll event over the content updates, the experience feels sluggish.".to_string(),
                "Compositional hierarchies introduce deferred constraints on visual updates.".to_string(),
                "The pipeline favors consistency over immediate reflectivity; \nstate transitions must be acknowledged.".to_string(),
                "A view's presence in the tree does not guarantee its materialization at all times.".to_string(),
                "Preemption allows mid-frame adjustments but requires careful arbitration.".to_string(),
                "Scheduling decisions should account for both animation pacing\nand input sampling windows.".to_string(),
                "Hybrid models allow speculative rendering with rollback on reconciliation failures.".to_string(),
                "The challenge is not just in rendering individual components but in managing the lifecycle of entire UI trees over time. Every frame represents a snapshot of a constantly evolving system where changes must be synchronized without stalling user interactions.".to_string(),
                "Timeout thresholds must align with perceptual boundaries, \nnot just system constraints.".to_string(),
                "Metrics must be captured at multiple levels to ensure coherence across update cycles.".to_string(),
                "Even small state mutations can trigger cascading updates, \nwhich must be throttled accordingly.".to_string(),
                "If the tree changes mid-frame, what happens to the pending operations?".to_string(),
                "A single frame drop is perceptible, \nbut a consistent 30ms jitter is often unnoticed.".to_string(),
                "Optimizations that assume a fixed frame budget \nare brittle in asynchronous environments.".to_string(),
                "Not every component needs to respond to every state change, \nonly those within its dependency graph.".to_string(),
                "The renderer should avoid overcommitting resources \nuntil confirmation of stable state.".to_string(),
                "There is a threshold where deferred updates create a feeling of lag, \nwhich must be minimized.".to_string(),
                "Frame timings are not absolute; \nperceptual smoothness is what actually matters.".to_string(),
                "Visual coherence is often more important than immediate accuracy.".to_string(),
                "In extreme cases, dropping frames can be better than introducing stutter.".to_string(),
            ],
            scroller: Rc::new(RefCell::new(ScrollerState::default())),
        },
        |state, app| {
            let texts = state.texts.clone();
            let scroller_state = state.scroller.clone();
            row(vec![
                space(),
                scroller(
                    id!(),
                    Some(
                        rect(id!())
                            .corner_rounding(10.)
                            .stroke(Color::from_rgb8(50, 50, 50), Stroke::new(2.))
                            .fill(Color::from_rgb8(30, 30, 30))
                            .finish(app.ctx()),
                    ),
                    scroller_state,
                    move |index, _id, ctx| {
                        texts.get(index).map(|s| {
                            column(vec![
                                if index == 0 {
                                    space().height(10.)
                                } else {
                                    empty()
                                },
                                stack(vec![
                                    rect(id!(index as u64))
                                        .fill(Color::from_rgb8(40, 40, 40))
                                        .corner_rounding(5.)
                                        .finish(ctx),
                                    row(vec![
                                        text(id!(index as u64), s.clone())
                                            .fill(Color::WHITE)
                                            .align(parley::Alignment::Start)
                                            .wrap()
                                            .finish(ctx)
                                            .pad(10.),
                                        svg(id!(index as u64), include_str!("../../../assets/tiger.svg"))
                                            .finish(ctx)
                                            .height(100.),
                                    ]),
                                ]),
                                space().height(10.),
                            ])
                            .pad_x(10.)
                        })
                    },
                    app.ctx(),
                ),
                space(),
            ])
            .pad_y(25.)
        },
    )
}
