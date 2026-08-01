//! stackchan-idf–compatible avatar renderer.
//!
//! A faithful port of the `stackchan-idf` C++ avatar: the face is drawn by the
//! [`vm`] bytecode interpreter (a port of `components/avatar_vm`) running the same
//! compiled `default_face.avbc` the C++ firmware embeds, driven by the same animators
//! (breath sine, saccade summed onto an external gaze target, instant blink) and
//! completed by the same bottom balloon with marquee scrolling.
//!
//! Faces are hot-swappable: [`StackchanAvatar::load_face_bytecode`] accepts any `AVDS`
//! v1 file produced by stackchan-idf's `tools/avatar_dsl` compiler, and
//! [`StackchanAvatar::reset_face_bytecode`] restores the embedded default.
//!
//! Rendering follows the C++ "direct" canvas strategy: the background is painted only
//! on a full repaint, and each face element is composited into a small scratch sprite
//! covering its maximum extent ("group") and blitted in one go, so size-varying shapes
//! neither flicker nor leave residue.
//!
//! Differences from the C++ implementation:
//! - Balloon text uses an embedded-graphics monospace ASCII font instead of
//!   `lgfxJapanGothic`; Japanese text is not yet supported.

pub mod vm;

use alloc::string::String;
use alloc::vec::Vec;

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::mono_font::{MonoTextStyle, ascii};
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{
    Circle, CornerRadii, PrimitiveStyle, Rectangle, RoundedRectangle, Triangle,
};
use embedded_graphics::text::{Alignment, Baseline, Text, TextStyleBuilder};

use crate::Expression;
use crate::sprite::Sprite;

pub use vm::{Bytecode, Vm, VmError};

/// The default face, compiled from stackchan-idf's `assets/default_face.avdsl`
/// (bit-identical to the bytecode the C++ firmware embeds).
pub const DEFAULT_FACE_AVBC: &[u8] = include_bytes!("default_face.avbc");

/// Colour bounds shared by everything in this module: an RGB565-compatible colour
/// (the VM exchanges colours as raw RGB565 values, exactly like the C++ version).
pub trait VmColor:
    RgbColor + PixelColor<Raw = RawU16> + From<RawU16> + Into<RawU16> + Default
{
}
impl<T> VmColor for T where
    T: RgbColor + PixelColor<Raw = RawU16> + From<RawU16> + Into<RawU16> + Default
{
}

/// Colour set, mirroring `stackchan::avatar::Palette`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette<Color> {
    pub primary: Color,
    pub background: Color,
    pub secondary: Color,
    pub balloon_foreground: Color,
    pub balloon_background: Color,
}

impl<Color: RgbColor> Default for Palette<Color> {
    fn default() -> Self {
        Self {
            primary: Color::WHITE,
            background: Color::BLACK,
            secondary: Color::YELLOW,
            // Balloon uses inverted colours so it stands out against the dark face.
            balloon_foreground: Color::BLACK,
            balloon_background: Color::WHITE,
        }
    }
}

/// Face layout tuning, mirroring `stackchan::avatar::FaceTuning` (colours live in
/// [`Palette`] instead). All lengths are in 320x240 design pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FaceTuning {
    pub eyebrows_visible: bool,
    pub eye_radius: f32,
    pub eye_off_x: f32,
    pub eye_off_y: f32,
    pub brow_off_x: f32,
    pub brow_off_y: f32,
    pub mouth_off_x: f32,
    pub mouth_off_y: f32,
    /// Resting / open mouth width range.
    pub mouth_min_w: i32,
    pub mouth_max_w: i32,
    /// Closed / open mouth height range.
    pub mouth_min_h: i32,
    pub mouth_max_h: i32,
    pub cheeks_visible: bool,
    pub cheek_radius: f32,
    pub cheek_off_x: f32,
    pub cheek_off_y: f32,
}

