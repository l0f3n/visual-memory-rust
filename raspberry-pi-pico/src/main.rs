#![no_std]
#![no_main]
#![cfg(target_arch = "arm")]
use embedded_hal_bus::i2c::RefCellDevice;
use program::abstract_device::{AbstractDevice, Inputs};
use core::cell::RefCell;
use cortex_m::delay::Delay;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::{InputPin, OutputPin};
#[cfg(not(target_arch = "x86_64"))]
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use crate::error::Error;
use bsp::entry;
use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};
use cortex_m::prelude::_embedded_hal_adc_OneShot;
use rp2040_hal::{
    adc::AdcPin,
    fugit::RateExtU32,
    uart::{DataBits, StopBits, UartConfig, UartPeripheral},
    Adc, I2C,
};
use ssd1306::mode::{BufferedGraphicsMode, DisplayConfig};
use ssd1306::prelude::{DisplayRotation, DisplaySize128x32, I2CInterface};
use ssd1306::Ssd1306;

mod error;

#[entry]
#[allow(unreachable_code)]
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
    let button1_pin = pins.gpio7.into_pull_up_input();
    let button2_pin = pins.gpio8.into_pull_up_input();
    let mut adc = Adc::new(pac.ADC, &mut pac.RESETS);
    let mut adc_pin_0 = AdcPin::new(pins.gpio28.into_floating_input()).unwrap();
    let seed: u16 = adc.read(&mut adc_pin_0).unwrap();

    let result = (|| -> Result<(), Error> {
        let i2c = I2C::i2c0(
            pac.I2C0,
            pins.gpio20.reconfigure(), // sda
            pins.gpio21.reconfigure(), // scl
            400.kHz(),
            &mut pac.RESETS,
            125_000_000.Hz(),
        );
        let i2c_ref_cell = RefCell::new(i2c);
        let interface = ssd1306::I2CDisplayInterface::new(RefCellDevice::new(&i2c_ref_cell));
        let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
        display.init()?;

        let device = Device {
            display_storage: display,
            button1_pin,
            button2_pin,
            led_pin: &mut led_pin,
            delay: &mut delay,
            seed,
        };
        let mut game = program::game::Game::new(device)?;
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

struct Device<'a, I2C, Button1Pin, Button2Pin, LedPin> {
    display_storage:
        Ssd1306<I2CInterface<I2C>, DisplaySize128x32, BufferedGraphicsMode<DisplaySize128x32>>,
    button1_pin: Button1Pin,
    button2_pin: Button2Pin,
    led_pin: &'a mut LedPin,
    delay: &'a mut Delay,
    seed: u16,
}

impl<'a, I2C, Button1Pin: InputPin, Button2Pin: InputPin, LedPin: OutputPin> AbstractDevice
    for Device<'a, I2C, Button1Pin, Button2Pin, LedPin>
where
    I2C: embedded_hal::i2c::I2c,
{
    type Display =
        Ssd1306<I2CInterface<I2C>, DisplaySize128x32, BufferedGraphicsMode<DisplaySize128x32>>;
    type Error = Error;
    fn get_inputs(&mut self) -> Result<Inputs, Self::Error> {
        Ok(Inputs {
            button1_down: self.button1_pin.is_low().unwrap(),
            button2_down: self.button2_pin.is_low().unwrap(),
        })
    }
    fn set_led(&mut self, new_state: bool) {
        if new_state {
            self.led_pin.set_high().unwrap();
        } else {
            self.led_pin.set_low().unwrap();
        }
    }
    fn delay_ms(&mut self, ms: u32) {
        self.delay.delay_ms(ms);
    }
    fn get_rng_seed(&mut self) -> u64 {
        self.seed as u64
    }

    fn display(&mut self) -> &mut Self::Display {
        &mut self.display_storage
    }
    fn flush_display(&mut self) -> Result<(), Self::Error> {
        self.display_storage.flush()?;
        Ok(())
    }
}
