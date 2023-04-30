use core::marker::PhantomData;

use embedded_graphics::prelude::{PixelColor, Point, Size, DrawTarget};
use embedded_graphics::Drawable as DrawableGraphics;
use embedded_graphics::primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Circle, Rectangle, Primitive, Triangle};
use crate::sprite::Sprite;
use crate::util::{make_point_f32_rounded, prepare_sprite_buffer};
use crate::{BasicPaletteContext, ExpressionContext, Expression};
use crate::component::Component;
use crate::palette::{Palette, BasicPaletteKey};

use super::mouth::MouthContext;

pub struct Eyeblow<'a, Context: BasicPaletteContext<'a> + ExpressionContext> {
    width: u32,
    height: u32,
    is_left: bool,
    context: PhantomData<&'a Context>,
}

impl<'a, Context: BasicPaletteContext<'a> + ExpressionContext> Eyeblow<'a, Context> {
    pub fn new(width: u32, height: u32, is_left: bool) -> Self {
        Self {
            width,
            height,
            is_left,
            context: PhantomData::default(),
        }
    }
}

pub struct DrawableEyeblow<Color: PixelColor> {
    bounding_box: Rectangle,
    background_color: Color,
    style: PrimitiveStyle<Color>,
    angry_sad_triangles: Option<(Triangle, Triangle)>,
    other_rect: Option<Rectangle>,
}

impl<Color: PixelColor + Into<Color::Raw> + From<Color::Raw>> DrawableGraphics for DrawableEyeblow<Color> {
    type Color = Color;
    type Output = ();
    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
        where
            D: embedded_graphics::prelude::DrawTarget<Color = Self::Color> {
        let mut buffer = prepare_sprite_buffer::<Color>(self.bounding_box);
        let mut sprite = Sprite::<Color>::new_unaligned(&mut buffer, self.bounding_box).unwrap();
        sprite.clear(self.background_color).ok();
        self.angry_sad_triangles.map_or(Ok(()), |p| {
            p.0.into_styled(self.style.clone()).draw(&mut sprite).ok();
            p.1.into_styled(self.style.clone()).draw(&mut sprite).ok();
            Ok(())
        })?;
        self.other_rect.map_or(Ok(()), |p| p.into_styled(self.style.clone()).draw(&mut sprite)).ok();
        sprite.draw(target)?;
        Ok(())
    }
}

impl <'a, Context: BasicPaletteContext<'a> + ExpressionContext + MouthContext<'a>> Component<'a> for Eyeblow<'a, Context> 
    where Context::Color: From<<Context::Color as PixelColor>::Raw> + Into<<Context::Color as PixelColor>::Raw> 
{
    type Context = Context;
    type Drawable = DrawableEyeblow<Context::Color>;
    fn render(&self, bounding_rect: Rectangle, context: &Self::Context) -> Self::Drawable {
        let foreground_color = context.get_basic_palette().get_color(&BasicPaletteKey::Primary);
        let background_color = context.get_basic_palette().get_color(&BasicPaletteKey::Background);
        let style = PrimitiveStyleBuilder::new()
            .stroke_color(foreground_color)
            .stroke_width(1)
            .fill_color(foreground_color)
            .build();
        
        let breath_offset = context.breath() * 3.0;
        let center = bounding_rect.center();

        let bounding_box = Rectangle::new(
            center - Point::new((self.width / 2) as i32 + 3 + 3, (self.height / 2) as i32 + 3 + 5),
            Size::new(self.width + (3 + 3) * 2, self.height + (3 + 5) * 2, ),
        );

        let x = center.x as f32 + breath_offset;
        let y = center.y as f32 + breath_offset;
        let expression = context.expression();
        let width = self.width as f32;
        let height = self.height as f32;
        match expression {
            Expression::Angry | Expression::Sad => {
                let aspect = if self.is_left ^ (expression == Expression::Sad) { -1.0 } else { 1.0 };
                let dx = aspect * 3.0;
                let dy = aspect * 5.0;
                let x1 = x - width / 2.0;
                let x2 = x1 - dx;
                let x4 = x + width / 2.0;
                let x3 = x4 + dx;
                let y1 = y - height / 2.0 - dy;
                let y2 = y + height / 2.0 - dy;
                let y3 = y - height / 2.0 + dy;
                let y4 = y + height / 2.0 + dy;
                let triangle1 = Triangle::new(
                    make_point_f32_rounded(x1, y1), 
                    make_point_f32_rounded(x2, y2),
                    make_point_f32_rounded(x3, y3),
                );
                let triangle2 = Triangle::new(
                    make_point_f32_rounded(x2, y2), 
                    make_point_f32_rounded(x3, y3),
                    make_point_f32_rounded(x4, y4),
                );
                Self::Drawable {
                    bounding_box,
                    background_color,
                    style,
                    angry_sad_triangles: Some((triangle1, triangle2)),
                    other_rect: None,
                }
            },
            _ => {
                let x1 = x - width / 2.0;
                let y1 = if expression == Expression::Happy { y - height / 2.0 - 5.0 } else { y - height / 2.0 };
                let rect = Rectangle::new(make_point_f32_rounded(x1, y1), Size::new(self.width, self.height));
                Self::Drawable {
                    bounding_box,
                    background_color,
                    style,
                    angry_sad_triangles: None,
                    other_rect: Some(rect),
                }
            },
        }
    }
}