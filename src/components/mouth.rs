use core::marker::PhantomData;

use embedded_graphics::prelude::{PixelColor, Point, Size, DrawTarget};
use embedded_graphics::Drawable as DrawableGraphics;
use embedded_graphics::primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, Primitive, StyledDrawable};
use crate::sprite::Sprite;
use crate::util::prepare_sprite_buffer;
use crate::{BasicPaletteContext, ExpressionContext};
use crate::component::Component;
use crate::palette::{Palette, BasicPaletteKey};

pub struct Mouth<'a, Context: MouthContext<'a>> {
    min_width: u32,
    max_width: u32,
    min_height: u32,
    max_height: u32,
    context: PhantomData<&'a Context>,
}

impl<'a, Context: MouthContext<'a>> Mouth<'a, Context> {
    pub fn new(min_width: u32, max_width: u32, min_height: u32, max_height: u32) -> Self {
        Self {
            min_width,
            max_width,
            min_height,
            max_height,
            context: PhantomData::default(),
        }
    }
}

pub trait MouthContext<'a>: BasicPaletteContext<'a> + ExpressionContext {
    fn open_ratio(&self) -> f32;
    fn set_open_ratio(&mut self, value: f32);
    fn breath(&self) -> f32;
    fn set_breath(&mut self, value: f32);
}
pub struct DrawableMouth<Color: PixelColor> {
    bounding_box: Rectangle,
    background_color: Color,
    style: PrimitiveStyle<Color>,
    mouth_rect: Rectangle,
}

impl<Color: PixelColor + Into<Color::Raw> + From<Color::Raw>> DrawableGraphics for DrawableMouth<Color> {
    type Color = Color;
    type Output = ();
    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
        where
            D: embedded_graphics::prelude::DrawTarget<Color = Self::Color> {
        let mut buffer = prepare_sprite_buffer::<Color>(self.bounding_box);
        let mut sprite = Sprite::<Color>::new_unaligned(&mut buffer, self.bounding_box).unwrap();
        sprite.clear(self.background_color).ok();
        self.mouth_rect.into_styled(self.style.clone()).draw(&mut sprite).ok();
        sprite.draw(target)?;
        Ok(())
    }
}

impl <'a, Context: MouthContext<'a>> Component<'a> for Mouth<'a, Context> 
    where Context::Color: From<<Context::Color as PixelColor>::Raw> + Into<<Context::Color as PixelColor>::Raw> 
{
    type Context = Context;
    type Drawable = DrawableMouth<Context::Color>;
    fn render(&self, bounding_rect: Rectangle, context: &'a Self::Context) -> Self::Drawable {
        let foreground_color = context.get_basic_palette().get_color(&BasicPaletteKey::Primary);
        let background_color = context.get_basic_palette().get_color(&BasicPaletteKey::Background);
        let open_ratio = context.open_ratio();
        let breath = context.breath();
        let style = PrimitiveStyleBuilder::new()
            .stroke_color(foreground_color)
            .stroke_width(1)
            .fill_color(foreground_color)
            .build();
        let bounding_box = Rectangle::new(
            bounding_rect.top_left - Point::new((self.max_width/2) as i32 + 3, (self.max_height / 2) as i32 + 3),
            Size::new(self.max_width + 6, self.max_height + 6),
        );
        let h = self.min_height + (((self.max_height - self.min_height) as f32) * open_ratio) as u32;
        let w = self.min_width + (((self.max_width - self.min_width) as f32) * (1.0 - open_ratio)) as u32;
        let x = bounding_rect.top_left.x - (w / 2) as i32;
        let y = bounding_rect.top_left.y - (h / 2) as i32 + (breath * 2.0) as i32;
        let mouth_rect = Rectangle::new(Point::new(x, y), Size::new(w, h));
        Self::Drawable {
            bounding_box,
            background_color,
            style,
            mouth_rect,
        }
    }
}