impl Default for FaceTuning {
    fn default() -> Self {
        Self {
            eyebrows_visible: true,
            eye_radius: 8.0,
            eye_off_x: 0.0,
            eye_off_y: 0.0,
            brow_off_x: 0.0,
            brow_off_y: 0.0,
            mouth_off_x: 0.0,
            mouth_off_y: 0.0,
            mouth_min_w: 50,
            mouth_max_w: 90,
            mouth_min_h: 4,
            mouth_max_h: 60,
            cheeks_visible: false,
            cheek_radius: 10.0,
            cheek_off_x: 0.0,
            cheek_off_y: 0.0,
        }
    }
}

/// Idle-animator tuning, mirroring `stackchan::avatar::internal::AnimatorParams`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimatorParams {
    pub breath_enabled: bool,
    pub saccade_enabled: bool,
    /// Gaze dwell-time range.
    pub saccade_min_ms: u32,
    pub saccade_max_ms: u32,
    /// Peak gaze offset (0..1).
    pub gaze_amplitude: f32,
    pub blink_enabled: bool,
    pub blink_open_min_ms: u32,
    pub blink_open_max_ms: u32,
    pub blink_closed_min_ms: u32,
    pub blink_closed_max_ms: u32,
}

impl Default for AnimatorParams {
    fn default() -> Self {
        Self {
            breath_enabled: true,
            saccade_enabled: true,
            saccade_min_ms: 500,
            saccade_max_ms: 2500,
            gaze_amplitude: 1.0,
            blink_enabled: true,
            blink_open_min_ms: 2500,
            blink_open_max_ms: 4500,
            blink_closed_min_ms: 300,
            blink_closed_max_ms: 500,
        }
    }
}

// Same xorshift32 as the C++ implementation.
struct XorShift32 {
    state: u32,
}

impl XorShift32 {
    fn new(seed: u32) -> Self {
        Self {
            state: if seed != 0 { seed } else { 0x1234_5678 },
        }
    }
    fn next(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }
    fn next_range(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * (self.next() as f32 / u32::MAX as f32)
    }
    fn next_inclusive(&mut self, low: u32, high: u32) -> u32 {
        low + self.next() % (high - low + 1)
    }
}

/// Animation / drawing state, mirroring `stackchan::avatar::DrawContext`.
pub struct DrawContext<Color> {
    pub expression: Expression,
    pub breath: f32,
    /// External / commanded gaze target (e.g. touch-follow). Saccade is added on top so
    /// the eyes still wander a bit around the target rather than locking dead-stop.
    pub gaze_horizontal: f32,
    pub gaze_vertical: f32,
    gaze_saccade_h: f32,
    gaze_saccade_v: f32,
    pub eye_open_ratio: f32,
    pub mouth_open_ratio: f32,
    pub palette: Palette<Color>,
    rng_state: u32,
    balloon_text: Option<String>,
    now_ms: u32,
    balloon_set_ms: u32,
    balloon_hold_ms: u32,
    balloon_done: bool,
}

impl<Color: RgbColor> Default for DrawContext<Color> {
    fn default() -> Self {
        Self {
            expression: Expression::Neutral,
            breath: 0.0,
            gaze_horizontal: 0.0,
            gaze_vertical: 0.0,
            gaze_saccade_h: 0.0,
            gaze_saccade_v: 0.0,
            eye_open_ratio: 1.0,
            mouth_open_ratio: 0.0,
            palette: Palette::default(),
            rng_state: 0xC0FFEE,
            balloon_text: None,
            now_ms: 0,
            balloon_set_ms: 0,
            balloon_hold_ms: 0,
            balloon_done: false,
        }
    }
}

/// Combines breath / saccade / blink. Each animator schedules its next firing in
/// absolute milliseconds; `tick()` applies whichever are due.
struct FaceAnimator {
    params: AnimatorParams,
    breath_next_ms: u32,
    saccade_next_ms: u32,
    blink_next_ms: u32,
    breath_phase: u32,
    eyes_open: bool,
}

