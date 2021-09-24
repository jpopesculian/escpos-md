use crate::command::{CharMagnification, Font, Justification, UnderlineThickness};
use crate::config::default::DEFAULT_CHAR_SPACING;
use crate::error::Result;
use crate::{Printer, PrinterDevice};

pub enum Display {
    Block,
    Inline,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Style {
    pub font: Font,
    pub char_magnification: CharMagnification,
    pub underline: UnderlineThickness,
    pub bold: bool,
    pub white_black_reverse: bool,
    pub split_words: bool,
    pub justification: Justification,
    pub char_spacing: usize,
    pub line_spacing: Option<usize>,
}

impl<D> Printer<D>
where
    D: PrinterDevice,
{
    pub fn style(&mut self, style: &Style, display: &Display) -> Result<&mut Self> {
        self.font(style.font)?
            .char_size(style.char_magnification)?
            .underline(style.underline)?
            .bold(style.bold)?
            .white_black_reverse(style.white_black_reverse)?
            .char_spacing(style.char_spacing)?;
        match display {
            Display::Block => {
                self.line_spacing(style.line_spacing)?
                    .split_words(style.split_words)?
                    .justification(style.justification)?;
            }
            _ => {}
        }
        Ok(self)
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            font: Font::default(),
            char_magnification: CharMagnification::default(),
            underline: UnderlineThickness::default(),
            bold: false,
            white_black_reverse: false,
            split_words: true,
            justification: Justification::default(),
            line_spacing: None,
            char_spacing: DEFAULT_CHAR_SPACING,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct RelativeStyle {
    pub font: Option<Font>,
    pub font_width: Option<u8>,
    pub font_height: Option<u8>,
    pub underline: Option<UnderlineThickness>,
    pub bold: Option<bool>,
    pub white_black_reverse: Option<bool>,
    pub split_words: Option<bool>,
    pub justification: Option<Justification>,
    pub char_spacing: Option<usize>,
    pub line_spacing: Option<Option<usize>>,
}

impl Style {
    pub fn apply(&mut self, style: &RelativeStyle) -> Result<()> {
        macro_rules! apply_field {
            ($($field:ident),*) => {
                $(
                    if let Some($field) = style.$field {
                        self.$field = $field;
                    }
                )*
            }
        }
        apply_field!(
            font,
            underline,
            bold,
            white_black_reverse,
            split_words,
            justification,
            char_spacing,
            line_spacing
        );
        self.char_magnification = CharMagnification::new(
            style
                .font_width
                .unwrap_or_else(|| self.char_magnification.width()),
            style
                .font_height
                .unwrap_or_else(|| self.char_magnification.height()),
        )?;
        Ok(())
    }
}
