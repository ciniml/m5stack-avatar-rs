use core::marker::PhantomData;

use alloc::vec::Vec;
use embedded_graphics::{prelude::{PixelColor, Point, Drawable as DrawableGraphics, Dimensions, Size}, primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, Ellipse, Triangle, StyledDrawable, Primitive}, text::{Text, renderer::TextRenderer}, mono_font::{MonoTextStyle, iso_8859_14::FONT_6X9, ascii::FONT_10X20}, transform::Transform};

use crate::{sprite::Sprite, BasicPaletteContext, Component, Palette, BasicPaletteKey, util::{make_ellipse_at_ceter_with_size, rectangle_union, prepare_sprite_buffer}};


pub struct DrawableBalloon<'a, Color: PixelColor, TextStyle> {
    style: PrimitiveStyle<Color>,
    text: Option<Text<'a, TextStyle>>,
    ellipse_outer: Option<Ellipse>,
    triangle_outer: Option<Triangle>,
    bounding_box: Rectangle,
}

pub trait BalloonContext<'a> : BasicPaletteContext<'a> {
    fn text(&self) -> Option<&str>;
    fn set_text(&mut self, string: Option<&str>);
}

impl<'a, Color: PixelColor + Into<Color::Raw> + From<Color::Raw>, TextStyle: TextRenderer<Color = Color>> DrawableGraphics for DrawableBalloon<'a, Color, TextStyle> 
{
    type Color = Color;
    type Output = ();
    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
        where
            D: embedded_graphics::prelude::DrawTarget<Color = Self::Color> {
        match (&self.ellipse_outer, &self.triangle_outer, &self.text) {
            (Some(ellipse_outer), Some(triangle_outer), Some(text)) => {
                let mut buffer = prepare_sprite_buffer::<Color>(self.bounding_box);
                let mut sprite = Sprite::<Color>::new_unaligned(&mut buffer, self.bounding_box).unwrap();
                ellipse_outer.draw_styled(&self.style, &mut sprite);
                triangle_outer.draw_styled(&self.style, &mut sprite);
                text.draw(&mut sprite);
                sprite.draw(target)?;
            },
            _ => {}
        }
        Ok(())
    }
}

pub struct Balloon<'a, Context: BalloonContext<'a>> {
    context: PhantomData<&'a Context>,
}

impl <'a, Context: BalloonContext<'a>> Balloon<'a, Context> {
    pub fn new() -> Self {
        Self {
            context: PhantomData {},
        }
    }
}

impl <'a, Context: BalloonContext<'a>> Component<'a> for Balloon<'a, Context> 
    where Context::Color: From<<Context::Color as PixelColor>::Raw> + Into<<Context::Color as PixelColor>::Raw> 
{
    type Context = Context;
    type Drawable = DrawableBalloon<'a, Context::Color, MonoTextStyle<'static, Context::Color>>;
    fn render(&self, bounding_rect: Rectangle, context: &'a Self::Context) -> Self::Drawable {
        let foreground_color = context.get_basic_palette().get_color(&BasicPaletteKey::BalloonForeground);
        let style = PrimitiveStyleBuilder::new()
            .stroke_color(foreground_color)
            .stroke_width(1)
            .build();
        let cx = 240;
        let cy = 220;
        let bounding_box = Rectangle::new(Point::new(0, cy - 42), Size::new(320, 240 - (cy - 42) as u32));
        if let Some(text) = context.text() {
            let font = &FONT_10X20;
            let baseline = font.baseline;
            let spacing_width = if text.is_empty() { 0 } else { font.character_spacing * (text.len() as u32 - 1) };
            let text_width = font.character_size.width * (text.len() as u32) + spacing_width;
            let text_height = font.character_size.height;
            let ellipse_outer = Some(make_ellipse_at_ceter_with_size(cx - 20, cy, text_width + 12, text_height * 2 + 2));
            let triangle_outer = Some(Triangle::new(
                Point::new(cx - 62, cy - 42),
                Point::new(cx - 8, cy - 20),
                Point::new(cx - 41, cy - 18),
            ));
            let character_style = MonoTextStyle::new(font, foreground_color);
            let text = Text::new(text, Point::new(cx - text_width as i32 / 2 - 20, cy + (baseline - text_height / 2) as i32), character_style);
            Self::Drawable {
                style,
                text: Some(text),
                ellipse_outer,
                triangle_outer,
                bounding_box,
            }
        } else {
            Self::Drawable {
                style,
                text: None,
                ellipse_outer: None,
                triangle_outer: None,
                bounding_box,
            }
        }
    }
}