impl FaceAnimator {
    fn new() -> Self {
        Self {
            params: AnimatorParams::default(),
            breath_next_ms: 0,
            saccade_next_ms: 0,
            blink_next_ms: 0,
            breath_phase: 0,
            eyes_open: true,
        }
    }

    fn tick<Color>(&mut self, now_ms: u32, ctx: &mut DrawContext<Color>) {
        use micromath::F32Ext;
        if self.params.breath_enabled {
            if now_ms >= self.breath_next_ms {
                self.breath_phase = (self.breath_phase + 1) % 100;
                ctx.breath = (self.breath_phase as f32 * 2.0 * core::f32::consts::PI / 100.0).sin();
                self.breath_next_ms = now_ms + 33;
            }
        } else {
            ctx.breath = 0.0;
        }
        if self.params.saccade_enabled {
            if now_ms >= self.saccade_next_ms {
                let mut rng = XorShift32::new(ctx.rng_state);
                let amp = self.params.gaze_amplitude;
                ctx.gaze_saccade_h = rng.next_range(-amp, amp);
                ctx.gaze_saccade_v = rng.next_range(-amp, amp);
                let lo = self.params.saccade_min_ms;
                let hi = self.params.saccade_max_ms.max(lo);
                let delay = rng.next_inclusive(lo, hi);
                ctx.rng_state = rng.next();
                self.saccade_next_ms = now_ms + delay;
            }
        } else {
            ctx.gaze_saccade_h = 0.0;
            ctx.gaze_saccade_v = 0.0;
        }
        if self.params.blink_enabled {
            if now_ms >= self.blink_next_ms {
                let mut rng = XorShift32::new(ctx.rng_state);
                self.eyes_open = !self.eyes_open;
                let delay = if self.eyes_open {
                    ctx.eye_open_ratio = 1.0;
                    let lo = self.params.blink_open_min_ms;
                    rng.next_inclusive(lo, self.params.blink_open_max_ms.max(lo))
                } else {
                    ctx.eye_open_ratio = 0.0;
                    let lo = self.params.blink_closed_min_ms;
                    rng.next_inclusive(lo, self.params.blink_closed_max_ms.max(lo))
                };
                ctx.rng_state = rng.next();
                self.blink_next_ms = now_ms + delay;
            }
        } else {
            ctx.eye_open_ratio = 1.0;
            self.eyes_open = true;
        }
    }
}

// Balloon geometry / marquee tuning (from balloon.cpp).
const BALLOON_MARGIN: i32 = 4;
const BALLOON_PANEL_RADIUS: u32 = 10;
const BALLOON_INNER_PADDING: i32 = 8;
const BALLOON_SMALL_PANEL_THRESHOLD: i32 = 160;
const BALLOON_BIG_PANEL_H: i32 = 40;
const BALLOON_SMALL_PANEL_H: i32 = 22;
const BALLOON_SCROLL_PX_PER_SEC: i32 = 60;
const BALLOON_REPEAT_GAP_PX: i32 = 60;
const BALLOON_DEFAULT_STATIC_HOLD_MS: u32 = 3000;

/// stackchan-idf–style avatar. Owns no display: [`Self::tick`] composes one frame into
/// the borrowed `DrawTarget`.
pub struct StackchanAvatar<Color> {
    context: DrawContext<Color>,
    tuning: FaceTuning,
    animator: FaceAnimator,
    vm: Vm,
    bytecode: Bytecode,
    last_vm_error: Option<VmError>,
    scratch: Vec<u8>,
    full_repaint_pending: bool,
}

impl<Color: VmColor> StackchanAvatar<Color> {
    pub fn new() -> Self {
        Self {
            context: DrawContext::default(),
            tuning: FaceTuning::default(),
            animator: FaceAnimator::new(),
            vm: Vm::new(),
            // The embedded default is validated by the DSL compiler at build time.
            bytecode: vm::decode(DEFAULT_FACE_AVBC).unwrap(),
            last_vm_error: None,
            scratch: Vec::new(),
            full_repaint_pending: true,
        }
    }

