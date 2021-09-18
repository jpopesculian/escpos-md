pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Could not convert string to CP437")]
    Cp437(codepage_437::Cp437Error),
    #[error("Image scale must be greater than 0 and less than or equal to 1")]
    InvalidImageScale,
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
