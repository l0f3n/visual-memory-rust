use crate::debouncing::{DebounceResult, Debouncer};
use crate::error::Error;
use core::mem::MaybeUninit;
use defmt::*;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::ascii::*;
use embedded_graphics::mono_font::{MonoTextStyle, MonoTextStyleBuilder};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::{Primitive, Size};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::text::{Baseline, Text};
use embedded_graphics::Drawable;
use fixed_slice_vec::FixedSliceVec;

const FONT_WIDTH: u32 = 6;
const FONT_HEIGHT: u32 = 10;
const BLOCK_GROUPING_EXTRA_SPACING: u32 = 2;
const BLOCK_LINE_HEIGHT: u32 = 2;
const BLOCK_SPACE: u32 = 2;

#[derive(Format, PartialEq, Clone, Copy)]
enum GameState {
    Menu,
    Displaying,
    Inputting,
    Next,
    Failure,
    Score,
}
pub trait ValidDisplay: DrawTarget<Color = BinaryColor> {
    fn flush(&mut self) -> Result<(), Error>;
}

pub struct Game<
    'a,
    FuncGetButton1: FnMut() -> bool,
    FuncGetButton2: FnMut() -> bool,
    FuncSetLed: FnMut(bool),
    FuncDelayMs: FnMut(u32),
    Display: ValidDisplay,
> {
    get_button_1: FuncGetButton1,
    get_button_2: FuncGetButton2,
    set_led: FuncSetLed,
    delay_ms: FuncDelayMs,
    display: Display,
    text_style: MonoTextStyle<'a, BinaryColor>,
    rng: fastrand::Rng,
    cursor: Point,
    screen_size: Size,
}

impl<
        'a,
        FuncGetButton1: FnMut() -> bool,
        FuncGetButton2: FnMut() -> bool,
        FuncSetLed: FnMut(bool),
        FuncDelayMs: FnMut(u32),
        Display: ValidDisplay,
    > Game<'a, FuncGetButton1, FuncGetButton2, FuncSetLed, FuncDelayMs, Display>
