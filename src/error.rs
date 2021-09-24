pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Could not convert string to CP437")]
    Cp437(codepage_437::Cp437Error),
    #[error("Image scale must be greater than 0 and less than or equal to 1")]
    InvalidImageScale,
    #[error("Character magnification must greater than 0 and less than or equal to 8")]
    InvalidCharMagnification,
    #[error("Spacing must be between 0 and 255 inclusive")]
    InvalidSpacingParam,
    #[error("Unsupported Markdown Tag: {:?}", _0)]
    UnsupportedTag(pulldown_cmark::Tag<'static>),
}

impl From<codepage_437::IntoCp437Error> for Error {
    fn from(err: codepage_437::IntoCp437Error) -> Self {
        err.cp437_error().into()
    }
}

impl From<codepage_437::Cp437Error> for Error {
    fn from(err: codepage_437::Cp437Error) -> Self {
        Self::Cp437(err)
    }
}

impl<'a> From<pulldown_cmark::Tag<'a>> for Error {
    fn from(tag: pulldown_cmark::Tag<'a>) -> Self {
        use pulldown_cmark::{CodeBlockKind, CowStr, Tag};
        let tag: Tag<'static> = match tag {
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
        };
        Error::UnsupportedTag(tag)
    }
}
