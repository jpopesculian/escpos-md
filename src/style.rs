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
    pub margin_top: usize,
    pub margin_bottom: usize,
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
            margin_top: 80,
            margin_bottom: 0,
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
    pub margin_top: Option<usize>,
    pub margin_bottom: Option<usize>,
}

impl Style {
    pub fn apply(&mut self, style: &RelativeStyle) {
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
            line_spacing,
            margin_top,
            margin_bottom
        );
        self.char_magnification = CharMagnification::clamped(
            style
                .font_width
                .unwrap_or_else(|| self.char_magnification.width()),
            style
                .font_height
                .unwrap_or_else(|| self.char_magnification.height()),
        );
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum StyleTag {
    P,
    H1,
    H2,
    H3,
    H4,
    H5,
    Blockquote,
    Code,
    Codeblock,
    Ul,
    Ol,
    Li,
    Em,
    Strong,
    Strikethrough,
    A,
    Img,
    ImgCaption,
}

impl StyleTag {
    pub fn display(&self) -> Display {
        match self {
            Self::P => Display::Block,
            Self::H1 => Display::Block,
            Self::H2 => Display::Block,
            Self::H3 => Display::Block,
            Self::H4 => Display::Block,
            Self::H5 => Display::Block,
            Self::Blockquote => Display::Block,
            Self::Code => Display::Inline,
            Self::Codeblock => Display::Block,
            Self::Ul => Display::Block,
            Self::Ol => Display::Block,
            Self::Li => Display::Block,
            Self::Em => Display::Inline,
            Self::Strong => Display::Inline,
            Self::Strikethrough => Display::Inline,
            Self::A => Display::Inline,
            Self::Img => Display::Block,
            Self::ImgCaption => Display::Block,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ListStyle {
    pub left_margin: usize,
    pub symbols: Vec<String>,
}

impl ListStyle {
    pub fn get_symbol(&self, depth: usize) -> &str {
        &self.symbols[depth % self.symbols.len()]
    }

    pub fn get_left_margin(&self, depth: usize) -> usize {
        self.left_margin * depth
    }
}

impl Default for ListStyle {
    fn default() -> Self {
        ListStyle {
            left_margin: 28,
            symbols: vec!["*".into()],
        }
    }
}

#[derive(Debug, Clone)]
pub struct StyleSheet {
    pub root: Style,
    pub list_style: ListStyle,
    tag_styles: [RelativeStyle; 17],
}

impl StyleSheet {
    pub fn new(root: Style, list_style: ListStyle) -> Self {
        Self {
            root,
            list_style,
            tag_styles: Default::default(),
        }
    }
    pub fn get_tag(&self, tag: &StyleTag) -> &RelativeStyle {
        &self.tag_styles[*tag as usize]
    }
    pub fn set_tag(&mut self, tag: &StyleTag, style: RelativeStyle) {
        self.tag_styles[*tag as usize] = style;
    }

    pub fn calc_tags<'a, I>(&self, tags: I) -> Style
    where
        I: Iterator<Item = &'a StyleTag>,
    {
        let mut style = self.root.clone();
        for tag in tags {
            style.apply(self.get_tag(tag));
        }
        style
    }
}

impl Default for StyleSheet {
    fn default() -> Self {
        let mut this = Self::new(Style::default(), ListStyle::default());
        this.set_tag(
            &StyleTag::H1,
            RelativeStyle {
                font_width: Some(3),
                font_height: Some(3),
                bold: Some(true),
                ..Default::default()
            },
        );
        this.set_tag(
            &StyleTag::Li,
            RelativeStyle {
                margin_top: Some(10),
                ..Default::default()
            },
        );
        this.set_tag(
            &StyleTag::Strong,
            RelativeStyle {
                bold: Some(true),
                ..Default::default()
            },
        );
        this.set_tag(
            &StyleTag::Em,
            RelativeStyle {
                underline: Some(UnderlineThickness::OneDot),
                ..Default::default()
            },
        );
        this
    }
}
