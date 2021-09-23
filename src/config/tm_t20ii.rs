use super::PrinterConfig;
use crate::command::Font;

pub const TM_T20II_80MM_WIDTH: usize = 576;
pub const TM_T20II_58MM_WIDTH: usize = 420;
pub const TM_T20II_CHAR_SPACING: usize = 2;
pub const TM_T20II_FONTA_WIDTH: usize = 12;
pub const TM_T20II_FONTB_WIDTH: usize = 9;

impl PrinterConfig {
    pub fn tm_t20ii() -> Self {
        let mut this = PrinterConfig::default();
        this.width(TM_T20II_80MM_WIDTH)
            .char_spacing(TM_T20II_CHAR_SPACING)
            .font_width(&Font::FontA, TM_T20II_FONTA_WIDTH)
            .font_width(&Font::FontB, TM_T20II_FONTB_WIDTH);
        this
    }
}
