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
    #[error("Misaligned Markdown Tag: {:?}", _0)]
    UnexpectedTag(pulldown_cmark::Tag<'static>),
    #[error("Empty render tree")]
    EmptyRenderTree,
    #[error("Invalid rule tag: {}", _0)]
    InvalidRuleTag(String),
    #[error("Dangling direct child modifier '>'")]
    DanglingDirectChild,
    #[error("Empty rule string")]
    EmptyRuleString,
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
