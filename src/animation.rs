use core::{marker::PhantomData, fmt::Debug};
use rand_core::{SeedableRng, RngCore};

use crate::components::{face::{DrawContext, FaceContext, RandomGeneratorContext}, mouth::MouthContext, eye::{GazeContext, EyeContext}};

#[derive(Clone, Copy, Debug, Default)]
pub struct FrameCounter {
    counter: u64,
    frames_per_second: u64,
}

impl FrameCounter {
    pub fn after_milliseconds(&self, milliseconds: u64) -> Self {
        Self {
            counter: self.counter.wrapping_add(self.frames_per_second * milliseconds / 1000),
            frames_per_second: self.frames_per_second,
        }
    }
    pub fn after_frames(&self, frames: u64) -> Self {
        Self {
            counter: self.counter.wrapping_add(frames),
            frames_per_second: self.frames_per_second,
        }
    }
    pub fn is_after(&self, other: &FrameCounter) -> bool {
        self.counter >= other.counter
    }
}

pub trait Animator<Context> {
    fn next(&mut self, counter: FrameCounter, context: &mut Context) -> FrameCounter;
}

pub struct AnimationRunner<Context, RootAnimator: Animator<Context>> {
    context: Context,
    counter: FrameCounter,
    scheduled: FrameCounter,
    animator: RootAnimator,
}

impl<Context, RootAnimator: Animator<Context>> AnimationRunner<Context, RootAnimator> {
    pub fn new(context: Context, frames_per_second: u64, animator: RootAnimator) -> Self {
        Self {
            context,
            counter: FrameCounter { counter: 0, frames_per_second, },
            scheduled: FrameCounter { counter: 0, frames_per_second, },
            animator,
        }
    }
    pub fn next(&mut self) {
        if self.counter.is_after(&self.scheduled) {
            self.scheduled = self.animator.next(self.counter, &mut self.context);
        }
        self.counter = self.counter.after_frames(1);
    }
    pub fn context(&mut self) -> &mut Context {
        &mut self.context
    }
}

#[derive(Debug)]
pub struct BreathAnimator {
    c: u32,
}
impl Default for BreathAnimator {
    fn default() -> Self {
        Self {
            c: 0,
        }
    }
}
impl<Context: MouthContext> Animator<Context> for BreathAnimator {
    fn next(&mut self, counter: FrameCounter, context: &mut Context) -> FrameCounter {
        self.c = (self.c + 1) % 100;
        let f = f32::sin((self.c as f32) * 2.0 * core::f32::consts::PI / 100.0);
        context.set_breath(f);
        counter.after_milliseconds(33)
    }
} 

fn rand_f32_range<Rng: RngCore>(rng: &mut Rng, from: f32, to: f32) -> f32 {
    (to - from) * (rng.next_u32() as f32) / (u32::MAX as f32) + from
}
fn rand_u32_nonuniform<Rng: RngCore>(rng: &mut Rng, from: u32, to: u32) -> u32 {
    rng.next_u32() % (to - from + 1) + from
}

#[derive(Debug)]
pub struct SaccadeAnimator {}

impl Default for SaccadeAnimator {
    fn default() -> Self {
        Self {}
    }
}

impl<Context: GazeContext + RandomGeneratorContext> Animator<Context> for SaccadeAnimator {
    fn next(&mut self, counter: FrameCounter, context: &mut Context) -> FrameCounter {
        let vertical = rand_f32_range(context.rng(), -1.0, 1.0);
        let horizontal = rand_f32_range(context.rng(), -1.0, 1.0);
        context.set_horizontal(horizontal);
        context.set_vertical(vertical);
        counter.after_milliseconds(500 + 100 * rand_u32_nonuniform(context.rng(), 0, 20) as u64)
    }
}

#[derive(Debug)]
pub struct BlinkAnimator {
    is_open: bool,
}

impl Default for BlinkAnimator {
    fn default() -> Self {
        Self {
            is_open: false,
        }
    }
}

impl<Context: EyeContext + RandomGeneratorContext> Animator<Context> for BlinkAnimator {
    fn next(&mut self, counter: FrameCounter, context: &mut Context) -> FrameCounter {
        self.is_open = !self.is_open;
        if self.is_open {
            // open
            context.set_open_ratio(1.0);
            counter.after_milliseconds(2500 + 100 * rand_u32_nonuniform(context.rng(), 0, 20) as u64)
        } else {
            // close
            context.set_open_ratio(0.0);
            counter.after_milliseconds(300 + 10 * rand_u32_nonuniform(context.rng(), 0, 20) as u64)
        }
    }
}

#[derive(Debug)]
pub struct FaceAnimator {
    breath: BreathAnimator,
    saccade: SaccadeAnimator,
    blink: BlinkAnimator,
    breath_counter: FrameCounter,
    saccade_counter: FrameCounter,
    blink_counter: FrameCounter,
}

impl FaceAnimator {
    pub fn new() -> Self {
        Self {
            breath: BreathAnimator::default(),
            saccade: SaccadeAnimator::default(),
            blink: BlinkAnimator::default(),
            breath_counter: FrameCounter::default(),
            saccade_counter: FrameCounter::default(),
            blink_counter: FrameCounter::default(),
        }
    }
}

impl<Context: FaceContext + RandomGeneratorContext> Animator<Context> for FaceAnimator {
    fn next(&mut self, counter: FrameCounter, context: &mut Context) -> FrameCounter {
        if counter.is_after(&self.breath_counter) {
            self.breath_counter = self.breath.next(counter, context);
        }
        if counter.is_after(&self.saccade_counter) {
            self.saccade_counter = self.saccade.next(counter, context);
        }
        if counter.is_after(&self.blink_counter) {
            self.blink_counter = self.blink.next(counter, context);
        }
        [self.breath_counter, self.saccade_counter, self.blink_counter].into_iter().min_by(|x, y| x.counter.cmp(&y.counter)).unwrap() 
    }
}