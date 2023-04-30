use embedded_graphics::{Drawable as DrawableGraphics, prelude::{PixelColor, IntoStorage}};

use crate::{Palette, BasicPaletteKey};

pub trait Component<'a> {
    type Drawable: DrawableGraphics;
    type Context: BasicPaletteContext<'a>;
    fn render(&self, bounding_rect: embedded_graphics::primitives::Rectangle, context: &'a Self::Context) -> Self::Drawable;
}

pub trait BasicPaletteContext<'a> {
    type Color: PixelColor + From<<<Self as BasicPaletteContext<'a>>::Color as PixelColor>::Raw> + Into<<<Self as BasicPaletteContext<'a>>::Color as PixelColor>::Raw> + 'a;
    type BasicPalette: Palette<Key = BasicPaletteKey, Color = Self::Color>;
    fn get_basic_palette(&self) -> &Self::BasicPalette;
}