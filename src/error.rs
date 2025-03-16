// use ufmt::{Formatter, uDisplay, uWrite};
use defmt::Format;
// use crate::{bmi160_error, ssd1306_error};

#[derive(Format)]
pub enum UDisplayError<I2CError> {
    // BMI160Error(bmi160_error::Error<I2CError>),
    // SSD1306Error(ssd1306_error::Error<I2CError>),
    // PostcardError(postcard::Error),
    Marker(core::marker::PhantomData<I2CError>)
}

// #[cfg(feature = "string-errors")]
// impl<I2CError> uDisplay for UDisplayError<I2CError>
//     where I2CError: embedded_hal::i2c::Error {
//     fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error> where W: uWrite + ?Sized {
//         match self {
//             UDisplayError::BMI160Error(error) => {
//                 f.write_str("BMI160 error: ")?;
//                 error.fmt(f)
//             }
//             UDisplayError::SSD1306Error(error) => {
//                 f.write_str("BMI160 error: ")?;
//                 error.fmt(f)
//             }
//             UDisplayError::PostcardError(error) => {
//                 f.write_str("Postcard serialization error.")
//             }
//         }
//     }
// }
//
// impl<I2CError> From<bmi160_error::Error<I2CError>> for UDisplayError<I2CError> {
//     fn from(value: bmi160_error::Error<I2CError>) -> Self {
//         Self::BMI160Error(value)
//     }
// }
// impl<I2CError> From<ssd1306_error::Error<I2CError>> for UDisplayError<I2CError> {
//     fn from(value: ssd1306_error::Error<I2CError>) -> Self {
//         Self::SSD1306Error(value)
//     }
// }
// impl<I2CError> From<postcard::Error> for UDisplayError<I2CError> {
//     fn from(value: postcard::Error) -> Self {
//         Self::PostcardError(value)
//     }
// }
//
