pub mod default;
pub mod tm_t20ii;

use crate::command::Font;
use default::*;

#[derive(Clone, Debug)]
pub struct FontWidths {
    widths: [usize; 5],
}

impl Default for FontWidths {
    fn default() -> Self {
        FontWidths {
            widths: [
                DEFAULT_FONTA_WIDTH,
                DEFAULT_FONTB_WIDTH,
                DEFAULT_FONTC_WIDTH,
                DEFAULT_FONTD_WIDTH,
                DEFAULT_FONTE_WIDTH,
            ],
        }
    }
}

impl FontWidths {
    pub fn get(&self, font: &Font) -> usize {
        self.widths[*font as usize]
    }
    pub fn set(&mut self, font: &Font, width: usize) {
        self.widths[*font as usize] = width;
    }
}

#[derive(Clone, Debug)]
pub struct PrinterConfig {
    pub width: usize,
    pub char_spacing: usize,
    pub font_widths: FontWidths,
}

impl PrinterConfig {
    pub fn width(&mut self, width: usize) -> &mut Self {
        self.width = width;
        self
    }
    pub fn font_width(&mut self, font: &Font, width: usize) -> &mut Self {
        self.font_widths.set(font, width);
        self
    }
    pub fn char_spacing(&mut self, char_spacing: usize) -> &mut Self {
        self.char_spacing = char_spacing;
        self
    }
}

impl Default for PrinterConfig {
    fn default() -> Self {
        Self {
            width: DEFAULT_WIDTH,
            char_spacing: DEFAULT_CHAR_SPACING,
            font_widths: FontWidths::default(),
        }
    }
}
