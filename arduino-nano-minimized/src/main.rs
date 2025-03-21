#![no_std]
#![no_main]

mod abstract_device;
mod debouncing;
mod error;
mod game;

use crate::abstract_device::{AbstractDevice, Inputs};
use crate::error::Error;
use core::fmt::Write;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::i2c::I2c;
use ssd1306::mode::{BufferedGraphicsMode, DisplayConfig};
use ssd1306::prelude::{DisplayRotation, I2CInterface};
use ssd1306::size::DisplaySize128x32;
use ssd1306::Ssd1306;

use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // let display = adafruit_ssd1306::AdafruitSSD1306Driver::new(i2c);
    let button1_pin = pins.d5.into_pull_up_input();
    let button2_pin = pins.d4.into_pull_up_input();
    let mut led_pin = pins.d13.into_output();
    // loop{
    for i in 0..30 {
        led_pin.set_high();
        arduino_hal::delay_ms(100);
        led_pin.set_low();
        arduino_hal::delay_ms(100);
    }

    // examples and inspiration, head to https://github.com/Rahix/avr-hal/tree/main/examples
    let _result: Result<(), Error> = (|| {
        let i2c = arduino_hal::I2c::new(
            dp.TWI,
            pins.a4.into_pull_up_input(),
            pins.a5.into_pull_up_input(),
            50000,
        );
        let interface = ssd1306::I2CDisplayInterface::new(i2c);
        let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
        // Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0).into_terminal_mode();
        display.init().unwrap();
        // loop {
        //     led_pin.set_high();
        //     display.clear(BinaryColor::On)?;
        //     display.flush()?;
        //     arduino_hal::delay_ms(200);
        //     led_pin.set_low();
        //     display.clear(BinaryColor::Off)?;
        //     display.flush()?;
        //     arduino_hal::delay_ms(200);
        // }

        // let result: core::result::Result<(), core::convert::Infallible> = ufmt::uwrite!(&mut serial, "Hello from rust arduino!");
        // let type_name = get_type_name(result);
        let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
        let analog_pin = pins.a0.into_analog_input(&mut adc);
        let seed = analog_pin.analog_read(&mut adc);
        // display
        //     .write_str("Sequence memory! Try buttons. Push both buttons to start.")
        //     .unwrap();

        // result.unwrap_infallible();
        // ufmt::uwrite!(&mut serial, "{}", type_name).unwrap_infallible();
        let mut device = Device {
            display_storage: display,
            button1_pin,
            button2_pin,
            led_pin: &mut led_pin,
            seed: seed as u64,
        };
        for i in 0..10 {
            device.led_pin.set_high();
            device.display_storage.clear(BinaryColor::On)?;
            // device.display_storage.flush()?;
            arduino_hal::delay_ms(200);
            device.led_pin.set_low();
            device.display_storage.clear(BinaryColor::Off)?;
            device.display_storage.flush()?;
            arduino_hal::delay_ms(200);
        }
        // loop {
        //     device.led_pin.set_high();
        //     arduino_hal::delay_ms(200);
        //     device.led_pin.set_low();
        //     arduino_hal::delay_ms(200);
        // }
        let mut game = crate::game::Game::new(device)?;
        game.run_game()?;
        Ok(())
    })();
    loop {
        led_pin.toggle();
        arduino_hal::delay_ms(100);
    }
}

struct Device<I2C, Button1Pin, Button2Pin, LedPin> {
    display_storage:
        Ssd1306<I2CInterface<I2C>, DisplaySize128x32, BufferedGraphicsMode<DisplaySize128x32>>,
    button1_pin: Button1Pin,
    button2_pin: Button2Pin,
    led_pin: LedPin,
    seed: u64,
}
impl<I2C, Button1Pin, Button2Pin, LedPin> AbstractDevice
    for Device<I2C, Button1Pin, Button2Pin, LedPin>
where
    I2C: I2c,
    Button1Pin: InputPin,
    Button2Pin: InputPin,
    LedPin: OutputPin,
{
    type Display =
        Ssd1306<I2CInterface<I2C>, DisplaySize128x32, BufferedGraphicsMode<DisplaySize128x32>>;
    type Error = Error;

    fn get_inputs(&mut self) -> Result<Inputs, Self::Error> {
        let inputs = Inputs {
            button1_down: self.button1_pin.is_low().unwrap(),
            button2_down: self.button2_pin.is_low().unwrap(),
        };
        Ok(inputs)
    }

    fn set_led(&mut self, new_state: bool) {
        if new_state {
            self.led_pin.set_high().unwrap()
        } else {
            self.led_pin.set_low().unwrap()
        }
    }

    fn delay_ms(&mut self, ms: u32) {
        arduino_hal::delay_ms(ms as u16);
    }

    fn get_rng_seed(&mut self) -> u64 {
        self.seed
    }

    fn display(&mut self) -> &mut Self::Display {
        &mut self.display_storage
    }

    fn flush_display(&mut self) -> Result<(), Self::Error> {
        self.display_storage.clear_buffer();
        self.display_storage.flush()?;
        Ok(())
    }
}
