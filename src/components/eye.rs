use core::marker::PhantomData;
#[allow(unused)]
use micromath::F32Ext as _;

use embedded_graphics::prelude::{PixelColor, Point, Size, DrawTarget};
use embedded_graphics::Drawable as DrawableGraphics;
use embedded_graphics::primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Circle, Rectangle, Primitive, Triangle};
use crate::sprite::Sprite;
use crate::util::prepare_sprite_buffer;
use crate::{BasicPaletteContext, ExpressionContext, Expression};
use crate::component::Component;
use crate::palette::{Palette, BasicPaletteKey};

use super::mouth::MouthContext;

pub struct Eye<'a, Context: EyeContext<'a>> {
    radius: f32,
    is_left: bool,
    context: PhantomData<&'a Context>,
}

impl<'a, Context: EyeContext<'a>> Eye<'a, Context> {
    pub fn new(radius: f32, is_left: bool) -> Self {
        Self {
            radius,
            is_left,
            context: PhantomData::default(),
        }
    }
}

pub trait EyeContext<'a>: BasicPaletteContext<'a> +  GazeContext + ExpressionContext + MouthContext<'a> {
    fn open_ratio(&self) -> f32;
    fn set_open_ratio(&mut self, value: f32);
}

pub trait GazeContext {
    fn horizontal(&self) -> f32;
    fn set_horizontal(&mut self, value: f32);
    fn vertical(&self) -> f32;
    fn set_vertical(&mut self, value: f32);
}


pub struct DrawableEye<Color: PixelColor> {
    bounding_box: Rectangle,
    background_color: Color,
    style: PrimitiveStyle<Color>,
    mask_style: PrimitiveStyle<Color>,
    open_eye_main: Option<Circle>,
    open_eye_triangle: Option<Triangle>,
    open_eye_happy_circle: Option<Circle>,
    open_eye_half_mask: Option<Rectangle>,
    close_eye: Option<Rectangle>,
}

impl<Color: PixelColor + Into<Color::Raw> + From<Color::Raw>> DrawableGraphics for DrawableEye<Color> {
    type Color = Color;
    type Output = ();
    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
        where
            D: embedded_graphics::prelude::DrawTarget<Color = Self::Color> {
        let mut buffer = prepare_sprite_buffer::<Color>(self.bounding_box);
        let mut sprite = Sprite::<Color>::new_unaligned(&mut buffer, self.bounding_box).unwrap();
        sprite.clear(self.background_color).ok();
        self.open_eye_main.map_or(Ok(()), |p| p.into_styled(self.style.clone()).draw(&mut sprite)).ok();
        self.open_eye_triangle.map_or(Ok(()), |p| p.into_styled(self.mask_style.clone()).draw(&mut sprite)).ok();
        self.open_eye_happy_circle.map_or(Ok(()), |p| p.into_styled(self.mask_style.clone()).draw(&mut sprite)).ok();
        self.open_eye_half_mask.map_or(Ok(()), |p| p.into_styled(self.mask_style.clone()).draw(&mut sprite)).ok();
        self.close_eye.map_or(Ok(()), |p| p.into_styled(self.style.clone()).draw(&mut sprite)).ok();
        sprite.draw(target)?;
        Ok(())
    }
}

