// use std::{
//     hash::{DefaultHasher, Hash, Hasher},
//     marker::PhantomData,
// };

// use crate::{ui::UiCtx, view::ViewTrait, Ui};
// use backer::{models::Area, transitions::TransitionDrawable, Layout, Node};

// pub type Scope<'s, State, SubState> = fn(
//     &Layout<Ui<'s, SubState>>,
//     &mut Ui<'s, State>,
//     Area,
//     fn(&Layout<Ui<'s, SubState>>, Area, &mut Ui<'s, SubState>),
// );

// pub struct CustomView<'s, State, SubState> {
//     id: u64,
//     scope: Scope<'s, State, SubState>,
//     tree: Layout<Ui<'s, SubState>>,
//     _p: PhantomData<State>,
// }

// impl<'s, State, SubState> CustomView<'s, State, SubState> {
//     pub fn new(
//         id: String,
//         scope: fn(
//             &Layout<Ui<'s, SubState>>,
//             &mut Ui<'s, State>,
//             Area,
//             fn(&Layout<Ui<'s, SubState>>, Area, &mut Ui<'s, SubState>),
//         ),
//         tree: impl Fn(&mut Ui<SubState>) -> Node<Ui<'s, SubState>> + 'static,
//     ) -> Self {
//         let mut hasher = DefaultHasher::new();
//         id.hash(&mut hasher);
//         Self {
//             id: hasher.finish(),
//             scope,
//             tree: Layout::new(tree),
//             _p: PhantomData,
//         }
//     }
// }

// impl<'s, State, SubState> TransitionDrawable<UiCtx<'s, State>> for CustomView<'s, State, SubState> {
//     fn draw_interpolated(
//         &mut self,
//         area: backer::models::Area,
//         state: &mut Ui<'s, State>,
//         _visible: bool,
//         _visible_amount: f32,
//     ) {
//         (self.scope)(&self.tree, state, area, |tree, area_a, with_scoped| {
//             tree.draw(area_a, with_scoped);
//         });
//     }

//     fn constraints(
//         &self,
//         _available_area: backer::models::Area,
//         _state: &mut Ui<'s, State>,
//     ) -> Option<backer::SizeConstraints> {
//         None
//     }
//     fn id(&self) -> &u64 {
//         &self.id
//     }
//     fn easing(&self) -> backer::Easing {
//         lilt::Easing::EaseOut
//     }
//     fn duration(&self) -> f32 {
//         0.
//     }
//     fn delay(&self) -> f32 {
//         0.
//     }
// }

// impl<'s, State, SubState> ViewTrait<'s, State> for CustomView<'s, State, SubState> {
//     fn view(self, _ui: &mut UiCtx<State>, node: Node<Ui<'s, State>>) -> Node<Ui<'s, State>> {
//         node
//     }
// }
