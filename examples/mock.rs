use std::{time::{UNIX_EPOCH, Duration, SystemTime}};

use embedded_graphics::{pixelcolor::BinaryColor, prelude::{Size, DrawTarget}};
use m5stack_avatar_rs::{Avatar, components::face::DrawContext, Palette, BasicPaletteKey, Timer, Expression};
use embedded_graphics_simulator::{SimulatorDisplay, Window, OutputSettingsBuilder, BinaryColorTheme, SimulatorEvent};
struct StdTimer {}

impl Timer for StdTimer {
    fn timestamp_milliseconds(&self) -> u64 {
        let now = SystemTime::now().duration_since(UNIX_EPOCH.into()).unwrap();
        let milliseconds = now.as_millis();
        milliseconds as u64
    }
}

fn main() -> Result<(), std::convert::Infallible> {
    let mut display = SimulatorDisplay::<BinaryColor>::new(Size::new(320, 240));

    let mut context: DrawContext<BinaryColor> = DrawContext::default();
    context.palette.set_color(&BasicPaletteKey::Primary, BinaryColor::On);
    context.palette.set_color(&BasicPaletteKey::Secondary, BinaryColor::On);
    context.palette.set_color(&BasicPaletteKey::Background, BinaryColor::Off);
    
    let mut avatar = Avatar::new(context, 30);
    let timer = StdTimer{};
    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    let mut window = Window::new("Avatar", &output_settings);
    loop {
        display.clear(BinaryColor::Off)?;

        avatar.context().expression = Expression::Happy;
        avatar.run(&mut display, &timer)?;
        window.update(&display);

        if window.events().any(|e| e == SimulatorEvent::Quit) {
            break;
        }
        std::thread::sleep(Duration::from_millis(1000/30));
    }

    Ok(())
}