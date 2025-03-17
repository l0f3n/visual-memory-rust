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
use embedded_hal::digital::OutputPin;
use crate::error::Error;
use defmt::*;

pub fn run_game<PIN: OutputPin, I2C: embedded_hal::i2c::I2c, D, P>(
    led_pin: &mut PIN,
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
    Text::with_baseline("One of the saddest\nlessons of history\nis this. If we've been", Point::zero(), text_style, Baseline::Top)
        .draw(&mut display)?;

    // Text::with_baseline("Hello Rust!", Point::new(0, 9), text_style, Baseline::Top)
    //     .draw(&mut display)?;
    // Text::with_baseline("Hello Rust!", Point::new(0, 18), text_style, Baseline::Top)
    //     .draw(&mut display)?;

    display.flush()?;

    let mut i = 0;
    loop {
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