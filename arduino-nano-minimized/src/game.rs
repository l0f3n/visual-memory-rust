use crate::abstract_device::AbstractDevice;
use crate::debouncing::{DebounceResult, Debouncer};
use core::mem::MaybeUninit;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Point;
use embedded_graphics::image::{ImageDrawable, ImageRaw, SubImage};
use embedded_graphics::mono_font::ascii::*;
use embedded_graphics::mono_font::{MonoFont, MonoTextStyle, MonoTextStyleBuilder};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::{Dimensions, OriginDimensions, Primitive, Size, Transform};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::text::renderer::TextRenderer;
use embedded_graphics::text::{Alignment, Baseline, Text};
use embedded_graphics::Drawable;
use fixed_slice_vec::FixedSliceVec;

const FONT_WIDTH: u32 = 6;
const FONT_HEIGHT: u32 = 10;
const BLOCK_GROUPING_EXTRA_SPACING: u32 = 2;
const BLOCK_LINE_HEIGHT: u32 = 2;
const BLOCK_SPACE: u32 = 2;

// #[derive(Format, PartialEq, Clone, Copy)]
#[derive(PartialEq, Clone, Copy)]
enum GameState {
    Menu,
    Displaying,
    Inputting,
    Next,
    Failure,
    Score,
}

pub struct Game<'a, Device: AbstractDevice> {
    device: Device,
    text_style: MonoTextStyle<'a, BinaryColor>,
    rng: fastrand::Rng,
    cursor: Point,
    screen_size: Size,
}

