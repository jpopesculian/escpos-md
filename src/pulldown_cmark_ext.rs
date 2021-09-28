use crate::error::{Error, Result};
use crate::style::StyleTag;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag};

pub trait CowStrExt {
    fn to_static(&self) -> CowStr<'static>;
}

impl<'a> CowStrExt for CowStr<'a> {
    fn to_static(&self) -> CowStr<'static> {
        CowStr::Boxed(self.to_string().into_boxed_str())
    }
}

pub trait TagExt {
    fn to_static(self) -> Tag<'static>;
    fn style_tag(&self) -> Result<StyleTag>;
}

impl<'a> TagExt for Tag<'a> {
    fn to_static(self) -> Tag<'static> {
        match self {
            Tag::CodeBlock(CodeBlockKind::Fenced(kind)) => {
                Tag::CodeBlock(CodeBlockKind::Fenced(kind.to_static()))
            }
            Tag::FootnoteDefinition(def) => Tag::FootnoteDefinition(def.to_static()),
            Tag::Link(ty, url, title) => Tag::Link(ty, url.to_static(), title.to_static()),
            Tag::Image(ty, url, title) => Tag::Link(ty, url.to_static(), title.to_static()),
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
            Tag::Heading(6) => StyleTag::H6,
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

pub trait EventExt {
    fn to_static(self) -> Event<'static>;
}

impl<'a> EventExt for Event<'a> {
    fn to_static(self) -> Event<'static> {
        match self {
            Event::Start(tag) => Event::Start(tag.to_static()),
            Event::End(tag) => Event::End(tag.to_static()),
            Event::Text(tag) => Event::Text(tag.to_static()),
            Event::Code(tag) => Event::Code(tag.to_static()),
            Event::Html(tag) => Event::Html(tag.to_static()),
            Event::FootnoteReference(tag) => Event::FootnoteReference(tag.to_static()),
            Event::SoftBreak => Event::SoftBreak,
            Event::HardBreak => Event::HardBreak,
            Event::Rule => Event::Rule,
            Event::TaskListMarker(b) => Event::TaskListMarker(b),
        }
    }
}
