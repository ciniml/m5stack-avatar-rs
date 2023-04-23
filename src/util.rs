use embedded_graphics::{prelude::{Point, Size}, primitives::{Circle, Ellipse}};

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
pub fn make_ellipse_at_ceter_with_size(center_x: i32, center_y: i32, width: u32, height: u32) -> Ellipse {
    Ellipse::new(
        Point::new(center_x - (width / 2) as i32, center_y - (height / 2) as i32),
        Size::new(width, height),
    )
}