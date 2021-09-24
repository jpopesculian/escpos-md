use crate::error::Result;
use crate::style::{RelativeStyle, Style};
use pulldown_cmark::Tag;

pub struct RenderState {}

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

#[derive(Default, Debug, Clone)]
pub struct StyleSheet {
    root: Style,
    styles: [RelativeStyle; 17],
}

impl StyleSheet {
    pub fn new(root: Style) -> Self {
        Self {
            root,
            styles: Default::default(),
        }
    }
    pub fn get(&self, tag: &StyleTag) -> &RelativeStyle {
        &self.styles[*tag as usize]
    }
    pub fn set(&mut self, tag: &StyleTag, style: RelativeStyle) {
        self.styles[*tag as usize] = style;
    }
}

impl StyleTag {
    pub fn for_tag(tag: &Tag) -> Result<Self> {
        Ok(match tag {
            Tag::Paragraph => Self::P,
            Tag::Heading(1) => Self::H1,
            Tag::Heading(2) => Self::H2,
            Tag::Heading(3) => Self::H3,
            Tag::Heading(4) => Self::H4,
            Tag::Heading(5) => Self::H5,
            Tag::BlockQuote => Self::Blockquote,
            Tag::CodeBlock(_) => Self::Codeblock,
            Tag::List(None) => Self::Ul,
            Tag::List(Some(..)) => Self::Ol,
            Tag::Item => Self::Li,
            Tag::Emphasis => Self::Em,
            Tag::Strong => Self::Strong,
            Tag::Strikethrough => Self::Strikethrough,
            Tag::Link(..) => Self::A,
            Tag::Image(..) => Self::Img,
            tag => return Err(tag.clone().into()),
        })
    }
}
