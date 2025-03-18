#[cfg(not(target_arch = "x86_64"))]
use defmt::Format;

#[cfg_attr(not(target_arch = "x86_64"), derive(Format))]
pub enum Error {
    #[cfg(target_arch = "arm")]
    I2c(rp2040_hal::i2c::Error),
    #[cfg(not(target_arch = "x86_64"))]
    Display(display_interface::DisplayError),
    Format,
}
#[cfg(not(target_arch = "x86_64"))]
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

impl From<core::fmt::Error> for Error {
    fn from(_value: core::fmt::Error) -> Self {
        Self::Format
    }
}

