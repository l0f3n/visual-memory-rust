#![cfg_attr(not(target_arch = "x86_64"), no_std)]
#![cfg_attr(not(target_arch = "x86_64"), no_main)]

use core::panic::PanicInfo;
use std::time::Duration;

mod error;
mod game;
mod debouncing;
mod main_rp2040;

use embedded_graphics::{Drawable, Pixel};
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
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Dimensions;
#[cfg(not(target_arch = "x86_64"))]
use panic_probe as _;

#[cfg(target_arch = "x86_64")]
use embedded_graphics_simulator::{BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window};
use crate::error::Error;
use crate::game::Game;

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
fn main() -> Result<(), Error> {
    let mut display = SimulatorDisplay::<BinaryColor>::new(Size::new(128, 32));
    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    let window = Window::new("Hello World", &output_settings);
    let display_and_window = DisplayAndWindow {
        display,
        window,
    };
    let get_button_1 = || {
        false
    };
    let get_button_2 = || {
        false
    };
    let set_led = |value| {

    };
    let delay_ms = |ms| {
        std::thread::sleep(Duration::from_millis(ms as u64));
    };
    let seed = 4;
    let mut game = Game::new(get_button_1, get_button_2, set_led, delay_ms, display_and_window, seed)?;
    game.run_game()?;


    // Window::new("Hello World", &output_settings).show_static(&display);
    // let line_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    // let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    // Circle::new(Point::new(72, 8), 48)
    //     .into_styled(line_style)
    //     .draw(&mut display)?;
    //
    // Line::new(Point::new(48, 16), Point::new(8, 16))
    //     .into_styled(line_style)
    //     .draw(&mut display)?;
    //
    // Line::new(Point::new(48, 16), Point::new(64, 32))
    //     .into_styled(line_style)
    //     .draw(&mut display)?;
    //
    // Rectangle::new(Point::new(79, 15), Size::new(34, 34))
    //     .into_styled(line_style)
    //     .draw(&mut display)?;
    //
    // Text::new("Hello World!", Point::new(5, 5), text_style).draw(&mut display)?;

    Ok(())
}

struct DisplayAndWindow {
    display: SimulatorDisplay<BinaryColor>,
    window: Window,
}

impl DrawTarget for DisplayAndWindow {
    type Color = BinaryColor;
    type Error = <SimulatorDisplay<BinaryColor> as DrawTarget>::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item=Pixel<Self::Color>>
    {
        self.display.draw_iter(pixels)
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item=Self::Color>
    {
        self.display.fill_contiguous(area, colors)
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        self.display.fill_solid(area, color)
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.display.clear(color)
    }
}

impl Dimensions for DisplayAndWindow {
    fn bounding_box(&self) -> Rectangle {
        self.display.bounding_box()
    }
}

impl game::ValidDisplay for DisplayAndWindow {
    fn flush(&mut self) -> Result<(), Error> {
        self.window.update(&self.display);
        Ok(())
    }
}
impl crate::game::ValidDisplay for SimulatorDisplay<BinaryColor> {
    fn flush(&mut self) -> Result<(), Error> {
        let output_settings = OutputSettingsBuilder::new()
            .theme(BinaryColorTheme::OledBlue)
            .build();
        Window::new("Hello World", &output_settings).update(&self);
        Ok(())
    }
}