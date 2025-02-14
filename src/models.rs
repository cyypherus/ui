use std::rc::Rc;

use crate::Area;
use vello_svg::vello::kurbo::Point;
use winit::keyboard::{NamedKey, SmolStr};

pub(crate) fn area_contains(area: &Area, point: Point) -> bool {
    let x = point.x;
    let y = point.y;
    if x > area.x as f64
        && y > area.y as f64
        && y < area.y as f64 + area.height as f64
        && x < area.x as f64 + area.width as f64
    {
        return true;
    }
    false
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Key<Str = SmolStr> {
    Named(NamedKey),
    Character(Str),
}

impl Key {
    pub(crate) fn from(value: winit::keyboard::Key) -> Option<Key> {
        match value {
            winit::keyboard::Key::Named(named_key) => Some(Key::Named(named_key)),
            winit::keyboard::Key::Character(c) => Some(Key::Character(c)),
            winit::keyboard::Key::Unidentified(_) => None,
            winit::keyboard::Key::Dead(_) => None,
        }
    }
}

pub struct Binding<State, T> {
    pub get: Rc<dyn Fn(&State) -> T>,
    pub set: Rc<dyn Fn(&mut State, T)>,
}

impl<State, T> Binding<State, T> {
    pub fn new(get: impl Fn(&State) -> T + 'static, set: impl Fn(&mut State, T) + 'static) -> Self {
        Self {
            get: Rc::new(get),
            set: Rc::new(set),
        }
    }
}

impl<State, T> Clone for Binding<State, T> {
    fn clone(&self) -> Self {
        Self {
            get: self.get.clone(),
            set: self.set.clone(),
        }
    }
}

impl<State, T> Binding<State, T> {
    pub fn get(&self, state: &State) -> T {
        (self.get)(state)
    }
    pub fn set(&self, state: &mut State, value: T) {
        (self.set)(state, value)
    }
    pub fn update(&self, state: &mut State, f: impl Fn(&mut T)) {
        let mut temp = (self.get)(state);
        f(&mut temp);
        (self.set)(state, temp)
    }
}
