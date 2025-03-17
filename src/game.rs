use cortex_m::delay::Delay;
use rp2040_hal::uart::{Enabled, UartDevice, UartPeripheral, ValidUartPinout};
use rp2040_hal::pac::{I2C0, UART0};
use rp2040_hal::gpio::{FunctionI2c, FunctionSio, FunctionUart, Pin, PullDown, PullUp, SioOutput};
use rp2040_hal::gpio::bank0::{Gpio0, Gpio1, Gpio20, Gpio21, Gpio25};
use embedded_hal_bus::i2c::RefCellDevice;
use rp2040_hal::I2C;
use ssd1306::Ssd1306;
use ssd1306::prelude::{DisplayRotation, DisplaySize128x32};
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::mono_font::ascii::*;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::text::{Baseline, Text};
use embedded_graphics::geometry::Point;
use ssd1306::mode::DisplayConfig;
use embedded_graphics::Drawable;
use embedded_hal::digital::{InputPin, OutputPin};
use crate::error::Error;
use defmt::*;
use crate::debouncing;
use crate::debouncing::DebounceResult;

pub fn run_game<B1Pin: InputPin, B2Pin: InputPin, LedPin: OutputPin, I2C: embedded_hal::i2c::I2c, D, P>(
    mut button1_pin: B1Pin,
    mut button2_pin: B2Pin,
    led_pin: &mut LedPin,
    delay: &mut Delay,
    i2c: I2C,
    uart: UartPeripheral<Enabled, D, P>,
) -> Result<(), Error>
where
    D: UartDevice,
    P: ValidUartPinout<D>,
{
    let interface = ssd1306::I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init()?;
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();
    let mut debouncer_storage = [0x00u8; 2];
    let mut debounce = debouncing::Debouncer::new(&mut debouncer_storage);
    Text::with_baseline("One of the saddest\nlessons of history\nis this. If we've been", Point::zero(), text_style, Baseline::Top)
        .draw(&mut display)?;

    // Text::with_baseline("Hello Rust!", Point::new(0, 9), text_style, Baseline::Top)
    //     .draw(&mut display)?;
    // Text::with_baseline("Hello Rust!", Point::new(0, 18), text_style, Baseline::Top)
    //     .draw(&mut display)?;

    display.flush()?;

    let mut i = 0;
    loop {
        let button1_down = button1_pin.is_low().unwrap();
        let button2_down = button2_pin.is_low().unwrap();
        let button1_pressed = debounce.update(0, button1_down) == DebounceResult::Pressed;
        let button2_pressed = debounce.update(1, button2_down) == DebounceResult::Pressed;

        uart.write_full_blocking(b"Uart loop\r\n");
        info!("on!");
        i += 1;
        led_pin.set_high().unwrap();
        delay.delay_ms(2000);
        info!("off! {}", i);
        led_pin.set_low().unwrap();
        delay.delay_ms(2000);
    }
}