impl<'a, Device: AbstractDevice> Game<'a, Device> {
    pub fn new(mut device: Device) -> Result<Self, Device::Error> {
        let rng = fastrand::Rng::with_seed(device.get_rng_seed());
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();
        let screen_size = device.display().bounding_box().size;
        Ok(Self {
            device,
            text_style,
            rng,
            cursor: Point::zero(),
            screen_size,
        })
    }
    pub fn run_game(&mut self) -> Result<(), Device::Error> {
        let mut debouncer_storage = [0x00u8; 2];
        let mut debounce = Debouncer::new(&mut debouncer_storage);

        const MAX_SEQUENCE: usize = 4096;
        // static BUFFER: [u8; 4096] = [0u8; 4096];
        // for byte in BUFFER {
        let mut sequence_storage: [MaybeUninit<bool>; MAX_SEQUENCE] =
            [MaybeUninit::new(false); MAX_SEQUENCE];
        let mut sequence = FixedSliceVec::new(&mut sequence_storage[..]);
        sequence.clear();
        self.device.display().clear(BinaryColor::Off)?;
        let mut game_state = GameState::Menu;
        let mut last_game_state = GameState::Score;
        let mut next_guess_index = 0;
        let mut highest_cleared = 0;
        let mut first = true;

        loop {
            let inputs = self.device.get_inputs()?;
            let button1_down = inputs.button1_down;
            let button2_down = inputs.button2_down;
            let button1_fell = debounce.update(0, button1_down) == DebounceResult::Pressed;
            let button2_fell = debounce.update(1, button2_down) == DebounceResult::Pressed;

            self.device.display().clear(BinaryColor::Off)?;
            self.reset_cursor();

            if last_game_state != game_state {
                last_game_state = game_state;
                // info!("New state: {}", game_state);
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
                    self.device.set_led(true);
                    self.device.flush_display()?;
                    self.device.set_led(false);
                    if sequence.len() > 6 {
                        self.device
                            .delay_ms(2000 + 200 * (sequence.len() as u32 - 6));
                    } else {
                        self.device.delay_ms(2000);
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
                    self.display_temporary_message("Good! Next:", 400)?;
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
                    self.device.set_led(true);
                    self.device.flush_display()?;
                    self.device.set_led(false);
                    self.device.delay_ms(2000);
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
            // self.device.delay_ms(10);
            self.device.set_led(true);
            self.device.flush_display()?;
            self.device.set_led(false);
            // loop {
            //     self.device.set_led(true);
            //     arduino_hal::delay_ms(200);
            //     self.device.set_led(false);
            //     arduino_hal::delay_ms(200);
            // }
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
    fn draw_string(&mut self, string: &str) -> Result<(), Device::Error> {
        let text = Text::with_baseline(string, self.cursor, self.text_style, Baseline::Top);

        let target = self.device.display();
        let mut next_position = text.position;

        let mut position = text.position;

        for line in text.text.split('\n') {
            let p = position;
            position.y += text
                .text_style
                .line_height
                .to_absolute(text.character_style.line_height()) as i32;

            // remove trailing '\r' for '\r\n' line endings
            let len = line.len();
            let (line, p) = if len > 0 && line.as_bytes()[len - 1] == b'\r' {
                (&line[0..len - 1], p)
            } else {
                (line, p)
            };

            let mut position = position - Point::new(0, 0);
            let char_width = self.text_style.font.character_size.width as i32;
            for next_char in line.chars() {
                let p = position;
                position.x += char_width;
                // let glyph = self.text_style.font.glyph(next_char);
                let glyph = Glyph::new(self.text_style.font, next_char);
                embedded_graphics::image::Image::new(&glyph, p).draw(target)?;
            }
            next_position = position;

            // let next = self.text_style.draw_string_binary(
            //     text,
            //     position,
            //     MonoFontDrawTarget::new(target, Foreground(text_color)),
            // )?;

            // next_position = self.text_style.draw_string(
            //     line,
            //     position,
            //     text.text_style.baseline,
            //     target,
            // )?;
        }

        self.cursor = next_position;
        Ok(())
    }

    fn draw_sequence(
        &mut self,
        sequence: &FixedSliceVec<bool>,
        subset_length: usize,
    ) -> Result<(), Device::Error> {
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

    fn _draw_string_wrapping(&mut self, string: &str) -> Result<(), Device::Error> {
        if self.cursor.x as u32 > self.screen_size.width - (FONT_WIDTH * string.len() as u32) {
            self.cursor.x = 0;
            self.cursor.y += FONT_HEIGHT as i32;
        }
        self.draw_string(string)?;
        Ok(())
    }

    fn draw_block_wrapping(&mut self, value: bool) -> Result<(), Device::Error> {
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
            .draw(self.device.display())?;
        self.cursor.x += FONT_WIDTH as i32 + BLOCK_SPACE as i32;
        Ok(())
    }

    fn display_temporary_message(&mut self, string: &str, ms: u32) -> Result<(), Device::Error> {
        self.device.display().clear(BinaryColor::Off)?;
        self.reset_cursor();
        self.draw_string(string)?;
        self.device.set_led(true);
        self.device.flush_display()?;
        self.device.set_led(false);
        self.device.delay_ms(ms);
        self.device.display().clear(BinaryColor::Off)?;
        self.reset_cursor();
        Ok(())
    }

    fn draw_float_string(&mut self, _value: f32) -> Result<(), Device::Error> {
        // let mut buffer = [0x00u8; 12];
        self.draw_string("X.X")?;
        Ok(())
    }
}

struct Glyph<'a> {
    parent: &'a ImageRaw<'a, BinaryColor>,
    area: Rectangle,
}

impl<'a> Glyph<'a> {
    pub fn new(font: &'a MonoFont<'a>, c: char) -> Self {
        if font.character_size.width == 0 || font.image.size().width < font.character_size.width {
            return Self::new_unchecked(&font.image, Rectangle::zero());
        }

        let glyphs_per_row = font.image.size().width / font.character_size.width;

        // Char _code_ offset from first char, most often a space
        // E.g. first char = ' ' (32), target char = '!' (33), offset = 33 - 32 = 1
        let glyph_index = font.glyph_mapping.index(c) as u32;
        let row = glyph_index / glyphs_per_row;

        // Top left corner of character, in pixels
        let char_x = (glyph_index - (row * glyphs_per_row)) * font.character_size.width;
        let char_y = row * font.character_size.height;

        Self::new_unchecked(
            &font.image,
            Rectangle::new(
                Point::new(char_x as i32, char_y as i32),
                font.character_size,
            ),
        )
    }

    // pub(super) fn new(parent: &'a T, area: &Rectangle) -> Self {
    //     let area = parent.bounding_box().intersection(area);
    //
    //     Self { parent, area }
    // }

    pub(crate) const fn new_unchecked(parent: &'a ImageRaw<BinaryColor>, area: Rectangle) -> Self {
        Self { parent, area }
    }
}

impl<'a> OriginDimensions for Glyph<'a> {
    fn size(&self) -> Size {
        todo!()
    }
}

// impl<'a> ImageDrawable for Glyph<'a> {
//     type Color = BinaryColor;
//
//     fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
//     where
//         D: DrawTarget<Color=Self::Color>
//     {
//         todo!()
//     }
//
//     fn draw_sub_image<D>(&self, target: &mut D, area: &Rectangle) -> Result<(), D::Error>
//     where
//         D: DrawTarget<Color=Self::Color>
//     {
//         todo!()
//     }
// }

impl<'a> ImageDrawable for Glyph<'a> {
    type Color = BinaryColor;

    fn draw<DT>(&self, target: &mut DT) -> Result<(), DT::Error>
    where
        DT: DrawTarget<Color = Self::Color>,
    {
        self.parent.draw_sub_image(target, &self.area)
    }

    fn draw_sub_image<DT>(&self, target: &mut DT, area: &Rectangle) -> Result<(), DT::Error>
    where
        DT: DrawTarget<Color = Self::Color>,
    {
        let area = area.translate(self.area.top_left);

        self.parent.draw_sub_image(target, &area)
    }
}
