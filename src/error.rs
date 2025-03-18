use std::convert::Infallible;
use std::fmt::{Display, Formatter, Write};
#[cfg(not(target_arch = "x86_64"))]
use defmt::Format;

#[cfg_attr(not(target_arch = "x86_64"), derive(Format))]
#[cfg_attr(target_arch = "x86_64", derive(Debug))]
pub enum Error {
    #[cfg(target_arch = "arm")]
    I2c(rp2040_hal::i2c::Error),
    #[cfg(not(target_arch = "x86_64"))]
    Display(display_interface::DisplayError),
    #[cfg(target_arch = "x86_64")]
    Infallible,
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
impl From<Infallible> for Error {
    fn from(_value: Infallible) -> Self {
        Self::Infallible
    }
}

// impl Display for Error {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Error::Infallible => f.write_str("Infallible"),
//             Error::Format => f.write_str("Format"),
//         }
//     }
// }
