#![cfg_attr(not(target_arch = "x86_64"), no_std)]
#![cfg_attr(not(target_arch = "x86_64"), no_main)]

use core::panic::PanicInfo;

mod error;
mod game;
mod debouncing;
mod main_rp2040;

use embedded_graphics::Drawable;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::*;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::{Point, Primitive, Size};
use embedded_graphics::primitives::{Circle, Line, PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;

#[cfg(not(target_arch = "x86_64"))]
use rp_pico as bsp;
#[cfg(not(target_arch = "x86_64"))]
use bsp::entry;
#[cfg(not(target_arch = "x86_64"))]
use defmt::*;
#[cfg(not(target_arch = "x86_64"))]
use defmt_rtt as _;
#[cfg(not(target_arch = "x86_64"))]
use panic_probe as _;

#[cfg(target_arch = "x86_64")]
use embedded_graphics_simulator::{BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window};

#[cfg(target_arch = "arm")]
#[entry]
#[allow(unreachable_code)]
fn main() -> ! {
    info!("Program start");

    #[cfg(target_arch = "arm")]
    main_rp2040::main_rp2040();
    info!("End");
    loop {}
}


#[cfg(target_arch = "x86_64")]
fn main() -> Result<(), core::convert::Infallible> {
    let mut display = SimulatorDisplay::<BinaryColor>::new(Size::new(128, 32));
    let line_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    Circle::new(Point::new(72, 8), 48)
        .into_styled(line_style)
        .draw(&mut display)?;

    Line::new(Point::new(48, 16), Point::new(8, 16))
        .into_styled(line_style)
        .draw(&mut display)?;

    Line::new(Point::new(48, 16), Point::new(64, 32))
        .into_styled(line_style)
        .draw(&mut display)?;

    Rectangle::new(Point::new(79, 15), Size::new(34, 34))
        .into_styled(line_style)
        .draw(&mut display)?;

    Text::new("Hello World!", Point::new(5, 5), text_style).draw(&mut display)?;
    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    Window::new("Hello World", &output_settings).show_static(&display);

    Ok(())
}