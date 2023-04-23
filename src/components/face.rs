use embedded_graphics::prelude::{PixelColor, Size, Point, Drawable as DrawableGraphics};
use embedded_graphics::primitives::Rectangle;
use rand_core::SeedableRng;

use crate::{Expression, ArrayPalette, BasicPaletteKey, BasicPaletteContext, ExpressionContext, Component, Palette};
use crate::components::eye::{Eye, EyeContext, GazeContext};
use crate::components::mouth::{Mouth, MouthContext};

use super::eye::DrawableEye;
use super::eyeblow::{Eyeblow, DrawableEyeblow};
use super::mouth::DrawableMouth;

pub trait RandomGeneratorContext {
    type Rng: rand_core::RngCore;
    fn rng(&mut self) -> &mut Self::Rng;
}

pub struct DrawContext<Color: PixelColor> {
    pub expression: Expression,
    pub breath: f32,
    pub gaze_horizontal: f32,
    pub gaze_vertical: f32,
    pub eye_open_ratio: f32,
    pub mouth_open_ratio: f32,
    pub palette: ArrayPalette<BasicPaletteKey, Color, {BasicPaletteKey::VARIANT_COUNT}>,
    pub rng: rand_xorshift::XorShiftRng,
}

impl<Color: PixelColor + Default> Default for DrawContext<Color> {
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
        }
    }
}

impl<Color: PixelColor> RandomGeneratorContext for DrawContext<Color> {
    type Rng = rand_xorshift::XorShiftRng;
    fn rng(&mut self) -> &mut rand_xorshift::XorShiftRng {
        &mut self.rng
    }
}

impl<Color: PixelColor> BasicPaletteContext for DrawContext<Color> {
    type BasicPalette = ArrayPalette<BasicPaletteKey, Color, {BasicPaletteKey::VARIANT_COUNT}>;
    type Color = Color;
    fn get_basic_palette(&self) -> &Self::BasicPalette {
        &self.palette
    }
}

impl<Color: PixelColor> ExpressionContext for DrawContext<Color> {
    fn expression(&self) -> Expression {
        self.expression
    }
}

impl<Color: PixelColor> GazeContext for DrawContext<Color> {
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

impl<Color: PixelColor> EyeContext for DrawContext<Color> {
    fn open_ratio(&self) -> f32 {
        self.eye_open_ratio
    }
    fn set_open_ratio(&mut self, value: f32) {
        self.eye_open_ratio = value;
    }
}

impl<Color: PixelColor> MouthContext for DrawContext<Color> {
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

impl<Color: PixelColor> FaceContext for DrawContext<Color> {}

pub trait FaceContext: EyeContext + MouthContext {}

pub struct Face<Context: FaceContext> {
    eye_l: Eye<Context>,
    eye_r: Eye<Context>,
    mouth: Mouth<Context>,
    eyeblow_l: Eyeblow<Context>,
    eyeblow_r: Eyeblow<Context>,
    pos_eye_l: Rectangle,
    pos_eye_r: Rectangle,
    pos_mouth: Rectangle,
    pos_eyeblow_l: Rectangle,
    pos_eyeblow_r: Rectangle,
    bounding_rect: Rectangle,
}

impl<Context: FaceContext> Face<Context> {
    pub fn new(
        eye_l: Eye<Context>,
        eye_r: Eye<Context>,
        mouth: Mouth<Context>,
        eyeblow_l: Eyeblow<Context>,
        eyeblow_r: Eyeblow<Context>,
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

impl<Context: FaceContext> Default for Face<Context> {
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


impl<Color: PixelColor> DrawableGraphics for DrawableFace<Color> {
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


impl <Context: FaceContext> Component for Face<Context> {
    type Context = Context;
    type Drawable = DrawableFace<Context::Color>;
    fn render(&self, bounding_rect: Rectangle, context: &Self::Context) -> Self::Drawable {
        let breath = context.breath();
        let breath_offset = Point::new(0, (breath * 3.0) as i32);
        let mouth = {
            let mut rect = self.pos_mouth;
            rect.top_left += breath_offset;
            self.mouth.render(rect, context)
        };
        let eye_l = {
            let mut rect = self.pos_eye_l;
            rect.top_left += breath_offset;
            self.eye_l.render(rect, context)
        };
        let eye_r = {
            let mut rect = self.pos_eye_r;
            rect.top_left += breath_offset;
            self.eye_r.render(rect, context)
        };
        let eyeblow_l = {
            let mut rect = self.pos_eyeblow_l;
            rect.top_left += breath_offset;
            self.eyeblow_l.render(rect, context)
        };
        let eyeblow_r = {
            let mut rect = self.pos_eyeblow_r;
            rect.top_left += breath_offset;
            self.eyeblow_r.render(rect, context)
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