    pub fn context(&mut self) -> &mut DrawContext<Color> {
        &mut self.context
    }

    pub fn animator_params(&mut self) -> &mut AnimatorParams {
        &mut self.animator.params
    }

    pub fn set_expression(&mut self, expression: Expression) {
        if self.context.expression != expression {
            self.context.expression = expression;
            self.full_repaint_pending = true;
        }
    }

    pub fn set_mouth_open(&mut self, ratio: f32) {
        self.context.mouth_open_ratio = ratio.clamp(0.0, 1.0);
    }

    pub fn set_gaze(&mut self, horizontal: f32, vertical: f32) {
        self.context.gaze_horizontal = horizontal;
        self.context.gaze_vertical = vertical;
    }

    pub fn set_palette(&mut self, palette: Palette<Color>) {
        self.context.palette = palette;
        self.full_repaint_pending = true;
    }

    pub fn set_face_tuning(&mut self, tuning: FaceTuning) {
        self.tuning = tuning;
        self.full_repaint_pending = true;
    }

    pub fn request_full_repaint(&mut self) {
        self.full_repaint_pending = true;
    }

    /// Hot-swap the face bytecode (`AVDS` v1, produced by stackchan-idf's
    /// `tools/avatar_dsl` compiler). On failure the previous face is kept.
    pub fn load_face_bytecode(&mut self, bytes: &[u8]) -> Result<(), VmError> {
        let bc = vm::decode(bytes)?;
        self.bytecode = bc;
        self.last_vm_error = None;
        self.full_repaint_pending = true;
        Ok(())
    }

    /// Revert to the embedded default face.
    pub fn reset_face_bytecode(&mut self) {
        self.bytecode = vm::decode(DEFAULT_FACE_AVBC).unwrap();
        self.last_vm_error = None;
        self.full_repaint_pending = true;
    }

    /// Runtime error from the most recent VM run, if any (face rendering is skipped
    /// while the error persists; balloon and background continue drawing).
    pub fn last_vm_error(&self) -> Option<VmError> {
        self.last_vm_error
    }

    /// Show `text` in the balloon. `hold_ms` overrides the default display time
    /// (0 = use balloon defaults: short text holds for a few seconds, long text plays
    /// one full marquee pass).
    pub fn set_balloon_text(&mut self, text: &str, hold_ms: u32) {
        self.context.balloon_text = Some(String::from(text));
        self.context.balloon_hold_ms = hold_ms;
        self.context.balloon_done = false;
        self.context.balloon_set_ms = self.context.now_ms;
    }

    pub fn clear_balloon(&mut self) {
        self.context.balloon_text = None;
        self.context.balloon_hold_ms = 0;
        self.context.balloon_done = false;
        self.full_repaint_pending = true;
    }

    /// True once the current balloon has been fully displayed (hold elapsed or one
    /// marquee pass completed). Stays true until the next set/clear.
    pub fn is_balloon_done(&self) -> bool {
        self.context.balloon_done
    }

