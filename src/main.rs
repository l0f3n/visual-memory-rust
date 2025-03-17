//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

mod error;
mod game;
mod debouncing;

use bsp::entry;
use core::cell::RefCell;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::{InputPin, OutputPin};
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
use cortex_m::prelude::_embedded_hal_adc_OneShot;
use rp2040_hal::{
    Adc,
    I2C,
    uart::{DataBits, StopBits, UartConfig, UartPeripheral},
    fugit::RateExtU32,
    adc::AdcPin,
};
use ssd1306::mode::{BufferedGraphicsMode, DisplayConfig};
use ssd1306::prelude::{DisplayRotation, DisplaySize128x32, I2CInterface};
use ssd1306::Ssd1306;
use crate::error::Error;
use crate::game::ValidDisplay;

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
    let mut button1_pin = pins.gpio7.into_pull_up_input();
    let mut button2_pin = pins.gpio8.into_pull_up_input();
    let mut adc = Adc::new(pac.ADC, &mut pac.RESETS);
    let mut adc_pin_0 = AdcPin::new(pins.gpio28.into_floating_input()).unwrap();
    let seed: u16 = adc.read(&mut adc_pin_0).unwrap();

    // use embedded_hal_0_2::adc::OneShot;
    // use rp2040_hal::{adc::Adc, adc::AdcPin, gpio::Pins, pac, Sio};
    // let mut peripherals = pac::Peripherals::take().unwrap();
    // let sio = Sio::new(peripherals.SIO);
    // let pins = Pins::new(peripherals.IO_BANK0, peripherals.PADS_BANK0, sio.gpio_bank0, &mut peripherals.RESETS);
    // Enable adc
    // Configure one of the pins as an ADC input
    // Read the ADC counts from the ADC channel
    // pins(&mut led_pin, &mut button1_pin);

    let result = (|| -> Result<(), error::Error> {
        let i2c = I2C::i2c0(
            pac.I2C0,
            pins.gpio20.reconfigure(), // sda
            pins.gpio21.reconfigure(), // scl
            400.kHz(),
            &mut pac.RESETS,
            125_000_000.Hz(),
        );
        let i2c_ref_cell = RefCell::new(i2c);
        let i2c = embedded_hal_bus::i2c::RefCellDevice::new(&i2c_ref_cell);
        let interface = ssd1306::I2CDisplayInterface::new(i2c);
        let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
        display.init()?;

        // let mut game = game::Game::new(button1_pin, button2_pin, &mut led_pin, &mut delay, display, seed as u64)?;
        let get_button_1 = || {
            button1_pin.is_low().unwrap()
        };
        let get_button_2 = || {
            button2_pin.is_low().unwrap()
        };
        let set_led = |value: bool| {
            if value {
                led_pin.set_high().unwrap();
            } else {
                led_pin.set_low().unwrap();
            }
        };
        let delay_ms = |ms: u32| {
            delay.delay_ms(ms);
        };
        let mut game = game::Game::new(get_button_1, get_button_2, set_led, delay_ms, display, seed as u64)?;
        game.run_game()?;
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

impl<I2C> ValidDisplay
for Ssd1306<I2CInterface<I2C>, DisplaySize128x32, BufferedGraphicsMode<DisplaySize128x32>>
where
    I2C: embedded_hal::i2c::I2c,
{
    fn flush(&mut self) -> Result<(), Error> {
        Ssd1306::flush(self)?;
        Ok(())
    }
}

