use defmt::Format;

#[derive(Format)]
pub enum Error {
    I2c(rp2040_hal::i2c::Error),
    Display(display_interface::DisplayError),
    Format,
}
impl From<rp2040_hal::i2c::Error> for Error {
    fn from(value: rp2040_hal::i2c::Error) -> Self {
        Self::I2c(value)
    }
}

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