    /// Drives the animators with the current time (ms) and renders one frame into
    /// `display`. The face is authored for 320x240 and scaled to the display size.
    pub fn tick<D>(&mut self, now_ms: u32, display: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Color>,
    {
        self.animator.tick(now_ms, &mut self.context);
        self.context.now_ms = now_ms;

        let bounds = display.bounding_box();
        let canvas_w = bounds.size.width as i32;
        let canvas_h = bounds.size.height as i32;

        if self.full_repaint_pending {
            display.clear(self.context.palette.background)?;
            self.full_repaint_pending = false;
        }

        // Face via the bytecode VM.
        let mut adapter = VmCanvasAdapter {
            display,
            scratch: &mut self.scratch,
            ctx: &self.context,
            tuning: &self.tuning,
            canvas_w,
            canvas_h,
            group: None,
            error: None,
        };
        match self.vm.run(&self.bytecode, &mut adapter) {
            Ok(()) => self.last_vm_error = None,
            Err(e) => self.last_vm_error = Some(e),
        }
        if let Some(e) = adapter.error {
            return Err(e);
        }

        // Balloon (native, as in the C++ implementation).
        let balloon_done = draw_balloon(
            display,
            &mut self.scratch,
            &self.context,
            canvas_w,
            canvas_h,
        )?;
        if balloon_done {
            self.context.balloon_done = true;
        }
        Ok(())
    }
}

impl<Color: VmColor> Default for StackchanAvatar<Color> {
    fn default() -> Self {
        Self::new()
    }
}

/// Expression → VM numeric value (opcodes.hpp `ExprValue`).
fn expression_to_f(e: Expression) -> f32 {
    match e {
        Expression::Neutral => 0.0,
        Expression::Happy => 1.0,
        Expression::Sad => 2.0,
        Expression::Angry => 3.0,
        Expression::Doubt => 4.0,
        Expression::Sleepy => 5.0,
    }
}

/// Grow-only scratch helper: composite `drawable` either into the active group sprite
/// or directly onto the display.
fn ensure_scratch<Color: VmColor>(scratch: &mut Vec<u8>, rect: &Rectangle) {
    let needed = Sprite::<Color>::unaligned_buffer_size(rect.size.width, rect.size.height);
    if scratch.len() < needed {
        scratch.resize(needed, 0);
    }
}

/// Clamp a group rect to the canvas. Returns `None` when fully off-screen.
fn clamp_rect(x: i32, y: i32, w: i32, h: i32, canvas_w: i32, canvas_h: i32) -> Option<Rectangle> {
    let x0 = x.max(0);
    let y0 = y.max(0);
    let x1 = (x + w).min(canvas_w);
    let y1 = (y + h).min(canvas_h);
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    Some(Rectangle::new(
        Point::new(x0, y0),
        Size::new((x1 - x0) as u32, (y1 - y0) as u32),
    ))
}

/// [`vm::VmCanvas`] backend over an embedded-graphics `DrawTarget`, implementing the
/// C++ DirectCanvas strategy: primitives inside a group are composited into a scratch
/// sprite (persisting across primitives) and blitted once at `end_group`.
struct VmCanvasAdapter<'a, D: DrawTarget> {
    display: &'a mut D,
    scratch: &'a mut Vec<u8>,
    ctx: &'a DrawContext<D::Color>,
    tuning: &'a FaceTuning,
    canvas_w: i32,
    canvas_h: i32,
    group: Option<Rectangle>,
    error: Option<D::Error>,
}

impl<'a, D> VmCanvasAdapter<'a, D>
where
    D: DrawTarget,
    D::Color: VmColor,
{
    fn color(&self, raw: u16) -> D::Color {
        D::Color::from(RawU16::new(raw))
    }

    fn draw_prim(&mut self, drawable: &impl Drawable<Color = D::Color>) {
        if let Some(rect) = self.group {
            if let Ok(mut sprite) = Sprite::<D::Color>::new_unaligned(self.scratch, rect) {
                let _ = drawable.draw(&mut sprite);
            }
        } else if self.error.is_none()
            && let Err(e) = drawable.draw(self.display).map(|_| ())
        {
            self.error = Some(e);
        }
    }
}

