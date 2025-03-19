#[cfg(target_arch = "x86_64")]
use std::convert::Infallible;
#[cfg(target_arch = "arm")]
use defmt::Format;

#[cfg_attr(target_arch = "arm", derive(Format))]
#[cfg_attr(target_arch = "x86_64", derive(Debug))]
pub enum Error {
    // Error on the I2C bus
    #[cfg(target_arch = "arm")]
    I2c(rp2040_hal::i2c::Error),
    // Error inside the display transport layer
    #[cfg(not(target_arch = "x86_64"))]
    Display(display_interface::DisplayError),
    // Never instantiated, just exists for type system reasons
    #[cfg(target_arch = "x86_64")]
    Infallible,
    // Format error
    Format,
    // Exit out of the application
    #[cfg(target_arch = "x86_64")]
    Quit,
}
#[cfg(target_arch = "arm")]
impl From<rp2040_hal::i2c::Error> for Error {
    fn from(value: rp2040_hal::i2c::Error) -> Self {
        Self::I2c(value)
    }
}

#[cfg(not(target_arch = "x86_64"))]
impl From<display_interface::DisplayError> for Error {
    fn from(value: display_interface::DisplayError) -> Self {
        Self::Display(value)
    }
}

#[cfg(target_arch = "x86_64")]
impl From<Infallible> for Error {
    fn from(_value: Infallible) -> Self {
        Self::Infallible
    }
}

impl From<core::fmt::Error> for Error {
    fn from(_value: core::fmt::Error) -> Self {
        Self::Format
    }
}

