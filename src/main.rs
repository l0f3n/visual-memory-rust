//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

mod error;
mod game;

use bsp::entry;
use core::cell::RefCell;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::OutputPin;
#[cfg(not(target_arch = "x86_64"))]
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};
use rp2040_hal::fugit::RateExtU32;
use rp2040_hal::uart::{DataBits, Enabled, StopBits, UartConfig, UartPeripheral};
use rp2040_hal::I2C;

use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use rp2040_hal::gpio::bank0::{Gpio1, Gpio20, Gpio21, Gpio25};
use rp2040_hal::gpio::{FunctionSio, FunctionUart, Pin, PullDown, PullUp, SioOutput};
use rp2040_hal::pac::UART0;

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let uart_pins = (pins.gpio0.into_function(), pins.gpio1.into_function());
    let uart = UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(115200.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    uart.write_full_blocking(b"Hello World!\r\n");

    let mut led_pin = pins.led.into_push_pull_output();

    let result = (|| -> Result<(), error::Error> {
        let i2c = I2C::i2c0(
            pac.I2C0,
            pins.gpio20.reconfigure(), // sda
            pins.gpio21.reconfigure(), // scl
            50.kHz(),
            &mut pac.RESETS,
            125_000_000.Hz(),
        );
        let i2c_ref_cell = RefCell::new(i2c);
        let i2c = embedded_hal_bus::i2c::RefCellDevice::new(&i2c_ref_cell);

        game::run_game(&mut led_pin, &mut delay, i2c, uart)?;
        Ok(())
    })();
    if let Err(error) = result {
        info!("Error: {}", error);
        loop {
            led_pin.set_high().unwrap();
            delay.delay_ms(100);
            led_pin.set_low().unwrap();
            delay.delay_ms(100);
        }
    }
    loop {}
}
