use embedded_graphics::{prelude::{PixelColor, DrawTarget}, primitives::Rectangle, Drawable};

use crate::{components::face::{Face, DrawContext}, animation::{AnimationRunner, FaceAnimator}, Component};

pub trait Timer {
    fn timestamp_milliseconds(&self) -> u64; 
}

pub struct Avatar<Color: PixelColor> {
    last_time: Option<u64>,
    frames_per_second: u64,
    face: Face<DrawContext<Color>>,
    runner: AnimationRunner<DrawContext<Color>, FaceAnimator>,
}

impl<Color: PixelColor> Avatar<Color> {
    pub fn new(context: DrawContext<Color>, frames_per_second: u64) -> Self {
        Self {
            last_time: None,
            frames_per_second,
            face: Face::default(),
            runner: AnimationRunner::new(context, frames_per_second, FaceAnimator::new()),
        }
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
        }
        Ok(())
    }
}

impl<Color: PixelColor + Default> Default for Avatar<Color> {
    fn default() -> Self {
        Self::new(Default::default(), 30)
    }
}