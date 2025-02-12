use crate::{clipping, rect, Binding, RcUi};
use backer::{
    models::Area,
    nodes::{area_reader, column, stack},
    Node,
};
use vello_svg::vello::peniko::{color::AlphaColor, Color};

#[derive(Debug, Clone, Default)]
pub struct ScrollerState {
    visible_window: Vec<Element>,
    dt: f32,
    compensated: f32,
    offset: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Element {
    height: f32,
    index: usize,
}

impl ScrollerState {
    fn fill_forwards<State, CellFn>(
        &mut self,
        state: &mut State,
        available_area: Area,
        id: u64,
        cell: CellFn,
    ) where
        CellFn: Fn(&mut State, usize, u64, Area) -> Option<f32> + Copy,
    {
        let mut current_height = self.visible_window.iter().fold(0., |acc, e| acc + e.height);
        let mut index = self.visible_window.last().map(|l| l.index).unwrap_or(0) + {
            if self.visible_window.is_empty() {
                0
            } else {
                1
            }
        };
        while current_height + self.compensated < available_area.height {
            if let Some(added_height) = cell(state, index, id, available_area) {
                current_height += added_height;
                self.visible_window.push(Element {
                    height: added_height,
                    index,
                });
                index += 1;
            } else {
                break;
            }
        }
    }
    fn fill_backwards<State, CellFn>(
        &mut self,
        state: &mut State,
        available_area: Area,
        id: u64,
        cell: CellFn,
    ) where
        CellFn: Fn(&mut State, usize, u64, Area) -> Option<f32> + Copy,
    {
        let mut current_height = self.visible_window.iter().fold(0., |acc, e| acc + e.height);
        while let Some((index, true, true)) = self.visible_window.first().map(|f| {
            (
                f.index,
                current_height + self.compensated < available_area.height,
                f.index > 0,
            )
        }) {
            if let Some(added_height) = cell(state, index - 1, id, available_area) {
                current_height += added_height;
                self.visible_window.insert(
                    0,
                    Element {
                        height: added_height,
                        index: index - 1,
                    },
                );
            } else {
                break;
            }
        }
    }
    pub fn update<State, CellFn>(
        &mut self,
        available_area: Area,
        state: &mut State,
        id: u64,
        cell: CellFn,
    ) where
        CellFn: Fn(&mut State, usize, u64, Area) -> Option<f32> + Copy,
    {
        if self.visible_window.is_empty() {
            self.fill_forwards(state, available_area, id, cell);
        }
        dbg!(&self.dt);
        if self.dt != 0. {
            if self.dt.is_sign_negative() {
                self.compensated += self.dt;
                self.dt = 0.;
                // let mut limit = false;
                while let Some(true) = self
                    .visible_window
                    .first()
                    .map(|first| first.height < -self.compensated && -self.compensated > 0.)
                {
                    let removed = self.visible_window.remove(0);
                    self.compensated += removed.height;
                }

                self.fill_forwards(state, available_area, id, cell);
            } else if self.dt.is_sign_positive() {
                self.compensated += self.dt;
                self.dt = 0.;
                while let (Some((last_height, true)), Some(true)) = (
                    self.visible_window.last().map(|last| {
                        (
                            last.height,
                            last.height < self.compensated && self.compensated > 0.,
                        )
                    }),
                    self.visible_window.first().map(|f| {
                        if f.index != 0 {
                            true
                        } else {
                            self.compensated = 0.;
                            false
                        }
                    }),
                ) {
                    self.visible_window.pop();
                    self.compensated -= last_height;
                }
                self.fill_backwards(state, available_area, id, cell);
            }
        }
        self.offset = -(available_area.height
            - self.visible_window.iter().fold(0., |acc, e| acc + e.height))
            * 0.5;
    }
}

pub fn scroller<'n, State: 'static, CellFn>(
    id: u64,
    scroller: Binding<State, ScrollerState>,
    cell: CellFn,
) -> Node<'n, RcUi<State>>
where
    CellFn: for<'x> Fn(&'x mut State, usize, u64) -> Option<Node<'n, RcUi<State>>> + 'static + Copy,
{
    stack(vec![
        rect(crate::id!())
            .corner_rounding(30.)
            .fill(AlphaColor::from_rgb8(30, 30, 30))
            .stroke(Color::WHITE, 1.)
            .view()
            .on_scroll(move |s, dt| {
                let mut sc = scroller.get(s);
                sc.dt += dt.y;
                scroller.set(s, sc);
            })
            .finish(),
        // clipping(
        //     |area| {
        //         RoundedRect::from_origin_size(
        //             Point::new(area.x.into(), area.y.into()),
        //             Size::new(area.width.into(), area.height.into()),
        //             30.,
        //         )
        //         .to_path(0.001)
        //     },
        area_reader::<RcUi<State>>(move |area, state| {
            let mut scroller_state = scroller.get(&state.ui.state);
            scroller_state.update(area, state, id, |state, index, id, area| {
                cell(&mut state.ui.state, index, id)?.min_height(area, state)
            });
            let window = &scroller_state.visible_window;
            let mut cells = Vec::new();
            for element in window {
                if let Some(cell) = cell(&mut state.ui.state, element.index, id) {
                    cells.push(cell);
                }
            }
            let offset = scroller_state.offset;
            let comp = scroller_state.compensated;
            scroller.set(&mut state.ui.state, scroller_state);
            column(cells)
                //
                .offset_y(offset + comp)
                .height(1.)
        })
        .expand(),
        // ),
    ])
}
// #[cfg(test)]
// mod tests {
//     use super::*;

//     fn cell_fn(_: &mut (), _: usize, _: u64, _: Area) -> Option<f32> {
//         Some(10.0)
//     }

//     #[test]
//     fn test_concise_update() {
//         let mut scroller = ScrollerState {
//             visible_window: vec![],
//             dt: 0.,
//             compensated_top: 5.0,
//             compensated_bottom: 5.0,
//         };

//         let available_area = Area {
//             x: 0.0,
//             y: 0.0,
//             width: 0.0,
//             height: 20.0,
//         };

//         let mut dummy_state = ();

//         run_update_and_assert(
//             &mut scroller,
//             0.0,
//             available_area,
//             42,
//             &mut dummy_state,
//             cell_fn,
//             vec![
//                 Element {
//                     height: 10.0,
//                     index: 0,
//                 },
//                 Element {
//                     height: 10.0,
//                     index: 1,
//                 },
//             ],
//             5.0,
//             0.0,
//         );
//     }

//     #[allow(clippy::too_many_arguments)]
//     fn run_update_and_assert<State, F>(
//         scroller: &mut ScrollerState,
//         new_dt: f32,
//         available_area: Area,
//         id: u64,
//         state: &mut State,
//         cell_fn: F,
//         expected_visible_window: Vec<Element>,
//         expected_compensated_top: f32,
//         expected_compensated_bottom: f32,
//     ) where
//         F: Fn(&mut State, usize, u64, Area) -> Option<f32> + Copy,
//     {
//         scroller.dt = new_dt;
//         scroller.update(available_area, state, id, cell_fn);
//         assert_eq!(
//             scroller.visible_window, expected_visible_window,
//             "visible_window mismatch"
//         );
//         assert_eq!(
//             scroller.compensated_top, expected_compensated_top,
//             "compensated_top mismatch"
//         );
//         assert_eq!(
//             scroller.compensated_bottom, expected_compensated_bottom,
//             "compensated_bottom mismatch"
//         );
//     }
// }
