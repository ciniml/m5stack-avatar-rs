use alloc::vec::Vec;
use embedded_graphics::{prelude::{Point, Size, PixelColor}, primitives::{Circle, Ellipse, Rectangle}};
#[allow(unused)]
use micromath::F32Ext as _;

use crate::sprite::Sprite;

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
pub fn rectangle_union(r1: &Rectangle, r2: &Rectangle) -> Rectangle {
    let top_left = Point::new(r1.top_left.x.min(r2.top_left.x), r1.top_left.y.min(r2.top_left.y));
    let bottom_right = match (r1.bottom_right(), r2.bottom_right()) {
        (Some(bottom_right), None) => bottom_right,
        (None, Some(bottom_right)) => bottom_right,
        (Some(bottom_right_r1), Some(bottom_right_r2)) => {
            Point::new(bottom_right_r1.x.max(bottom_right_r2.x), bottom_right_r1.y.max(bottom_right_r2.y))
        },
        (None, None) => {
            return Rectangle::zero();
        },
    };
    let size = Size::new((bottom_right.x - top_left.x) as u32, (bottom_right.y - top_left.y) as u32);
    Rectangle::new(top_left, size)
}

pub fn rectangle_union_all(rectangles: &[Rectangle]) -> Option<Rectangle> {
    let mut rectangle = None;
    for rect in rectangles {
        if let Some(prev_rect) = rectangle {
            rectangle = Some(rectangle_union(&prev_rect, rect));
        } else {
            rectangle = Some(*rect);
        }
    }
    rectangle
}

pub fn prepare_sprite_buffer<C: PixelColor>(bounding_box: Rectangle) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.resize(Sprite::<C>::unaligned_buffer_size(bounding_box.size.width, bounding_box.size.height), 0);
        buffer
}