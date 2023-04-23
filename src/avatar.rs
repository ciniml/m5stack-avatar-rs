use core::str::FromStr;

use embedded_graphics::{prelude::{PixelColor, DrawTarget}, primitives::Rectangle, Drawable};

use crate::{components::{face::{Face, DrawContext}, effect::Effect, balloon::Balloon}, animation::{AnimationRunner, FaceAnimator}, Component};

pub trait Timer {
    fn timestamp_milliseconds(&self) -> u64; 
}

pub struct Avatar<'a, Color: PixelColor, String: AsRef<str> + FromStr> {
    last_time: Option<u64>,
    frames_per_second: u64,
    face: Face<'a, DrawContext<Color, String>>,
    effect: Effect<'a, DrawContext<Color, String>>,
    balloon: Balloon<'a, DrawContext<Color, String>>,
    runner: AnimationRunner<DrawContext<Color, String>, FaceAnimator>,
}

impl<'a, Color: PixelColor, String: AsRef<str> + FromStr> Avatar<'a, Color, String> {
    pub fn new(context: DrawContext<Color, String>, frames_per_second: u64) -> Self {
        Self {
            last_time: None,
            frames_per_second,
            face: Face::default(),
            effect: Effect::new(),
            balloon: Balloon::new(),
            runner: AnimationRunner::new(context, frames_per_second, FaceAnimator::new()),
        }
    }
    pub fn context(&mut self) -> &mut DrawContext<Color, String> {
        self.runner.context()
    }
    pub fn run<D: DrawTarget<Color = Color>, T: Timer>(&mut self, draw_target: &mut D, timer: &T) -> Result<(), <D as DrawTarget>::Error> {
        let now = timer.timestamp_milliseconds();
        let last_time = self.last_time.unwrap_or(now);
        let next_time = last_time.wrapping_add(1000 / self.frames_per_second);
        if next_time >= last_time {
            self.last_time = Some(now);
            self.runner.next();
            self.face.render(Rectangle::zero(), self.runner.context())
                .draw(draw_target)?;
            self.effect.render(Rectangle::zero(), self.runner.context())
                .draw(draw_target)?;
            self.balloon.render(Rectangle::zero(), self.runner.context())
                .draw(draw_target)?;
        }
        Ok(())
    }
}

impl<'a, Color: PixelColor + Default, String: AsRef<str> + FromStr> Default for Avatar<'a, Color, String> {
    fn default() -> Self {
        Self::new(Default::default(), 30)
    }
}