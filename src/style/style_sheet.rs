use super::rule::{parse_rules, Rule};
use super::style_tag::StyleTag;
use crate::command::{CharMagnification, Font, Justification, UnderlineThickness};
use crate::config::default::DEFAULT_CHAR_SPACING;
use crate::error::Result;
use crate::{Printer, PrinterDevice};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Display {
    Inline,
    Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Style {
    pub display: Display,
    // Font styles
    pub bold: bool,
    pub char_magnification: CharMagnification,
    pub char_spacing: usize,
    pub font: Font,
    pub line_spacing: Option<usize>,
    pub split_words: bool,
    pub underline: UnderlineThickness,
    pub white_black_reverse: bool,
    // block styles
    pub prefix: String,
    pub justification: Justification,
    pub margin_left: usize,
    pub margin_bottom: usize,
    pub margin_top: usize,
}

impl<D> Printer<D>
where
    D: PrinterDevice,
{
    pub(crate) fn font_style(&mut self, style: &Style) -> Result<&mut Self> {
        self.bold(style.bold)?
            .char_size(style.char_magnification)?
            .char_spacing(style.char_spacing)?
            .font(style.font)?
            .line_spacing(style.line_spacing)?
            .split_words(style.split_words)?
            .underline(style.underline)?
            .white_black_reverse(style.white_black_reverse)?;
        Ok(self)
    }
    pub(crate) fn begin_block_style(&mut self, style: &Style) -> Result<&mut Self> {
        if matches!(style.display, Display::Block) {
            self.justification(style.justification)?
                .feed_paper(style.margin_top)?;
            if style.margin_left != 0 {
                let new_left_margin = self.state.left_margin + style.margin_left as u16;
                self.left_margin(new_left_margin)?;
            }
        }
        if !style.prefix.is_empty() {
            self.print(&style.prefix)?;
        }
        Ok(self)
    }

    pub(crate) fn end_block_style(&mut self, style: &Style) -> Result<&mut Self> {
        if matches!(style.display, Display::Block) {
            self.feed_paper(style.margin_bottom)?;
            if style.margin_left != 0 {
                let new_left_margin = self.state.left_margin - style.margin_left as u16;
                self.left_margin(new_left_margin)?;
            }
        }
        Ok(self)
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            display: Display::Block,
            prefix: String::default(),
            font: Font::default(),
            char_magnification: CharMagnification::default(),
            underline: UnderlineThickness::default(),
            bold: false,
            white_black_reverse: false,
            split_words: true,
            justification: Justification::default(),
            line_spacing: None,
            char_spacing: DEFAULT_CHAR_SPACING,
            margin_top: 0,
            margin_bottom: 0,
            margin_left: 0,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct RelativeStyle {
    pub display: Option<Display>,
    pub prefix: Option<String>,
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
    pub margin_left: Option<usize>,
}

macro_rules! apply_fields {
    ($src:ident -> $dst:ident: $($field:ident),*) => {
        $(
            if let Some(ref $field) = $src.$field {
                $dst.$field = $field.clone();
            }
        )*
    }
}

impl Style {
    pub fn apply_font(&mut self, style: &RelativeStyle) {
        apply_fields!(
            style -> self:
            font,
            underline,
            bold,
            white_black_reverse,
            split_words,
            char_spacing,
            line_spacing
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
    pub fn apply_block(&mut self, style: &RelativeStyle) {
        apply_fields!(
            style -> self:
            display,
            prefix,
            justification,
            margin_top,
            margin_bottom,
            margin_left
        );
    }
}
#[derive(Clone, Debug)]
pub struct StyleSheet {
    base: Style,
    rules: Vec<(Rule, RelativeStyle)>,
}

impl StyleSheet {
    pub fn new(base: Style) -> Self {
        Self {
            base,
            rules: Vec::new(),
        }
    }

    pub fn push(&mut self, rule: impl AsRef<str>, style: RelativeStyle) -> Result<()> {
        for rule in parse_rules(rule)? {
            self.rules.push((rule, style.clone()));
        }
        Ok(())
    }

    pub fn get(&self, tree: &[StyleTag]) -> Style {
        let mut style = self.base.clone();
        for (rule, rel_style) in &self.rules {
            if rule.matches_loose(tree) {
                style.apply_font(rel_style);
                if rule.matches_exact(tree) {
                    style.apply_block(rel_style);
                }
            }
        }
        style
    }
}

impl Default for StyleSheet {
    fn default() -> Self {
        lazy_static! {
            static ref DEFAULT_STYLESHEET: StyleSheet = {
                let mut this = StyleSheet::new(Style::default());
                this.push(
                    "*",
                    RelativeStyle {
                        margin_top: Some(60),
                        ..Default::default()
                    },
                )
                .unwrap();
                this.push(
                    "h1",
                    RelativeStyle {
                        font_width: Some(3),
                        font_height: Some(3),
                        bold: Some(true),
                        ..Default::default()
                    },
                )
                .unwrap();
                this.push(
                    "ul ul, ul ol, ol ol, ol ul",
                    RelativeStyle {
                        margin_top: Some(0),
                        margin_bottom: Some(0),
                        ..Default::default()
                    },
                )
                .unwrap();
                this.push(
                    "li",
                    RelativeStyle {
                        margin_top: Some(12),
                        margin_left: Some(28),
                        ..Default::default()
                    },
                )
                .unwrap();
                this.push(
                    "> ul > li, > ol > li",
                    RelativeStyle {
                        margin_left: Some(0),
                        ..Default::default()
                    },
                )
                .unwrap();
                this.push(
                    "ul > li",
                    RelativeStyle {
                        prefix: Some("* ".into()),
                        ..Default::default()
                    },
                )
                .unwrap();
                this.push(
                    "strong",
                    RelativeStyle {
                        display: Some(Display::Inline),
                        bold: Some(true),
                        ..Default::default()
                    },
                )
                .unwrap();
                this.push(
                    "em",
                    RelativeStyle {
                        display: Some(Display::Inline),
                        underline: Some(UnderlineThickness::OneDot),
                        ..Default::default()
                    },
                )
                .unwrap();
                this
            };
        }
        DEFAULT_STYLESHEET.clone()
    }
}
