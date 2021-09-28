use super::rule::Enumerable;
use crate::error::{Error, Result};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum StyleTag {
    P,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    Blockquote,
    Code,
    Codeblock,
    Ul,
    Ol,
    Li,
    Em,
    Strong,
    Strikethrough,
    Hr,
    A,
    Img,
    ImgCaption,
}

impl FromStr for StyleTag {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use StyleTag::*;
        Ok(match s {
            "p" => P,
            "h1" => H1,
            "h2" => H2,
            "h3" => H3,
            "h4" => H4,
            "h5" => H5,
            "h6" => H6,
            "blockquote" => Blockquote,
            "code" => Code,
            "codeblock" => Codeblock,
            "ul" => Ul,
            "ol" => Ol,
            "li" => Li,
            "em" => Em,
            "strong" => Strong,
            "strikethrough" => Strikethrough,
            "hr" => Hr,
            "a" => A,
            "img" => Img,
            "imgcaption" => ImgCaption,
            _ => return Err(Error::InvalidRuleTag(s.to_string())),
        })
    }
}

impl Enumerable for StyleTag {
    fn enumerate_all() -> Vec<Self> {
        use StyleTag::*;
        vec![
            P,
            H1,
            H2,
            H3,
            H4,
            H5,
            H6,
            Blockquote,
            Code,
            Codeblock,
            Ul,
            Ol,
            Li,
            Em,
            Strong,
            Strikethrough,
            Hr,
            A,
            Img,
            ImgCaption,
        ]
    }
}
