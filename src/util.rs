use embedded_graphics::{prelude::{Point, Size}, primitives::Circle};

pub fn make_point_f32_rounded(x: f32, y: f32) -> Point {
    Point::new(
        x.round() as i32,
        y.round() as i32,
    )
}
pub fn make_size_f32_rounded(x: f32, y: f32) -> Size {
    Size::new(
        x.round() as u32,
        y.round() as u32,
    )
}
pub fn make_circle_center_radius(center_x: f32, center_y: f32, radius: f32) -> Circle {
    Circle::new(
        make_point_f32_rounded(center_x - radius, center_y - radius),
        (radius * 2.0).round() as u32,
    )
}