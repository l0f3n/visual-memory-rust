#![cfg_attr(not(target_arch = "x86_64"), no_std)]
#![cfg_attr(not(target_arch = "x86_64"), no_main)]

mod abstract_device;
mod debouncing;
mod error;
mod game;
mod main_rp2040;
mod main_desktop;
mod main_arduino;

#[cfg(target_arch = "arm")]
use bsp::entry;
#[cfg(target_arch = "arm")]
use defmt::*;
#[cfg(target_arch = "arm")]
use defmt_rtt as _;
#[cfg(target_arch = "arm")]
use panic_probe as _;
#[cfg(target_arch = "arm")]
use rp_pico as bsp;

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
use crate::error::Error;

#[cfg(target_arch = "x86_64")]
fn main() -> Result<(), Error> {
    main_desktop::main_desktop()
}

#[cfg(target_arch = "avr")]
use panic_halt as _;

#[cfg(target_arch = "avr")]
fn get_type_name<T>(_: T) -> &'static str {
    nostd::any::type_name::<T>()
}

#[cfg(target_arch = "avr")]
#[arduino_hal::entry]
fn main() -> ! {
    main_arduino::main_arduino()
}

