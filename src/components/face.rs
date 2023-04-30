use core::str::FromStr;

use embedded_graphics::prelude::{PixelColor, Size, Point, Drawable as DrawableGraphics, IntoStorage};
use embedded_graphics::primitives::Rectangle;
use rand_core::SeedableRng;

use crate::{Expression, ArrayPalette, BasicPaletteKey, BasicPaletteContext, ExpressionContext, Component, Palette};
use crate::components::eye::{Eye, EyeContext, GazeContext};
use crate::components::mouth::{Mouth, MouthContext};

use super::balloon::BalloonContext;
use super::eye::DrawableEye;
use super::eyeblow::{Eyeblow, DrawableEyeblow};
use super::mouth::DrawableMouth;

pub trait RandomGeneratorContext {
    type Rng: rand_core::RngCore;
    fn rng(&mut self) -> &mut Self::Rng;
}

pub struct DrawContext<Color: PixelColor, String> {
    pub expression: Expression,
    pub breath: f32,
    pub gaze_horizontal: f32,
    pub gaze_vertical: f32,
    pub eye_open_ratio: f32,
    pub mouth_open_ratio: f32,
    pub palette: ArrayPalette<BasicPaletteKey, Color, {BasicPaletteKey::VARIANT_COUNT}>,
    pub rng: rand_xorshift::XorShiftRng,
    pub text: Option<String>,
}

impl<Color: PixelColor + Default, String> Default for DrawContext<Color, String> {
    fn default() -> Self {
        Self {
            expression: Expression::Neutral,
            breath: 0.0,
            gaze_horizontal: 0.0,
            gaze_vertical: 0.0,
            eye_open_ratio: 1.0,
            mouth_open_ratio: 0.0,
            palette: ArrayPalette::default(),
            rng: rand_xorshift::XorShiftRng::from_seed([0u8; 16]),
            text: None,
        }
    }
}

impl<Color: PixelColor, String> RandomGeneratorContext for DrawContext<Color, String> {
    type Rng = rand_xorshift::XorShiftRng;
    fn rng(&mut self) -> &mut rand_xorshift::XorShiftRng {
        &mut self.rng
    }
}

impl<'a, Color: PixelColor + From<Color::Raw> + Into<Color::Raw> + 'a, String> BasicPaletteContext<'a> for DrawContext<Color, String> {
    type BasicPalette = ArrayPalette<BasicPaletteKey, Color, {BasicPaletteKey::VARIANT_COUNT}>;
    type Color = Color;
    fn get_basic_palette(&self) -> &Self::BasicPalette {
        &self.palette
    }
}

impl<Color: PixelColor, String> ExpressionContext for DrawContext<Color, String> {
    fn expression(&self) -> Expression {
        self.expression
    }
}

impl<Color: PixelColor, String> GazeContext for DrawContext<Color, String> {
    fn horizontal(&self) -> f32 {
        self.gaze_horizontal
    }
    fn set_horizontal(&mut self, value: f32) {
        self.gaze_horizontal = value;
    }
    fn vertical(&self) -> f32 {
        self.gaze_vertical
    }
    fn set_vertical(&mut self, value: f32) {
        self.gaze_vertical = value;
    }
}

impl<'a, Color: PixelColor + From<Color::Raw> + Into<Color::Raw> + 'a, String> EyeContext<'a> for DrawContext<Color, String> {
    fn open_ratio(&self) -> f32 {
        self.eye_open_ratio
    }
    fn set_open_ratio(&mut self, value: f32) {
        self.eye_open_ratio = value;
    }
}

impl<'a, Color: PixelColor + From<Color::Raw> + Into<Color::Raw> + 'a, String> MouthContext<'a> for DrawContext<Color, String> {
    fn open_ratio(&self) -> f32 {
        self.mouth_open_ratio
    }
    fn set_open_ratio(&mut self, value: f32) {
        self.mouth_open_ratio = value;
    }
    fn breath(&self) -> f32 {
        self.breath
    }
    fn set_breath(&mut self, value: f32) {
        self.breath = value;
    }
}

impl<'a, Color: PixelColor + From<Color::Raw> + Into<Color::Raw> + 'a, String> FaceContext<'a> for DrawContext<Color, String> {}

impl<'a, Color: PixelColor + From<Color::Raw> + Into<Color::Raw> + 'a, String: AsRef<str> + FromStr> BalloonContext<'a> for DrawContext<Color, String> {
    fn text(&self) -> Option<&str> {
        self.text.as_ref().map(|string| string.as_ref())
    }
    fn set_text(&mut self, string: Option<&str>) {
        self.text = string.map(|s| String::from_str(s).ok()).flatten();
    }
}