impl<'a, D> vm::VmCanvas for VmCanvasAdapter<'a, D>
where
    D: DrawTarget,
    D::Color: VmColor,
{
    fn width(&self) -> i32 {
        self.canvas_w
    }
    fn height(&self) -> i32 {
        self.canvas_h
    }

    fn fill_rect(&mut self, mut x: i32, mut y: i32, mut w: i32, mut h: i32, color: u16) {
        // Normalize negative extents like LovyanGFX does.
        if w < 0 {
            x += w;
            w = -w;
        }
        if h < 0 {
            y += h;
            h = -h;
        }
        if w == 0 || h == 0 {
            return;
        }
        let color = self.color(color);
        let prim = Rectangle::new(Point::new(x, y), Size::new(w as u32, h as u32))
            .into_styled(PrimitiveStyle::with_fill(color));
        self.draw_prim(&prim);
    }

    fn fill_circle(&mut self, cx: i32, cy: i32, r: i32, color: u16) {
        if r < 0 {
            return;
        }
        let color = self.color(color);
        let prim = Circle::new(Point::new(cx - r, cy - r), (r * 2 + 1) as u32)
            .into_styled(PrimitiveStyle::with_fill(color));
        self.draw_prim(&prim);
    }

    fn fill_triangle(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32, color: u16) {
        let color = self.color(color);
        let prim = Triangle::new(Point::new(x0, y0), Point::new(x1, y1), Point::new(x2, y2))
            .into_styled(PrimitiveStyle::with_fill(color));
        self.draw_prim(&prim);
    }

    fn begin_group(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let Some(rect) = clamp_rect(x, y, w, h, self.canvas_w, self.canvas_h) else {
            self.group = None;
            return;
        };
        ensure_scratch::<D::Color>(self.scratch, &rect);
        if let Ok(mut sprite) = Sprite::<D::Color>::new_unaligned(self.scratch, rect) {
            let _ = sprite.clear(self.ctx.palette.background);
            self.group = Some(rect);
        } else {
            self.group = None;
        }
    }

    fn end_group(&mut self) {
        let Some(rect) = self.group.take() else {
            return;
        };
        if let Ok(sprite) = Sprite::<D::Color>::new_unaligned(self.scratch, rect)
            && self.error.is_none()
            && let Err(e) = sprite.draw(self.display)
        {
            self.error = Some(e);
        }
    }

    fn read_var(&self, id: u8, scale: f32) -> f32 {
        let ctx = self.ctx;
        let t = self.tuning;
        let c565 = |c: D::Color| -> f32 { Into::<RawU16>::into(c).into_inner() as f32 };
        match id {
            0x00 => self.canvas_w as f32,                            // CanvasW
            0x01 => self.canvas_h as f32,                            // CanvasH
            0x02 => scale,                                           // CanvasScale
            0x03 => ctx.now_ms as f32,                               // NowMs
            0x04 => ctx.breath,                                      // Breath
            0x05 => ctx.eye_open_ratio,                              // EyeOpen
            0x06 => ctx.gaze_horizontal + ctx.gaze_saccade_h,        // GazeH
            0x07 => ctx.gaze_vertical + ctx.gaze_saccade_v,          // GazeV
            0x08 => ctx.mouth_open_ratio,                            // MouthOpen
            0x09 => expression_to_f(ctx.expression),                 // Expr
            0x0A => c565(ctx.palette.primary),                       // Primary
            0x0B => c565(ctx.palette.background),                    // Background
            0x0C => c565(ctx.palette.secondary),                     // Secondary
            0x0D => c565(ctx.palette.balloon_foreground),            // BalloonFg
            0x0E => c565(ctx.palette.balloon_background),            // BalloonBg
            0x0F => t.eye_radius,                                    // EyeRadius
            0x10 => t.eye_off_x,
            0x11 => t.eye_off_y,
            0x12 => t.brow_off_x,
            0x13 => t.brow_off_y,
            0x14 => t.mouth_off_x,
            0x15 => t.mouth_off_y,
            0x16 => t.mouth_min_w as f32,
            0x17 => t.mouth_max_w as f32,
            0x18 => t.mouth_min_h as f32,
            0x19 => t.mouth_max_h as f32,
            0x1A => t.eyebrows_visible as u8 as f32,
            0x1B => t.cheeks_visible as u8 as f32,
            0x1C => t.cheek_radius,
            0x1D => t.cheek_off_x,
            0x1E => t.cheek_off_y,
            _ => 0.0,
        }
    }
}

