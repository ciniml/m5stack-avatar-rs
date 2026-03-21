use std::io::Write as IoWrite;

use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::{DrawTarget, OriginDimensions, Size},
    Pixel,
};
use m5stack_avatar_rs::{
    Avatar,
    components::face::DrawContext,
    Palette, BasicPaletteKey, Timer,
};

// タイムスタンプを固定値で進める決定論的タイマー
struct DeterministicTimer(u64);

impl Timer for DeterministicTimer {
    fn timestamp_milliseconds(&self) -> u64 {
        self.0
    }
}

// BinaryColor 用のピクセルバッファ（PBM 出力付き）
struct PbmDisplay {
    width: u32,
    height: u32,
    pixels: Vec<bool>,
}

impl PbmDisplay {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![false; (width * height) as usize],
        }
    }

    fn clear(&mut self) {
        self.pixels.fill(false);
    }

    fn save(&self, path: &str) -> std::io::Result<()> {
        let mut f = std::fs::File::create(path)?;
        writeln!(f, "P1")?;
        writeln!(f, "{} {}", self.width, self.height)?;
        for row in 0..self.height {
            for col in 0..self.width {
                let on = self.pixels[(row * self.width + col) as usize];
                write!(f, "{}", if on { 1 } else { 0 })?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl DrawTarget for PbmDisplay {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            if point.x >= 0
                && point.x < self.width as i32
                && point.y >= 0
                && point.y < self.height as i32
            {
                let idx = (point.y as u32 * self.width + point.x as u32) as usize;
                self.pixels[idx] = color == BinaryColor::On;
            }
        }
        Ok(())
    }
}

impl OriginDimensions for PbmDisplay {
    fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}

fn make_context() -> DrawContext<BinaryColor, String> {
    let mut ctx: DrawContext<BinaryColor, String> = DrawContext::default();
    ctx.palette.set_color(&BasicPaletteKey::Primary, BinaryColor::On);
    ctx.palette.set_color(&BasicPaletteKey::Secondary, BinaryColor::On);
    ctx.palette.set_color(&BasicPaletteKey::Background, BinaryColor::Off);
    ctx.palette.set_color(&BasicPaletteKey::BalloonForeground, BinaryColor::On);
    ctx.palette.set_color(&BasicPaletteKey::BalloonBackground, BinaryColor::Off);
    ctx
}

const FRAMES: usize = 90;
const FPS: u64 = 30;

fn main() -> Result<(), std::convert::Infallible> {
    std::fs::create_dir_all("frames/320x240").unwrap();
    std::fs::create_dir_all("frames/128x128").unwrap();

    // 320x240
    {
        let context = make_context();
        let mut avatar = Avatar::new(context, FPS);
        let mut display = PbmDisplay::new(320, 240);
        let mut timer = DeterministicTimer(0);
        for frame in 0..FRAMES {
            display.clear();
            avatar.run(&mut display, &timer)?;
            timer.0 += 1000 / FPS;
            display
                .save(&format!("frames/320x240/frame_{:04}.pbm", frame))
                .unwrap();
        }
        println!("Saved {} frames to frames/320x240/", FRAMES);
    }

    // 128x128
    {
        let context = make_context();
        let mut avatar = Avatar::new_small(context, FPS);
        let mut display = PbmDisplay::new(128, 128);
        let mut timer = DeterministicTimer(0);
        for frame in 0..FRAMES {
            display.clear();
            avatar.run(&mut display, &timer)?;
            timer.0 += 1000 / FPS;
            display
                .save(&format!("frames/128x128/frame_{:04}.pbm", frame))
                .unwrap();
        }
        println!("Saved {} frames to frames/128x128/", FRAMES);
    }

    Ok(())
}
