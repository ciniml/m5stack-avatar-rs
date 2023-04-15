use embedded_graphics::{Drawable as DrawableGraphics, prelude::PixelColor};

use crate::{Palette, BasicPaletteKey};

pub trait Component {
    type Drawable: DrawableGraphics;
    type Context: BasicPaletteContext;
    fn render(&self, bounding_rect: embedded_graphics::primitives::Rectangle, context: &Self::Context) -> Self::Drawable;
}

pub trait BasicPaletteContext {
    type Color: PixelColor;
    type BasicPalette: Palette<Key = BasicPaletteKey, Color = Self::Color>;
    fn get_basic_palette(&self) -> &Self::BasicPalette;
}