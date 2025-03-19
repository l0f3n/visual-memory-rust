use std::convert::Infallible;

#[derive(Debug)]
pub enum Error {
    // Never instantiated, just exists for type system reasons
    Infallible,
    // Format error
    Format,
    // Exit out of the application
    Quit,
}

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