pub trait FaceContext<'a>: EyeContext<'a> + MouthContext<'a> {}

pub struct Face<'a, Context: FaceContext<'a>> {
    eye_l: Eye<'a, Context>,
    eye_r: Eye<'a, Context>,
    mouth: Mouth<'a, Context>,
    eyeblow_l: Eyeblow<'a, Context>,
    eyeblow_r: Eyeblow<'a, Context>,
    pos_eye_l: Rectangle,
    pos_eye_r: Rectangle,
    pos_mouth: Rectangle,
    pos_eyeblow_l: Rectangle,
    pos_eyeblow_r: Rectangle,
    bounding_rect: Rectangle,
}

impl<'a, Context: FaceContext<'a>> Face<'a, Context> {
    pub fn new(
        eye_l: Eye<'a, Context>,
        eye_r: Eye<'a, Context>,
        mouth: Mouth<'a, Context>,
        eyeblow_l: Eyeblow<'a, Context>,
        eyeblow_r: Eyeblow<'a, Context>,
        pos_eye_l: Rectangle,
        pos_eye_r: Rectangle,
        pos_mouth: Rectangle,
        pos_eyeblow_l: Rectangle,
        pos_eyeblow_r: Rectangle,
        bounding_rect: Rectangle,
    ) -> Self {
        Self {
            eye_l,
            eye_r,
            mouth,
            eyeblow_l,
            eyeblow_r,
            pos_eye_l,
            pos_eye_r,
            pos_mouth,
            pos_eyeblow_l,
            pos_eyeblow_r,
            bounding_rect,
        }
    }
}

impl<'a, Context: FaceContext<'a>> Default for Face<'a, Context> {
    fn default() -> Self {
        Self {
            eye_l: Eye::new(8.0, false),
            eye_r: Eye::new(8.0, true),
            mouth: Mouth::new(50, 90, 4, 60),
            eyeblow_l: Eyeblow::new(32, 2, false),
            eyeblow_r: Eyeblow::new(32, 2, true),
            pos_eye_l: Rectangle::new(Point::new(230, 96), Size::zero()),
            pos_eye_r: Rectangle::new(Point::new(90, 93), Size::zero()),
            pos_mouth: Rectangle::new(Point::new(163, 148), Size::zero()),
            pos_eyeblow_l: Rectangle::new(Point::new(96, 67), Size::zero()),
            pos_eyeblow_r: Rectangle::new(Point::new(230, 72), Size::zero()),
            bounding_rect: Rectangle::new(Point::new(0, 0), Size::new(320, 240)),
        }
    }
}

pub struct DrawableFace<Color: PixelColor> {
    eye_l: DrawableEye<Color>,
    eye_r: DrawableEye<Color>,
    mouth: DrawableMouth<Color>,
    eyeblow_l: DrawableEyeblow<Color>,
    eyeblow_r: DrawableEyeblow<Color>,
}


impl<Color: PixelColor + Into<Color::Raw> + From<Color::Raw>> DrawableGraphics for DrawableFace<Color> {
    type Color = Color;
    type Output = ();
    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
        where
            D: embedded_graphics::prelude::DrawTarget<Color = Self::Color> {
        self.eye_l.draw(target)?;
        self.eye_r.draw(target)?;
        self.mouth.draw(target)?;
        self.eyeblow_l.draw(target)?;
        self.eyeblow_r.draw(target)?;
        Ok(())
    }
}


impl <'a, Context: FaceContext<'a>> Component<'a> for Face<'a, Context> {
    type Context = Context;
    type Drawable = DrawableFace<<Context as BasicPaletteContext<'a>>::Color>;
    fn render(&self, bounding_rect: Rectangle, context: &'a Self::Context) -> Self::Drawable {
        let mouth = {
            self.mouth.render(self.pos_mouth, context)
        };
        let eye_l = {
            self.eye_l.render(self.pos_eye_l, context)
        };
        let eye_r = {
            self.eye_r.render(self.pos_eye_r, context)
        };
        let eyeblow_l = {
            self.eyeblow_l.render(self.pos_eyeblow_l, context)
        };
        let eyeblow_r = {
            self.eyeblow_r.render(self.pos_eyeblow_r, context)
        };
        
        // TODO: support scaling
        Self::Drawable {
            eye_l,
            eye_r,
            mouth,
            eyeblow_l,
            eyeblow_r,
        }
    }
}