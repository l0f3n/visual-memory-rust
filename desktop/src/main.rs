mod error;
use crate::error::Error;
use embedded_graphics::geometry::Size;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::ImageDrawable;
use embedded_graphics_simulator::sdl2::Keycode;
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use program::abstract_device::{AbstractDevice, Inputs};
use program::game::Game;
use rand::RngCore;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Error> {
    let mut display = SimulatorDisplay::<BinaryColor>::new(Size::new(128, 128));
    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    let window = Window::new("Hello World", &output_settings);
    let seed = rand::rng().next_u64();

    // let text_style = MonoTextStyleBuilder::new()
    //     .font(&FONT_6X10)
    //     .text_color(BinaryColor::On)
    //     .build();
    let mut device = Device {
        simulator_display: display,
        window,
        has_updated: false,
        inputs: Inputs::default(),
        seed,
    };
    device.flush_display()?;
    loop {
        FONT_6X10.image.draw(device.display())?;
        device.flush_display()?;
        thread::sleep(Duration::from_millis(400))
    }
    Ok(())
    // let mut game = Game::new(device)?;
    // let result = game.run_game();

    // if let Err(Error::Quit) = result {
    //     Ok(())
    // } else {
    //     result
    // }
}

struct Device {
    simulator_display: SimulatorDisplay<BinaryColor>,
    window: Window,
    has_updated: bool,
    inputs: Inputs,
    seed: u64,
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
        self.seed
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
