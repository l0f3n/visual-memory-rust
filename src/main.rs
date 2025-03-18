#![cfg_attr(not(target_arch = "x86_64"), no_std)]
#![cfg_attr(not(target_arch = "x86_64"), no_main)]

mod abstract_device;
mod debouncing;
mod error;
mod game;
mod main_rp2040;
mod main_desktop;

#[cfg(not(target_arch = "x86_64"))]
use bsp::entry;
#[cfg(not(target_arch = "x86_64"))]
use defmt::*;
#[cfg(not(target_arch = "x86_64"))]
use defmt_rtt as _;
#[cfg(not(target_arch = "x86_64"))]
use panic_probe as _;
#[cfg(not(target_arch = "x86_64"))]
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

