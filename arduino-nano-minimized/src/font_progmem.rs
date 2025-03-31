use core::fmt::Binary;
use avr_progmem::progmem;
use avr_progmem::wrapper::ProgMem;
use core::marker::PhantomData;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Dimensions, OriginDimensions, Point, Size};
use embedded_graphics::image::{ImageDrawable, ImageRaw};
use embedded_graphics::iterator::raw::RawDataSlice;
use embedded_graphics::mono_font::mapping::GlyphMapping;
use embedded_graphics::mono_font::{DecorationDimensions, MonoFont};
use embedded_graphics::pixelcolor::raw::{BigEndian, ByteOrder};
use embedded_graphics::pixelcolor::{BinaryColor, PixelColor};
use embedded_graphics::prelude::{RawData, Transform};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::{geometry, image, mono_font};

const FONT_BYTE_SIZE: usize = 720;
avr_progmem::progmem! {
    static progmem FONT: [u8; FONT_BYTE_SIZE] = *include_bytes!("font_6x10.raw");
}

pub static /*progmem*/ FONT_6X10: MonoFontProgmem = MonoFontProgmem {
    image: ImageProgmem::new(
        &FONT,
        96,
    ),
    glyph_mapping: &mono_font::mapping::ASCII,
    character_size: geometry::Size::new(6, 10),
    character_spacing: 0,
    baseline: 7,
    underline: mono_font::DecorationDimensions::new(7 + 2, 1),
    strikethrough: mono_font::DecorationDimensions::new(10 / 2, 1),
};

#[derive(Clone, Copy)]
pub struct MonoFontProgmem<'a> {
    /// Raw image data containing the font.
    pub image: ImageProgmem<'a>,

    /// Size of a single character in pixel.
    pub character_size: Size,

    /// Spacing between characters.
    ///
    /// The spacing defines how many empty pixels are added horizontally between adjacent characters
    /// on a single line of text.
    pub character_spacing: u32,

    /// The baseline.
    ///
    /// Offset from the top of the glyph bounding box to the baseline.
    pub baseline: u32,

    /// Strikethrough decoration dimensions.
    pub strikethrough: DecorationDimensions,

    /// Underline decoration dimensions.
    pub underline: DecorationDimensions,

    /// Glyph mapping.
    pub glyph_mapping: &'a dyn GlyphMapping,
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(::defmt::Format))]
pub struct ImageProgmem<'a, BO = BigEndian>
where
    BO: ByteOrder,
{
    /// Image data, packed as dictated by raw data type `C::Raw`
    data: Option<&'a ProgMem<[u8; FONT_BYTE_SIZE]>>,

    /// Image size in pixels
    size: Size,

    byte_order: PhantomData<BO>,
}
avr_progmem::progmem! {
    static progmem EMPTY_FONT: [u8; 1] = [0xA5];
}

impl<'a, BO> ImageProgmem<'a, BO>
where
    BO: ByteOrder,
{
    /// Creates a new image.
    ///
    /// Only the width of the image needs to be specified. The height of the image will be
    /// calculated based on the length of the given image data. If the length of the image data
    /// isn't an integer multiple of the data length for a single row the last partial row will
    /// be ignored.
    pub const fn new(data: &'a ProgMem<[u8; FONT_BYTE_SIZE]>, width: u32) -> Self {
        // Prevent panic for `width == 0` by returning a zero sized image.
        if width == 0 {
            return Self {
                data: None,
                size: Size::zero(),
                byte_order: PhantomData,
            };
        }

        let height = FONT_BYTE_SIZE / bytes_per_row(width, <BinaryColor as PixelColor>::Raw::BITS_PER_PIXEL);

        Self {
            data: Some(data),
            size: Size::new(width, height as u32),
            byte_order: PhantomData,
        }
    }

    /// Returns the actual row width in pixels.
    ///
    /// For images with less than 8 bits per pixel each row is padded to contain an integer number
    /// of bytes. This method returns the width of each row including the padding pixels.
    const fn data_width(&self) -> u32 {
        if <BinaryColor as PixelColor>::Raw::BITS_PER_PIXEL < 8 {
            let pixels_per_byte = 8 / <BinaryColor as PixelColor>::Raw::BITS_PER_PIXEL as u32;

            bytes_per_row(self.size.width, <BinaryColor as PixelColor>::Raw::BITS_PER_PIXEL) as u32 * pixels_per_byte
        } else {
            self.size.width
        }
    }
}

impl<BO> OriginDimensions for ImageProgmem<'_, BO>
where
    BO: ByteOrder,
{
    fn size(&self) -> Size {
        self.size
    }
}

