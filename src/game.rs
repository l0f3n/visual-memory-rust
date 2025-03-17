#![allow(unused_imports)]

use crate::debouncing;
use crate::debouncing::{DebounceResult, Debouncer};
use crate::error::Error;
use core::fmt::Binary;
use core::mem::MaybeUninit;
use cortex_m::delay::Delay;
use defmt::*;
use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::ascii::*;
use embedded_graphics::mono_font::{MonoTextStyle, MonoTextStyleBuilder};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::text::{Baseline, Text};
use embedded_graphics::Drawable;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_bus::i2c::RefCellDevice;
use fixed_slice_vec::FixedSliceVec;
use rp2040_hal::gpio::bank0::{Gpio0, Gpio1, Gpio20, Gpio21, Gpio25};
use rp2040_hal::gpio::{FunctionI2c, FunctionSio, FunctionUart, Pin, PullDown, PullUp, SioOutput};
use rp2040_hal::pac::{I2C0, UART0};
use rp2040_hal::uart::{Enabled, UartDevice, UartPeripheral, ValidUartPinout};
use rp2040_hal::I2C;
use ssd1306::mode::{BufferedGraphicsMode, DisplayConfig};
use ssd1306::prelude::{DisplayRotation, DisplaySize128x32, I2CInterface};
use ssd1306::Ssd1306;

#[derive(Format, PartialEq, Clone, Copy)]
enum GameState {
    Menu,
    Displaying,
    Inputting,
    Next,
    Failure,
    Score,
}

pub struct Game<
    'a,
    B1Pin: InputPin,
    B2Pin: InputPin,
    LedPin: OutputPin,
    I2C: embedded_hal::i2c::I2c,
    D: UartDevice,
    P: ValidUartPinout<D>,
> {
    button1_pin: B1Pin,
    button2_pin: B2Pin,
    led_pin: &'a mut LedPin,
    delay: &'a mut Delay,
    uart: UartPeripheral<Enabled, D, P>,
    display: Ssd1306<I2CInterface<I2C>, DisplaySize128x32, BufferedGraphicsMode<DisplaySize128x32>>,
    text_style: MonoTextStyle<'a, BinaryColor>,
    cursor: Point,
}

impl<
    'a,
    B1Pin: InputPin,
    B2Pin: InputPin,
    LedPin: OutputPin,
    I2C: embedded_hal::i2c::I2c,
    D: UartDevice,
    P: ValidUartPinout<D>,
