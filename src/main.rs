#![cfg_attr(not(target_arch = "x86_64"), no_std)]
#![cfg_attr(not(target_arch = "x86_64"), no_main)]

use core::panic::PanicInfo;
use std::time::Duration;
use std::{process, thread};

mod abstract_device;
mod debouncing;
mod error;
mod game;
mod main_rp2040;

use embedded_graphics::mono_font::ascii::*;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::{Point, Primitive, Size};
use embedded_graphics::primitives::{Circle, Line, PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;
use embedded_graphics::{Drawable, Pixel};

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
#[cfg(not(target_arch = "x86_64"))]
use rp_pico as bsp;

use crate::abstract_device::{AbstractDevice, Inputs};
use crate::error::Error;
use crate::game::Game;
use embedded_graphics_simulator::sdl2::Keycode;
use embedded_graphics_simulator::SimulatorEvent;
#[cfg(target_arch = "x86_64")]
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};

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
    let device = Device {
        simulator_display: display,
        window,
        has_updated: false,
        inputs: Inputs::default(),
    };
    let mut game = Game::new(device)?;
    let result = game.run_game();

    if let Err(Error::Quit) = result {
        Ok(())
    } else {
        result
    }
}

struct Device {
    simulator_display: SimulatorDisplay<BinaryColor>,
    window: Window,
    has_updated: bool,
    inputs: Inputs,
}
impl AbstractDevice for Device {
    type Display = SimulatorDisplay<BinaryColor>;
    type Error = Error;

    fn get_inputs(&mut self) -> Result<Inputs, Error> {
        if !self.has_updated {
            self.has_updated = true;
            self.window.update(&self.simulator_display);
        }
        for event in self.window.events() {
            match event {
                SimulatorEvent::KeyUp {
                    keycode,
                    keymod: _,
                    repeat: _,
                } => match keycode {
                    Keycode::Z => {
                        self.inputs.button1_down = false;
                    }
                    Keycode::X => {
                        self.inputs.button2_down = false;
                    }
                    _ => (),
                },
                SimulatorEvent::KeyDown {
                    keycode,
                    keymod: _,
                    repeat: _,
                } => match keycode {
                    Keycode::Z => {
                        self.inputs.button1_down = true;
                    }
                    Keycode::X => {
                        self.inputs.button2_down = true;
                    }
                    Keycode::Escape => {
                        return Err(Error::Quit);
                    }
                    _ => (),
                },
                SimulatorEvent::Quit => {
                    return Err(Error::Quit);
                }
                _ => {}
            }
        }
        Ok(self.inputs)
    }

    fn set_led(&mut self, _new_state: bool) {}

    fn delay_ms(&mut self, ms: u32) {
        thread::sleep(Duration::from_millis(ms as u64))
    }

    fn get_rng_seed(&mut self) -> u64 {
        4
    }

    fn display(&mut self) -> &mut Self::Display {
        &mut self.simulator_display
    }

    fn flush_display(&mut self) -> Result<(), Error> {
        self.window.update(&self.simulator_display);
        self.has_updated = true;
        Ok(())
    }
}