where
    Error: From<<Display as DrawTarget>::Error>,
{
    pub fn new(
        get_button_1: FuncGetButton1,
        get_button_2: FuncGetButton2,
        set_led: FuncSetLed,
        delay_ms: FuncDelayMs,
        display: Display,
        seed: u64,
    ) -> Result<Self, Error> {
        let rng = fastrand::Rng::with_seed(seed);
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();
        let screen_size = display.bounding_box().size;
        Ok(Self {
            get_button_1,
            get_button_2,
            set_led,
            delay_ms,
            display,
            text_style,
            rng,
            cursor: Point::zero(),
            screen_size,
        })
    }
    pub fn run_game(&mut self) -> Result<(), Error> {
        let mut debouncer_storage = [0x00u8; 2];
        let mut debounce = Debouncer::new(&mut debouncer_storage);

        const MAX_SEQUENCE: usize = 128;
        let mut sequence_storage = [MaybeUninit::new(false); MAX_SEQUENCE];
        let mut sequence = FixedSliceVec::new(&mut sequence_storage[..]);
        sequence.clear();
        let mut game_state = GameState::Menu;
        let mut last_game_state = GameState::Score;
        let mut next_guess_index = 0;
        let mut highest_cleared = 0;
        let mut first = true;

        loop {
            let button1_down = (self.get_button_1)();
            let button2_down = (self.get_button_2)();
            let button1_fell = debounce.update(0, button1_down) == DebounceResult::Pressed;
            let button2_fell = debounce.update(1, button2_down) == DebounceResult::Pressed;

            self.display.clear(BinaryColor::Off)?;
            self.reset_cursor();

            if last_game_state != game_state {
                last_game_state = game_state;
                info!("New state: {}", game_state);
            }
            match game_state {
                GameState::Menu => {
                    self.draw_string(
                        "Sequence memory! Try\nbuttons. Push both\nbuttons to start.",
                    )?;
                    if button1_down && button2_down {
                        game_state = GameState::Displaying;
                        next_guess_index = 0;
                        highest_cleared = 0;
                        self.set_starting_sequence(&mut sequence);
                        first = true;
                    } else {
                        if button1_fell {
                            sequence.push(false);
                        } else if button2_fell {
                            sequence.push(true);
                        }
                        self.draw_sequence(&sequence, sequence.len())?;
                    }
                }
                GameState::Displaying => {
                    if first {
                        self.display_temporary_message("Remember!", 1000)?;
                    }
                    self.draw_string(": ")?;
                    self.draw_sequence(&sequence, sequence.len())?;
                    (self.set_led)(true);
                    self.display.flush()?;
                    (self.set_led)(false);
                    if sequence.len() > 6 {
                        (self.delay_ms)(2000 + 200 * (sequence.len() as u32 - 6));
                    } else {
                        (self.delay_ms)(2000);
                    }
                    game_state = GameState::Inputting;
                    if first {
                        self.display_temporary_message("Repeat!", 1000)?;
                    }
                }
                GameState::Inputting => {
                    self.draw_string(": ")?;
                    if button1_fell {
                        if sequence[next_guess_index] == false {
                            next_guess_index += 1;
                        } else {
                            game_state = GameState::Failure;
                        }
                    } else if button2_fell {
                        if sequence[next_guess_index] == true {
                            next_guess_index += 1;
                        } else {
                            game_state = GameState::Failure;
                        }
                    }
                    self.draw_sequence(&sequence, next_guess_index)?;
                    if next_guess_index == sequence.len() {
                        game_state = GameState::Next;
                    }
                }
                GameState::Next => {
                    if first {
                        self.display_temporary_message("Good!", 1000)?;
                        self.display_temporary_message("Next sequence:", 1000)?;
                    } else {
                        self.display_temporary_message("Good! Next:", 400)?;
                    }
                    next_guess_index = 0;
                    highest_cleared = sequence.len();
                    game_state = GameState::Displaying;
                    first = false;
                    self.generate_sequence(sequence.len() + 1, &mut sequence);
                }
                GameState::Failure => {
                    self.display_temporary_message("No!", 200)?;
                    self.draw_string(": ")?;
                    self.draw_sequence(&sequence, sequence.len())?;
                    (self.set_led)(true);
                    self.display.flush()?;
                    (self.set_led)(false);
                    (self.delay_ms)(2000);
                    game_state = GameState::Score;
                }
                GameState::Score => {
                    self.draw_string("You cleared ")?;
                    let score =
                        highest_cleared as f32 + next_guess_index as f32 / sequence.len() as f32;
                    self.draw_float_string(score)?;
                    self.cursor = Point::new(0, 10);
                    self.draw_string("sequences!")?;
                    if button1_fell || button2_fell {
                        game_state = GameState::Menu;
                        sequence.clear();
                    }
                }
            }
            (self.set_led)(true);
            self.display.flush()?;
            (self.set_led)(false);
        }
    }

    fn set_starting_sequence(&self, sequence: &mut FixedSliceVec<bool>) {
        sequence.clear();
        sequence.push(false);
        sequence.push(false);
        sequence.push(true);
    }
    fn generate_sequence(&mut self, length: usize, sequence: &mut FixedSliceVec<bool>) {
        sequence.clear();
        for _ in 0..length {
            sequence.push(self.rng.bool());
        }
    }

    fn reset_cursor(&mut self) {
        self.cursor = Point::zero();
    }
    fn draw_string(&mut self, string: &str) -> Result<(), Error> {
        let next_point = Text::with_baseline(string, self.cursor, self.text_style, Baseline::Top)
            .draw(&mut self.display)?;
        self.cursor = next_point;
        Ok(())
    }

    fn draw_sequence(
        &mut self,
        sequence: &FixedSliceVec<bool>,
        subset_length: usize,
    ) -> Result<(), Error> {
        for i in 0..subset_length {
            let value = sequence[i];
            if i % 3 == 0 {
                // Create a grouping that's easier to parse when facing long sequences
                self.cursor.x += BLOCK_GROUPING_EXTRA_SPACING as i32;
            }
            self.draw_block_wrapping(value)?;
        }
        Ok(())
    }

    fn _draw_string_wrapping(&mut self, string: &str) -> Result<(), Error> {
        if self.cursor.x as u32 > self.screen_size.width - (FONT_WIDTH * string.len() as u32) {
            self.cursor.x = 0;
            self.cursor.y += FONT_HEIGHT as i32;
        }
        self.draw_string(string)?;
        Ok(())
    }

    fn draw_block_wrapping(&mut self, value: bool) -> Result<(), Error> {
        if self.cursor.x as u32 > self.screen_size.width - FONT_WIDTH {
            self.cursor.x = 0;
            self.cursor.y += FONT_HEIGHT as i32;
        }
        let block = if value {
            Rectangle::new(self.cursor, Size::new(FONT_WIDTH, FONT_HEIGHT - 1))
        } else {
            Rectangle::new(
                Point::new(
                    self.cursor.x,
                    self.cursor.y + FONT_HEIGHT as i32 - BLOCK_LINE_HEIGHT as i32 - 1,
                ),
                Size::new(FONT_WIDTH, BLOCK_LINE_HEIGHT),
            )
        };
        block
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
            .draw(&mut self.display)?;
        self.cursor.x += FONT_WIDTH as i32 + BLOCK_SPACE as i32;
        Ok(())
    }

    fn display_temporary_message(&mut self, string: &str, ms: u32) -> Result<(), Error> {
        self.display.clear(BinaryColor::Off)?;
        self.reset_cursor();
        self.draw_string(string)?;
        (self.set_led)(true);
        self.display.flush()?;
        (self.set_led)(false);
        (self.delay_ms)(ms);
        self.display.clear(BinaryColor::Off)?;
        self.reset_cursor();
        Ok(())
    }

    fn draw_float_string(&mut self, value: f32) -> Result<(), Error> {
        let mut buffer = [0x00u8; 12];
        let string = format_no_std::show(&mut buffer, format_args!("{:0.1}", value))?;
        self.draw_string(string)?;
        Ok(())
    }
}
