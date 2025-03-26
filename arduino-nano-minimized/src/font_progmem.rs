use avr_progmem::progmem;
use embedded_graphics::image::ImageRaw;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::geometry::Size;
use embedded_graphics::mono_font::{DecorationDimensions, MonoFont};
use embedded_graphics::mono_font::mapping::GlyphMapping;
use embedded_graphics::{geometry, image, mono_font};

#[derive(Clone, Copy)]
pub struct MonoFontProgmem<'a> {
    /// Raw image data containing the font.
    pub image: ImageRaw<'a, BinaryColor>,

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

pub static /*progmem*/ FONT_6X10: MonoFont = MonoFont {
    image: image::ImageRaw::new(
        include_bytes!("font_6x10.raw"),
        96,
    ),
    glyph_mapping: &mono_font::mapping::ASCII,
    character_size: geometry::Size::new(6, 10),
    character_spacing: 0,
    baseline: 7,
    underline: mono_font::DecorationDimensions::new(7 + 2, 1),
    strikethrough: mono_font::DecorationDimensions::new(10 / 2, 1),
};

// avr_progmem::progmem! {
//     static progmem FONT: [u8; 720] = include_bytes!("font_6x10.raw").clone();
// }
