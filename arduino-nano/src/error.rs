
pub enum Error {
    // Error inside the display transport layer
    Display(display_interface::DisplayError),
    // Format error
    Format,
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

