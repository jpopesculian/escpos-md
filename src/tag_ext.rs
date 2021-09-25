use crate::error::{Error, Result};
use crate::style::StyleTag;
use pulldown_cmark::{CodeBlockKind, CowStr, Tag};

pub trait TagExt {
    fn to_static(self) -> Tag<'static>;
    fn style_tag(&self) -> Result<StyleTag>;
}

impl<'a> TagExt for Tag<'a> {
    fn to_static(self) -> Tag<'static> {
        match self {
            Tag::CodeBlock(CodeBlockKind::Fenced(kind)) => {
                let kind: CowStr<'static> = CowStr::Boxed(kind.to_string().into_boxed_str());
                Tag::CodeBlock(CodeBlockKind::Fenced(kind))
            }
            Tag::FootnoteDefinition(def) => {
                let def: CowStr<'static> = CowStr::Boxed(def.to_string().into_boxed_str());
                Tag::FootnoteDefinition(def)
            }
            Tag::Link(ty, url, title) => {
                let title: CowStr<'static> = CowStr::Boxed(title.to_string().into_boxed_str());
                let url: CowStr<'static> = CowStr::Boxed(url.to_string().into_boxed_str());
                Tag::Link(ty, url, title)
            }
            Tag::Image(ty, url, title) => {
                let title: CowStr<'static> = CowStr::Boxed(title.to_string().into_boxed_str());
                let url: CowStr<'static> = CowStr::Boxed(url.to_string().into_boxed_str());
                Tag::Link(ty, url, title)
            }
            Tag::CodeBlock(CodeBlockKind::Indented) => Tag::CodeBlock(CodeBlockKind::Indented),
            Tag::Paragraph => Tag::Paragraph,
            Tag::Heading(heading) => Tag::Heading(heading),
            Tag::BlockQuote => Tag::BlockQuote,
            Tag::List(list) => Tag::List(list),
            Tag::Item => Tag::Item,
            Tag::Table(table) => Tag::Table(table),
            Tag::TableRow => Tag::TableRow,
            Tag::TableHead => Tag::TableHead,
            Tag::TableCell => Tag::TableCell,
            Tag::Emphasis => Tag::Emphasis,
            Tag::Strong => Tag::Strong,
            Tag::Strikethrough => Tag::Strikethrough,
        }
    }

    fn style_tag(&self) -> Result<StyleTag> {
        Ok(match self {
            Tag::Paragraph => StyleTag::P,
            Tag::Heading(1) => StyleTag::H1,
            Tag::Heading(2) => StyleTag::H2,
            Tag::Heading(3) => StyleTag::H3,
            Tag::Heading(4) => StyleTag::H4,
            Tag::Heading(5) => StyleTag::H5,
            Tag::BlockQuote => StyleTag::Blockquote,
            Tag::CodeBlock(_) => StyleTag::Codeblock,
            Tag::List(None) => StyleTag::Ul,
            Tag::List(Some(..)) => StyleTag::Ol,
            Tag::Item => StyleTag::Li,
            Tag::Emphasis => StyleTag::Em,
            Tag::Strong => StyleTag::Strong,
            Tag::Strikethrough => StyleTag::Strikethrough,
            Tag::Link(..) => StyleTag::A,
            Tag::Image(..) => StyleTag::Img,
            tag => return Err(Error::UnsupportedTag(tag.clone().to_static())),
        })
    }
}
