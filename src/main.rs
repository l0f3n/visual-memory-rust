//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use core::cell::RefCell;
use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::OutputPin;
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
use nano_rust_drivers::{error, ssd1306, ssd1306_registers};
use nano_rust_drivers::ssd1306_registers::{BLACK, WHITE};
use rp2040_hal::fugit::RateExtU32;
use rp2040_hal::I2C;
use rp2040_hal::uart::{DataBits, StopBits, UartConfig, UartPeripheral};

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
    let uart_pins = (
        pins.gpio0.into_function(),
        pins.gpio1.into_function(),
    );
    let uart = UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(115200.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        ).unwrap();

    uart.write_full_blocking(b"Hello World!\r\n");
    // info!("Startup");

    // This is the correct pin on the Raspberry Pico board. On other boards, even if they have an
    // on-board LED, it might need to be changed.
    //
    // Notably, on the Pico W, the LED is not connected to any of the RP2040 GPIOs but to the cyw43 module instead.
    // One way to do that is by using [embassy](https://github.com/embassy-rs/embassy/blob/main/examples/rp/src/bin/wifi_blinky.rs)
    //
    // If you have a Pico W and want to toggle a LED with a simple GPIO output pin, you can connect an external
    // LED to one of the GPIO pins, and reference that pin here. Don't forget adding an appropriate resistor
    // in series with the LED.
    let mut led_pin = pins.led.into_push_pull_output();


    // let dp = Peripherals::take().unwrap();
    // let pins = arduino_hal::pins!(dp);
    // let mut serial = arduino_hal::default_serial!(dp, pins, 115200);
    // print::put_console(serial);

    // let result = (|| -> Result<(), error::UDisplayError<arduino_hal::i2c::Error>> {
    //     let mut i2c = arduino_hal::I2c::new(
    //         dp.TWI,
    //         pins.a4.into_pull_up_input(),
    //         pins.a5.into_pull_up_input(),
    //         50000,
    //     );
    //     let mut led = pins.d13.into_output();
    //     let mut button1 = pins.d5.into_pull_up_input();
    //     let mut button2 = pins.d4.into_pull_up_input();
    //     let mut button1_last_pressed = button1.is_low();
    //     let mut button2_last_pressed = button2.is_low();
    //     let i2c_ref_cell = RefCell::new(i2c);
    //     let mut accelerometer = bmi160::Driver::new(embedded_hal_bus::i2c::RefCellDevice::new(&i2c_ref_cell), None, None)?;
    //     let mut buffer = [0x00; ssd1306::BUFFER_SIZE];
    //     let display_result = ssd1306::DisplayDriver::new(embedded_hal_bus::i2c::RefCellDevice::new(&i2c_ref_cell), None, &mut buffer);
    //     // print::print_type_name(&display_result);
    //     //
    //     // core::result::Result<
    //     // hackathon_pong_controller::ssd1306::DisplayDriver<embedded_hal_bus::i2c::refcell::RefCellDevice<avr_hal_generic::i2c::I2c<atmega_hal::Atmega,
    //     // avr_device::devices::atmega328p::TWI,
    //     // avr_hal_generic::port::Pin<avr_hal_generic::port::mode::Input,
    //     // atmega_hal::port::PC4>,
    //     // avr_hal_generic::port::Pin<avr_hal_generic::port::mode::Input,
    //     // atmega_hal::port::PC5>,
    //     // avr_hal_generic::clock::MHz16>>>,
    //     // hackathon_pong_controller::ssd1306_error::Error<avr_hal_generic::i2c::Error>>
    //     let mut display = match display_result {
    //         Ok(display) => {
    //             display
    //         }
    //         Err(error) => {
    //             return Err(error.into());
    //         }
    //     };
    //
    //     // println!("BMI160 initialized");
    //     info!("Started");

    // loop {
    //     // let button1_pressed = button1.is_low();
    //     // let button2_pressed = button2.is_low();
    //     // if button1_pressed && !button1_last_pressed {
    //     //     println!("Button1 pressed");
    //     // }
    //     // if button2_pressed && !button2_last_pressed {
    //     //     println!("Button2 pressed");
    //     // }
    //     // button1_last_pressed = button1_pressed;
    //     // button2_last_pressed = button2_pressed;
    //     //
    //     // led.toggle();
    //     // accelerometer.update()?;
    //     // if let Some(output_data) = accelerometer.get_output_data() {
    //     // }
    //     println!("On");
    //     display.fill_screen(ssd1306_registers::WHITE);
    //     display.display()?;
    //     led.set_high();
    //     arduino_hal::delay_ms(2000);
    //
    //     println!("Off");
    //     display.fill_screen(ssd1306_registers::BLACK);
    //     display.display()?;
    //     led.set_low();
    //     arduino_hal::delay_ms(2000);
    // }
    let result = (|| -> Result<(), error::UDisplayError<bsp::hal::i2c::Error>> {
        let mut i2c = I2C::i2c0(
            pac.I2C0,
            pins.gpio20.reconfigure(), // sda
            pins.gpio21.reconfigure(), // scl
            50.kHz(),
            &mut pac.RESETS,
            125_000_000.Hz(),
        );
        let i2c_ref_cell = RefCell::new(i2c);
        let mut display_buffer = [0x00; ssd1306::BUFFER_SIZE];
        let display_result = ssd1306::DisplayDriver::new(embedded_hal_bus::i2c::RefCellDevice::new(&i2c_ref_cell), None, &mut display_buffer);
        let mut display = match display_result {
            Ok(display) => {
                display
            }
            Err(error) => {
                return Err(error.into());
            }
        };

        loop {
            // delay.delay_ms(1000);
            // display.fill_screen(ssd1306_registers::BLACK);
            // display.display()?;
            // led_pin.set_high().unwrap();
            // delay.delay_ms(1000);
            //
            // display.fill_screen(ssd1306_registers::WHITE);
            // display.display()?;
            // led_pin.set_low().unwrap();
            // delay.delay_ms(1000);
            display.fill_screen(ssd1306_registers::BLACK);
            display.display()?;
            delay.delay_ms(1000);
            display.fill_screen(ssd1306_registers::WHITE);
            led_pin.set_high().unwrap();
            for i in 0..=ssd1306::BUFFER_SIZE {
                led_pin.set_high().unwrap();
                delay.delay_ms(20);
                led_pin.set_low().unwrap();
                delay.delay_ms(20);
                display.display_num(i)?;
            }
            led_pin.set_low().unwrap();
            delay.delay_ms(1000);
            // display.display_num(10)?;
            // delay.delay_ms(1000);

            // display.fill_screen(BLACK);
            // display.display()?;
            // delay.delay_ms(1000);
            // display.fill_screen(WHITE);
            // display.display()?;
            // delay.delay_ms(1000);
            // display.fill_screen(ssd1306_registers::BLACK);
            // for i in 0..32 {
            //     display.draw_pixel(i, i, WHITE)?;
            // }
            // for i in 0..32 {
            //     display.draw_pixel(31 + i, 31, WHITE)?;
            // }
            // for i in 0..32 {
            //     display.draw_pixel(63 + i, 31 - i, WHITE)?;
            // }
            // for i in 0..32 {
            //     display.draw_pixel(95 + i, 0, WHITE)?;
            // }
            // display.display()?;
            // delay.delay_ms(1000);
            // display.draw_line(4, 4, 57, 25, WHITE)?;
            // display.display()?;
            // delay.delay_ms(1000);
            // display.draw_line(57, 4, 4, 25, WHITE)?;
            // display.display()?;
            // delay.delay_ms(1000);
            // display.draw_line(97, 15, 44, 30, WHITE)?;
            // display.display()?;
            // delay.delay_ms(1000);
            // display.draw_line(44, 15, 97, 30, WHITE)?;
            // display.display()?;
            // delay.delay_ms(1000);
            // display.draw_fill_rect(4, 4, 57 - 4, 25 - 4, WHITE)?;
            // display.display()?;
            // // display.dim(true)?;
            // delay.delay_ms(1000);
            // // display.dim(false)?;
            // delay.delay_ms(1000);
            //
            // display.clear_display();
            // display.set_cursor(0, 0);
            // // display.draw_string("You're a mean one,\nMister Grinch!");
            // display.draw_string(" Abc");
            // display.display()?;
            // delay.delay_ms(2000);
        }
    })();
    if let Err(error) = result {
        info!("Error");
        loop {
            // uart.write_full_blocking(b"on!\n");
            // info!("on!");
            led_pin.set_high().unwrap();
            delay.delay_ms(100);
            // uart.write_full_blocking(b"off!\n");
            // info!("off!");
            led_pin.set_low().unwrap();
            delay.delay_ms(100);
        }
        // info!("Error: {}", error);
    }
    loop {
        info!("on!");
        led_pin.set_high().unwrap();
        delay.delay_ms(200);
        info!("off!");
        led_pin.set_low().unwrap();
        delay.delay_ms(200);
    }
}

// End of file
