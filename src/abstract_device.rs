use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::DrawTarget;

#[derive(Copy, Clone, PartialEq)]
pub struct Inputs {
    pub button1_down: bool,
    pub button2_down: bool,
}

impl Inputs {
    pub(crate) fn default() -> Inputs {
        Inputs {
            button1_down: false,
            button2_down: false,
        }
    }
}

pub trait AbstractDevice {
    type Display: DrawTarget<Color=BinaryColor>;
    type Error: From<<Self::Display as DrawTarget>::Error> + From<core::fmt::Error>;
    fn get_inputs(&mut self) -> Result<Inputs, Self::Error>;
    fn set_led(&mut self, new_state: bool);
    fn delay_ms(&mut self, ms: u32);
    fn get_rng_seed(&mut self) -> u64;

    fn display(&mut self) -> &mut Self::Display;
    fn flush_display(&mut self) -> Result<(), Self::Error>;
}