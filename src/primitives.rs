use crate::Area;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

pub(crate) fn area_contains(area: &Area, point: Point) -> bool {
    let x = point.x;
    let y = point.y;
    if x > area.x && y > area.y && y < area.y + area.height && x < area.x + area.width {
        return true;
    }
    false
}

impl Point {
    pub(crate) fn distance(&self, to: Point) -> f32 {
        ((to.x - self.x).powf(2.) + (to.y - self.y).powf(2.)).sqrt()
    }
}