impl <'a, Context: EyeContext<'a>> Component<'a> for Eye<'a, Context> 
    where Context::Color: From<<Context::Color as PixelColor>::Raw> + Into<<Context::Color as PixelColor>::Raw> 
{
    type Context = Context;
    type Drawable = DrawableEye<Context::Color>;
    fn render(&self, bounding_rect: Rectangle, context: &'a Self::Context) -> Self::Drawable {
        let foreground_color = context.get_basic_palette().get_color(&BasicPaletteKey::Primary);
        let background_color = context.get_basic_palette().get_color(&BasicPaletteKey::Background);
        let open_ratio = EyeContext::open_ratio(context);
        let breath_offset = context.breath();
        let style = PrimitiveStyleBuilder::new()
            .stroke_color(foreground_color)
            .stroke_width(1)
            .fill_color(foreground_color)
            .build();
        let mask_style = PrimitiveStyleBuilder::new()
            .stroke_color(background_color)
            .stroke_width(1)
            .fill_color(background_color)
            .build();
        let center = bounding_rect.center();
        let x = center.x as f32 + breath_offset * 3.0;
        let y = center.y as f32 + breath_offset * 3.0;
        let bounding_box = Rectangle::new(
            center - Point::new(self.radius.ceil() as i32 + 3 + 3, self.radius.ceil() as i32 + 3 + 3),
            Size::new((self.radius * 2.0 + 12.0).ceil() as u32, (self.radius * 2.0 + 12.0).ceil() as u32),
        );
        let offset_x = context.horizontal() * 3.0;
        let offset_y = context.vertical() * 3.0;
        let expression = context.expression();
        if open_ratio > 0.0 {
            let body = Circle::new(Point::new((x + offset_x - self.radius) as i32, (y + offset_y - self.radius) as i32), (self.radius * 2.0) as u32);
            match expression {
                Expression::Angry | Expression::Sad => {
                    let x0 = x + offset_x - self.radius;
                    let y0 = y + offset_y - self.radius;
                    let x1 = x0 + self.radius * 2.0;
                    let y1 = y0;
                    let x2 = if self.is_left ^ (expression == Expression::Angry) { x0 } else {  x1 };
                    let y2 = y0 + self.radius;
                    let triangle = Triangle::new(Point::new(x0 as i32, y0 as i32), Point::new(x1 as i32, y1 as i32), Point::new(x2 as i32, y2 as i32));
                    Self::Drawable {
                        bounding_box,
                        background_color,
                        style,
                        mask_style,
                        open_eye_main: Some(body),
                        open_eye_triangle: Some(triangle),
                        open_eye_happy_circle: None,
                        open_eye_half_mask: None,
                        close_eye: None,
                    }
                },
                Expression::Happy | Expression::Sleepy => {
                    let x0 = x + offset_x - self.radius;
                    let y0 = y + offset_y - self.radius + if expression == Expression::Happy { self.radius } else { 0.0 };
                    let w = self.radius * 2.0 + 4.0;
                    let h = self.radius + 2.0;
                    let open_eye_happy_circle = if expression == Expression::Happy {
                        let radius = self.radius / 1.5;
                        Some(Circle::new(Point::new((x + offset_x - radius).round() as i32, (y + offset_y - radius).round() as i32), (radius * 2.0).round() as u32))
                    } else {
                        None
                    };
                    let open_eye_half_mask = Some(Rectangle::new(
                        Point::new(x0 as i32, y0 as i32),
                        Size::new(w as u32, h as u32)
                    ));
                    Self::Drawable {
                        bounding_box,
                        background_color,
                        style,
                        mask_style,
                        open_eye_main: Some(body),
                        open_eye_triangle: None,
                        open_eye_happy_circle,
                        open_eye_half_mask,
                        close_eye: None,
                    }
                },
                _ => {
                    Self::Drawable {
                        bounding_box,
                        background_color,
                        style,
                        mask_style,
                        open_eye_main: Some(body),
                        open_eye_triangle: None,
                        open_eye_happy_circle: None,
                        open_eye_half_mask: None,
                        close_eye: None,
                    }
                }
            }
        } else {
            let x1 = x - self.radius + offset_x;
            let y1 = y - 2.0 + offset_y;
            let w = self.radius * 2.0;
            let h = 4.0f32;
            let close_eye = Some(Rectangle::new(Point::new(x1 as i32, y1 as i32), Size::new(w as u32, h as u32)));
            Self::Drawable {
                bounding_box,
                background_color,
                style,
                mask_style,
                open_eye_main: None,
                open_eye_triangle: None,
                open_eye_happy_circle: None,
                open_eye_half_mask: None,
                close_eye,
            }
        }
    }
}