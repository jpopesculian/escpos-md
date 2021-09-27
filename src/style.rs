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
    pub prefix: String,
    pub split_words: bool,
    pub underline: UnderlineThickness,
    pub white_black_reverse: bool,
    // block styles
    pub justification: Justification,
    pub margin_left: usize,
    pub margin_bottom: usize,
    pub margin_top: usize,
}

impl<D> Printer<D>
where
    D: PrinterDevice,
{
    pub(crate) fn begin_style(&mut self, style: &Style) -> Result<&mut Self> {
        self.bold(style.bold)?
            .char_size(style.char_magnification)?
            .char_spacing(style.char_spacing)?
            .font(style.font)?
            .line_spacing(style.line_spacing)?
            .split_words(style.split_words)?
            .underline(style.underline)?
            .white_black_reverse(style.white_black_reverse)?;
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

    pub(crate) fn end_style(&mut self, style: &Style) -> Result<&mut Self> {
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
            prefix,
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
            justification,
            margin_top,
            margin_bottom,
            margin_left
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StyleTag {
    Any,
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
    pub fn matches(&self, other: &Self) -> bool {
        self == other || matches!(other, Self::Any) || matches!(self, Self::Any)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RuleMatch {
    Child,
    Exact,
}

#[derive(Debug, Clone)]
pub struct Rule(Vec<StyleTag>);

impl Rule {
    pub fn new<I>(tags: I) -> Self
    where
        I: IntoIterator<Item = StyleTag>,
    {
        Rule(tags.into_iter().collect())
    }

    pub fn is_match(&self, tree: &[StyleTag]) -> Option<RuleMatch> {
        Self::find_match(tree, &self.0).map(|mat| {
            if *mat.last().unwrap() == (tree.len() - 1) {
                RuleMatch::Exact
            } else {
                RuleMatch::Child
            }
        })
    }

    fn find_match(tree: &[StyleTag], rule: &[StyleTag]) -> Option<Vec<usize>> {
        match rule {
            [] => None,
            [tag] => Self::find_tag_matches(tree, tag)
                .next()
                .map(|idx| vec![idx]),
            [rem @ .., tag] => {
                for mat in Self::find_tag_matches(tree, tag) {
                    let rem_tree = &tree[..mat];
                    if let Some(mut mats) = Self::find_match(rem_tree, rem) {
                        mats.push(mat);
                        return Some(mats);
                    }
                }
                None
            }
        }
    }

    fn find_tag_matches<'a>(
        tree: &'a [StyleTag],
        tag: &'a StyleTag,
    ) -> impl Iterator<Item = usize> + 'a {
        tree.iter()
            .enumerate()
            .rev()
            .filter_map(move |(idx, tree_tag)| {
                if tree_tag.matches(tag) {
                    Some(idx)
                } else {
                    None
                }
            })
    }
}

#[derive(Debug, Clone)]
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

    pub fn push(&mut self, rule: Rule, style: RelativeStyle) {
        self.rules.push((rule, style));
    }

    pub fn get(&self, tree: &[StyleTag]) -> Style {
        let mut style = self.base.clone();
        for (rule, rel_style) in &self.rules {
            if let Some(mat_kind) = rule.is_match(tree) {
                style.apply_font(rel_style);
                if matches!(mat_kind, RuleMatch::Exact) {
                    style.apply_block(rel_style);
                }
            }
        }
        style
    }
}

impl Default for StyleSheet {
    fn default() -> Self {
        use StyleTag::*;
        let mut this = Self::new(Style::default());
        this.push(
            Rule::new([Any]),
            RelativeStyle {
                margin_top: Some(60),
                ..Default::default()
            },
        );
        this.push(
            Rule::new([H1]),
            RelativeStyle {
                font_width: Some(3),
                font_height: Some(3),
                bold: Some(true),
                ..Default::default()
            },
        );
        for nested_list in [[Ul, Ul], [Ul, Ol], [Ol, Ol], [Ol, Ul]] {
            this.push(
                Rule::new(nested_list),
                RelativeStyle {
                    margin_top: Some(0),
                    margin_bottom: Some(0),
                    ..Default::default()
                },
            );
        }
        this.push(
            Rule::new([Li]),
            RelativeStyle {
                margin_top: Some(12),
                margin_left: Some(28),
                ..Default::default()
            },
        );
        this.push(
            Rule::new([Ul, Li]),
            RelativeStyle {
                prefix: Some("* ".into()),
                ..Default::default()
            },
        );
        this.push(
            Rule::new([Strong]),
            RelativeStyle {
                bold: Some(true),
                ..Default::default()
            },
        );
        this.push(
            Rule::new([Em]),
            RelativeStyle {
                underline: Some(UnderlineThickness::OneDot),
                ..Default::default()
            },
        );
        this
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rule_matches() {
        use StyleTag::*;
        let tree = &[A, P, P, H1, P, P, Ul, Li];
        assert_eq!(Rule::find_match(tree, &[P]), Some(vec![5]));
        assert_eq!(Rule::find_match(tree, &[P, P]), Some(vec![4, 5]));
        assert_eq!(Rule::find_match(tree, &[P, Any, P]), Some(vec![2, 4, 5]));
        assert_eq!(
            Rule::find_match(tree, &[P, H1, Any, P]),
            Some(vec![2, 3, 4, 5])
        );
        assert_eq!(
            Rule::find_match(tree, &[P, Any, H1, P]),
            Some(vec![1, 2, 3, 5])
        );
        assert_eq!(
            Rule::find_match(tree, &[A, P, Any, H1, P]),
            Some(vec![0, 1, 2, 3, 5])
        );
        assert_eq!(Rule::find_match(tree, &[Em, P, Any, H1, P]), None);
        assert_eq!(Rule::find_match(tree, &[Ol]), None);
        assert_eq!(Rule::find_match(tree, &[]), None);
    }
}