> Game<'a, B1Pin, B2Pin, LedPin, I2C, D, P>
{
    pub fn new(
        mut button1_pin: B1Pin,
        mut button2_pin: B2Pin,
        led_pin: &'a mut LedPin,
        delay: &'a mut Delay,
        i2c: I2C,
        uart: UartPeripheral<Enabled, D, P>,
    ) -> Result<Self, Error> {
        let interface = ssd1306::I2CDisplayInterface::new(i2c);
        let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
        display.init()?;
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();
        Ok(Self {
            button1_pin,
            button2_pin,
            led_pin,
            delay,
            uart,
            display,
            text_style,
            cursor: Point::zero(),
        })
    }
    pub fn run_game(&mut self) -> Result<(), Error> {
        let mut debouncer_storage = [0x00u8; 2];
        let mut debounce = debouncing::Debouncer::new(&mut debouncer_storage);
        // let mut rng = rand::rng();

        // Text::with_baseline("Hello Rust!", Point::new(0, 9), text_style, Baseline::Top)
        //     .draw(&mut display)?;
        // Text::with_baseline("Hello Rust!", Point::new(0, 18), text_style, Baseline::Top)
        //     .draw(&mut display)?;

        const MAX_SEQUENCE: usize = 128;
        let mut sequence_storage = [MaybeUninit::new(false); MAX_SEQUENCE];
        let mut sequence = fixed_slice_vec::FixedSliceVec::new(&mut sequence_storage[..]);
        sequence.clear();
        let mut game_state = GameState::Menu;
        let mut last_game_state = GameState::Score;
        let mut next_guess_index = 0;
        let mut highest_cleared = 0;
        let mut first = true;

        loop {
            let button1_down = self.button1_pin.is_low().unwrap();
            let button2_down = self.button2_pin.is_low().unwrap();
            let button1_fell = debounce.update(0, button1_down) == DebounceResult::Pressed;
            let button2_fell = debounce.update(1, button2_down) == DebounceResult::Pressed;

            if button1_fell {
                info!("B1 pressed");
            }
            if button2_fell {
                info!("B2 pressed");
            }

            self.display.clear_buffer();
            self.reset_cursor();

            if last_game_state != game_state {
                last_game_state = game_state;
                info!("New state: {}", game_state);
            }
            match game_state {
                GameState::Menu => {
                    self.write_string(
                        "Sequence memory! Try\nbuttons. Push both\nbuttons to start.",
                    )?;
                    if button1_down && button2_down {
                        game_state = GameState::Displaying;
                        next_guess_index = 0;
                        highest_cleared = 0;
                        self.generate_sequence(3, &mut sequence);
                        first = true;
                    } else {
                        if button1_fell {
                            sequence.push(false);
                        } else if button1_fell {
                            sequence.push(true);
                        }
                        self.render_sequence(&sequence, sequence.len())?;
                    }
                }
                GameState::Displaying => {
                    if first {
                        self.display_temporary_message("Remember!", 1000)?;
                    }
                    self.write_string(": ")?;
                    self.render_sequence(&sequence, sequence.len())?;
                    self.display.flush()?;
                    self.delay.delay_ms(2000);
                    game_state = GameState::Inputting;
                    if first {
                        self.display_temporary_message("Repeat!", 1000)?;
                    }
                }
                GameState::Inputting => {
                    self.write_string(": ")?;
                    // let next_point =
                    //     Text::with_baseline(": ", Point::zero(), text_style, Baseline::Top)
                    //         .draw(&mut display)?;
                    if button1_fell {
                        if sequence[next_guess_index] == false {
                            next_guess_index += 1;
                        } else {
                            game_state = GameState::Failure;
                        }
                    } else if button2_fell {
                        if sequence[next_guess_index] == true {
                            next_guess_index += 1;
                        } else {
                            game_state = GameState::Failure;
                        }
                    }
                    self.render_sequence(&sequence, next_guess_index)?;
                    if next_guess_index == sequence.len() {
                        game_state = GameState::Next;
                    }
                }
                GameState::Next => {
                    if first {
                        self.display_temporary_message("Good!", 1000)?;
                        self.display_temporary_message("Next sequence:", 1000)?;
                    } else {
                        self.display_temporary_message("Good! Next:", 400)?;
                    }
                    self.generate_sequence(sequence.len() + 1, &mut sequence);
                    next_guess_index = 0;
                    highest_cleared = sequence.len();
                    game_state = GameState::Displaying;
                    first = false;
                }
                GameState::Failure => {
                    self.display_temporary_message("No!", 200)?;
                    self.render_sequence(&sequence, sequence.len())?;
                    self.display.flush()?;
                    self.delay.delay_ms(2000);
                    game_state = GameState::Score;
                }
                GameState::Score => {
                    self.write_string("You cleared ")?;
                    let score =
                        highest_cleared as f32 + next_guess_index as f32 / sequence.len() as f32;
                    self.write_float_string(score)?;
                    self.write_string("\nsequences!")?;
                    if button1_fell || button2_fell {
                        game_state = GameState::Menu;
                        sequence.clear();
                    }
                }
            }
            self.display.flush()?;

            self.led_pin.set_high().unwrap();
            self.delay.delay_ms(5);
            self.led_pin.set_low().unwrap();
            self.delay.delay_ms(5);
        }
    }

    fn generate_sequence(&self, length: usize, sequence: &mut FixedSliceVec<bool>) {
        sequence.clear();
        for i in 0..length {
            sequence.push(i % 2 == 0);
        }
    }

    fn reset_cursor(&mut self) {
        self.cursor = Point::zero();
    }
    fn write_string(&mut self, string: &str) -> Result<(), Error> {
        let next_point = Text::with_baseline(string, self.cursor, self.text_style, Baseline::Top)
            .draw(&mut self.display)?;
        self.cursor = next_point;
        Ok(())
    }

    fn render_sequence(&mut self, sequence: &FixedSliceVec<bool>, subset_length: usize) -> Result<(), Error> {
        for i in 0..subset_length {
            let value = sequence[i];
            if value {
                self.write_string("1")?;
            } else {
                self.write_string("0")?;
            }
        }
        Ok(())
    }

    fn display_temporary_message(&mut self, string: &str, ms: u32) -> Result<(), Error> {
        self.display.clear_buffer();
        self.reset_cursor();
        self.write_string(string)?;
        self.display.flush()?;
        self.delay.delay_ms(ms);
        self.display.clear_buffer();
        self.reset_cursor();
        Ok(())
    }

    fn write_float_string(&mut self, value: f32) -> Result<(), Error> {
        let mut buffer = [0x00u8; 12];
        let string = format_no_std::show(&mut buffer, format_args!(""))?;
        self.write_string(string)?;
        Ok(())
    }
}