/// Bottom balloon strip (from balloon.cpp). Returns `true` once the message has been
/// fully displayed.
fn draw_balloon<D>(
    display: &mut D,
    scratch: &mut Vec<u8>,
    ctx: &DrawContext<D::Color>,
    canvas_w: i32,
    canvas_h: i32,
) -> Result<bool, D::Error>
where
    D: DrawTarget,
    D::Color: VmColor,
{
    let Some(text) = ctx.balloon_text.as_deref() else {
        return Ok(false);
    };
    if text.is_empty() {
        return Ok(false);
    }

    let small_panel = canvas_h <= BALLOON_SMALL_PANEL_THRESHOLD;
    let font = if small_panel {
        &ascii::FONT_6X10
    } else {
        &ascii::FONT_10X20
    };
    let panel_h = if small_panel {
        BALLOON_SMALL_PANEL_H
    } else {
        BALLOON_BIG_PANEL_H
    };
    let panel_x = BALLOON_MARGIN;
    let panel_w = canvas_w - BALLOON_MARGIN * 2;
    let panel_y = canvas_h - panel_h - BALLOON_MARGIN;

    let fg = ctx.palette.balloon_foreground;
    let bg = ctx.palette.balloon_background;
    let inner_x = panel_x + BALLOON_INNER_PADDING;
    let inner_w = panel_w - 2 * BALLOON_INNER_PADDING;
    let text_w = (font.character_size.width as i32 + font.character_spacing as i32)
        * text.chars().count() as i32;
    let mid_y = panel_y + panel_h / 2;
    let elapsed_ms = ctx.now_ms.wrapping_sub(ctx.balloon_set_ms);
    let hold_ms = ctx.balloon_hold_ms;

    let mut done = false;
    let scrolling = text_w > inner_w;
    let x = if scrolling {
        // Marquee: text starts just past the right inner edge and scrolls left.
        let one_pass_px = text_w + inner_w;
        let cycle_px = one_pass_px + BALLOON_REPEAT_GAP_PX;
        let offset_in_cycle =
            (elapsed_ms as i32).wrapping_mul(BALLOON_SCROLL_PX_PER_SEC) / 1000 % cycle_px;
        let one_pass_ms = one_pass_px as u32 * 1000 / BALLOON_SCROLL_PX_PER_SEC as u32;
        if elapsed_ms >= hold_ms.max(one_pass_ms) {
            done = true;
        }
        inner_x + inner_w - offset_in_cycle
    } else {
        if elapsed_ms >= hold_ms.max(BALLOON_DEFAULT_STATIC_HOLD_MS) {
            done = true;
        }
        panel_x + panel_w / 2
    };

    let Some(rect) = clamp_rect(panel_x, panel_y, panel_w, panel_h, canvas_w, canvas_h) else {
        return Ok(done);
    };
    ensure_scratch::<D::Color>(scratch, &rect);
    let Ok(mut sprite) = Sprite::<D::Color>::new_unaligned(scratch, rect) else {
        return Ok(done);
    };
    let _ = sprite.clear(ctx.palette.background);
    let panel = RoundedRectangle::new(
        Rectangle::new(
            Point::new(panel_x, panel_y),
            Size::new(panel_w as u32, panel_h as u32),
        ),
        CornerRadii::new(Size::new(BALLOON_PANEL_RADIUS, BALLOON_PANEL_RADIUS)),
    );
    let _ = panel.into_styled(PrimitiveStyle::with_fill(bg)).draw(&mut sprite);
    let _ = panel
        .into_styled(PrimitiveStyle::with_stroke(fg, 1))
        .draw(&mut sprite);
    // The scratch sprite covers exactly the panel rect, so it doubles as the marquee
    // clip region.
    let style = MonoTextStyle::new(font, fg);
    let text_style = TextStyleBuilder::new()
        .alignment(if scrolling {
            Alignment::Left
        } else {
            Alignment::Center
        })
        .baseline(Baseline::Middle)
        .build();
    let _ = Text::with_text_style(text, Point::new(x, mid_y), style, text_style).draw(&mut sprite);
    sprite.draw(display)?;
    Ok(done)
}