impl<'a, BO> ImageDrawable for ImageProgmem<'a, BO>
where
    BO: ByteOrder,
    RawDataSlice<'a, <BinaryColor as PixelColor>::Raw, BO>: IntoIterator<Item = <BinaryColor as PixelColor>::Raw>,
{
    type Color = BinaryColor;

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        let row_skip = self.data_width() - self.size.width;

        if let Some(data) = self.data {
            target.fill_contiguous(
                &self.bounding_box(),
                ContiguousPixels::new(data, self.size, 0, row_skip as usize),
            )
        } else {
            // TODO choose a way in which to create an err here
            Ok(())
        }
    }

    fn draw_sub_image<D>(&self, target: &mut D, area: &Rectangle) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        // Don't draw anything if `area` is zero sized or partially outside the image.
        if area.is_zero_sized()
            || area.top_left.x < 0
            || area.top_left.y < 0
            || area.top_left.x as u32 + area.size.width > self.size.width
            || area.top_left.y as u32 + area.size.height > self.size.height
        {
            return Ok(());
        }

        let data_width = self.data_width() as usize;

        let initial_skip = area.top_left.y as usize * data_width + area.top_left.x as usize;
        let row_skip = data_width - area.size.width as usize;

        if let Some(data) = self.data {
            target.fill_contiguous(
                &Rectangle::new(Point::zero(), area.size),
                ContiguousPixels::new(data, area.size, initial_skip, row_skip),
            )
        } else {
            // TODO choose a way in which to create an err here
            Ok(())
        }
    }
}

/// Returns the length of each row in bytes.
const fn bytes_per_row(width: u32, bits_per_pixel: usize) -> usize {
    (width as usize * bits_per_pixel + 7) / 8
}

pub struct Glyph<'a> {
    parent: &'a ImageProgmem<'a>,
    area: Rectangle,
}

impl<'a> Glyph<'a> {
    pub fn new(font: &'a MonoFontProgmem<'a>, c: char) -> Self {
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

    pub(crate) const fn new_unchecked(
        parent: &'a ImageProgmem,
        area: Rectangle,
    ) -> Self {
        Self { parent, area }
    }
}

impl<'a> OriginDimensions for Glyph<'a> {
    fn size(&self) -> Size {
        self.area.size
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

const CHUNK_SIZE: usize = 8;
struct ContiguousPixels<'a, BO>
where
    // C: PixelColor + From<<C as PixelColor>::Raw>,
    BO: ByteOrder,
    RawDataSlice<'a, <BinaryColor as PixelColor>::Raw, BO>: IntoIterator<Item = <BinaryColor as PixelColor>::Raw>,
{
    iter: Option<<RawDataSlice<'a, <BinaryColor as PixelColor>::Raw, BO> as IntoIterator>::IntoIter>,

    remaining_x: u32,
    width: u32,

    remaining_y: u32,
    row_skip: usize,
    slice: [u8; CHUNK_SIZE],
    slice_pos: u32,
}

impl<'a, BO> ContiguousPixels<'a, BO>
where
    BO: ByteOrder,
    RawDataSlice<'a, <BinaryColor as PixelColor>::Raw, BO>: IntoIterator<Item = <BinaryColor as PixelColor>::Raw>,
{
    fn new(
        image: &ProgMem<[u8; FONT_BYTE_SIZE]>,
        size: Size,
        initial_skip: usize,
        row_skip: usize,
    ) -> Self {
        // let mut iter = RawDataSlice::new(image).into_iter();
        //
        // if initial_skip > 0 {
        //     iter.nth(initial_skip - 1);
        // }
        // TODO I haven't thought properly about this math yet. There's a modification of initial_skip that's necessary somehow
        let slice = if initial_skip * 8 + CHUNK_SIZE < FONT_BYTE_SIZE {
            image.load_sub_array::<CHUNK_SIZE>(initial_skip/8)
        } else {
            image.load_sub_array::<CHUNK_SIZE>(FONT_BYTE_SIZE - CHUNK_SIZE)
        };
        // let mut iter = RawDataSlice::new(&slice[..]).into_iter();

        // Set `remaining_y` to `0` if `width == 0` to prevent integer underflow in `next`.
        let remaining_y = if size.width > 0 { size.height } else { 0 };

        let contiguous = Self {
            slice,
            iter: None,
            remaining_x: size.width,
            width: size.width,
            remaining_y,
            row_skip,
            slice_pos: 0,
        };
        contiguous
    }
}

impl<'a, BO> Iterator for ContiguousPixels<'a, BO>
where
    BO: ByteOrder,
    RawDataSlice<'a, <BinaryColor as PixelColor>::Raw, BO>: IntoIterator<Item = <BinaryColor as PixelColor>::Raw>,
{
    type Item = BinaryColor;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_x > 0 {
            self.remaining_x -= 1;

            // self.iter.next()
            // self.slice_pos.
            self.slice_pos += 1;
        } else {
            if self.remaining_y == 0 {
                return None;
            }

            self.remaining_y -= 1;
            self.remaining_x = self.width - 1;

            // TODO this should load a new slice
            // self.iter.nth(self.row_skip)
        }
        if self.slice_pos % 2 == 0 {
            Some(BinaryColor::On)
        } else {
            Some(BinaryColor::Off)
        }
    }
}
