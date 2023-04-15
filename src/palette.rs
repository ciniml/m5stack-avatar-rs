use core::marker::PhantomData;

use embedded_graphics::pixelcolor::PixelColor;
use num_enum::IntoPrimitive;
use variant_count::VariantCount;

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, IntoPrimitive, VariantCount)]
pub enum BasicPaletteKey {
    Primary,
    Secondary,
    Background,
    BalloonForeground,
    BalloonBackground,
}

impl<'a> Into<usize> for &'a BasicPaletteKey {
    fn into(self) -> usize {
        (*self).into()
    }
}

pub trait Palette {
    type Key;
    type Color: PixelColor;
    fn get_color(&self, key: &Self::Key) -> Self::Color;
    fn set_color(&mut self, key: &Self::Key, color: Self::Color);
}

pub struct ArrayPalette<Key, Color: PixelColor, const SIZE: usize> 
    where for<'a> &'a Key: Into<usize>
{
    colors: [Color; SIZE],
    key: PhantomData<Key>,
}

impl<Key, Color: PixelColor + Default, const SIZE: usize> Default for ArrayPalette<Key, Color, SIZE> 
    where for<'a> &'a Key: Into<usize>
{
    fn default() -> Self {
        Self {
            colors: [Default::default(); SIZE],
            key: PhantomData::default(),
        }
    }
}

impl<Key, Color: PixelColor, const SIZE: usize> Palette for ArrayPalette<Key, Color, SIZE> 
    where for<'a> &'a Key: Into<usize>
{
    type Color = Color;
    type Key = Key;
    fn get_color(&self, key: &Self::Key) -> Self::Color {
        let index: usize = key.into();
        self.colors[index]
    }
    fn set_color(&mut self, key: &Self::Key, color: Self::Color) {
        let index: usize = key.into();
        self.colors[index] = color;
    }
}