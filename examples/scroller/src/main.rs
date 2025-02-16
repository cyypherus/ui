use ui::*;
use vello_svg::vello::peniko::color::AlphaColor;

#[derive(Clone, Default)]
struct AppState {
    texts: Vec<String>,
    scroller: ScrollerState,
}

fn main() {
    App::start(
        AppState {
            texts: vec![
                "Rendering must not wait for logic resolution unless explicitly requested.".to_string(),
                "Event dispatch latency is a function of input queuing and frame scheduling.".to_string(),
                "A critical but often overlooked aspect of UI performance is how input events interact with ongoing animations and updates. Consider a scenario where a user is scrolling through a feed while asynchronous content loads in the background. If the rendering pipeline does not correctly prioritize the scroll event over the content updates, the experience feels sluggish. The ideal solution is to partition work intelligently—allowing input handling to take precedence over background updates while still keeping enough flexibility to ensure the UI remains responsive under load. Systems that handle this well create an illusion of seamless interaction, even when multiple processes are competing for resources.".to_string(),
                "Compositional hierarchies introduce deferred constraints on visual updates.".to_string(),
                "The pipeline favors consistency over immediate reflectivity; \nstate transitions must be acknowledged.".to_string(),
                "A view’s presence in the tree does not guarantee its materialization at all times.".to_string(),
                "Preemption allows mid-frame adjustments but requires careful arbitration.".to_string(),
                "Scheduling decisions should account for both animation pacing\nand input sampling windows.".to_string(),
                "Hybrid models allow speculative rendering with rollback on reconciliation failures.".to_string(),
                "The challenge is not just in rendering individual components but in managing the lifecycle of entire UI trees over time. Every frame represents a snapshot of a constantly evolving system where changes must be synchronized without stalling user interactions. When an update is initiated, it must propagate through multiple layers: from logic, through state, into the view model, and finally down to the render layer. Any inefficiency along this pipeline compounds, leading to noticeable lag or instability. The key to maintaining fluidity lies in separating concerns—keeping computation-heavy logic away from the render loop, deferring non-critical updates, and ensuring that only the absolutely necessary state mutations trigger layout recalculations.".to_string(),
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
                "Modern UI architectures must be designed with concurrency in mind from the outset. The moment a system assumes a synchronous execution model, it risks collapsing under real-world conditions where delays are unpredictable. A robust design embraces the reality that tasks will complete at different times, that inputs will arrive asynchronously, and that the UI should never be locked waiting for an operation to finish. This means designing mechanisms for speculative rendering, intelligent state invalidation, and efficient batching. The real challenge is striking the right balance—too much speculative execution leads to wasted effort, but too little results in noticeable delays. The best architectures make these trade-offs seamlessly, adapting dynamically to runtime conditions.".to_string(),
            ],
            scroller: ScrollerState::default(),
        },
        dynamic_node(|_: &mut AppState| {
            let vec = vec![
                space(),
                scroller(
                    id!(),
                    rect(crate::id!())
                        .corner_rounding(10.)
                        .stroke(AlphaColor::from_rgb8(50, 50, 50), 2.)
                        .fill(AlphaColor::from_rgb8(30, 30, 30))
                        .view()
                        .transition_duration(0.)
                        .finish(),
                    binding!(AppState, scroller),
                    scroller_cell,
                ),
                space()
            ];
            let elements = vec;
            row(elements)
            .pad_y(25.)
        }),
    )
}

fn scroller_cell<'n>(
    state: &mut AppState,
    index: usize,
    _id: u64,
) -> Option<Node<'n, RcUi<AppState>>> {
    state.texts.get(index).map(|s| {
        column(vec![
            if index == 0 {
                space().height(10.)
            } else {
                empty()
            },
            stack(vec![
                rect(id!(index as u64))
                    .fill(AlphaColor::from_rgb8(40, 40, 40))
                    .corner_rounding(5.)
                    .view()
                    .transition_duration(0.)
                    .finish(),
                row(vec![
                    text(id!(index as u64), s.clone())
                        .fill(Color::WHITE)
                        .align(TextAlign::Leading)
                        .view()
                        .transition_duration(0.)
                        .finish()
                        .pad(10.),
                    svg(id!(index as u64), "assets/tiger.svg")
                        .view()
                        .transition_duration(0.)
                        .finish()
                        .height(100.)
                        .aspect(1.),
                ]),
            ]),
            space().height(10.),
        ])
        .pad_x(10.)
    })
}
