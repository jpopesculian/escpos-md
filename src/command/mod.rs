mod char_magnification;
mod charset;
mod code_table;
mod font;

pub use char_magnification::CharMagnification;
pub use charset::Charset;
pub use code_table::CodeTable;
pub use font::Font;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum UnderlineThickness {
    Off = 0,
    OneDot = 1,
    TwoDot = 2,
}

impl Default for UnderlineThickness {
    fn default() -> Self {
        Self::Off
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Justification {
    Left = 0,
    Center = 1,
    Right = 2,
}

impl Default for Justification {
    fn default() -> Self {
        Self::Left
    }
}

/// Common commands usefull for the printer
#[derive(Clone, Debug, Copy)]
pub enum Command {
    /// Cuts the paper after 0x96 vertical spaces
    Cut,
    /// Equivalent to ESC @
    Init,
    /// Print mode selected to init the fonts. Equivalent to ESC ! 0
    PrintModeDefault,
    /// Set an international character set, Equivalent to ESC R
    Charset(Charset),
    /// Selects a different code table, Equivalent to ESC t
    CodeTable(CodeTable),
    /// Sets up a font. Equivalent to ESC M
    Font(Font),
    Underline(UnderlineThickness),
    Bold(bool),
    DoubleStrike(bool),
    WhiteBlackReverse(bool),
    /// Equivalent to ESC * m = 0
    Bitmap,
    /// Change line size
    FeedPaper(u8),
    FeedLines(u8),
    LineSpacing(u8),
    DefaultLineSpacing,
    CharSpacing(u8),
    CharSize(CharMagnification),
    SplitWords(bool),
    LeftMargin(u16),
    Justification(Justification),
}

impl Command {
    /// Returns the byte-array representation of each command
    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            Command::Cut => vec![0x1d, 0x56, 0x41, 0x96],
            Command::Init => vec![0x1d, 0x40],
            Command::PrintModeDefault => vec![0x01b, 0x21, 0x00],
            Command::Charset(charset) => {
                let mut res = vec![0x1b, 0x52];
                res.append(&mut charset.as_bytes());
                res
            }
            Command::CodeTable(code_table) => {
                let mut res = vec![0x1b, 0x74];
                res.append(&mut code_table.as_bytes());
                res
            }
            Command::Font(font) => {
                let mut res = vec![0x1b, 0x4d];
                res.append(&mut font.as_bytes());
                res
            }
            Command::Underline(thickness) => vec![0x1b, 0x2d, *thickness as u8],
            Command::Bold(bold) => vec![0x1b, 0x45, *bold as u8],
            Command::DoubleStrike(double_strike) => vec![0x1b, 0x47, *double_strike as u8],
            Command::WhiteBlackReverse(reverse) => vec![0x1d, 0x42, *reverse as u8],
            Command::Bitmap => vec![0x1b, 0x2a],
            Command::FeedPaper(units) => vec![0x1b, 0x4a, *units],
            Command::FeedLines(lines) => vec![0x1b, 0x64, *lines],
            Command::LineSpacing(units) => vec![0x1b, 0x33, *units],
            Command::DefaultLineSpacing => vec![0x1b, 0x32],
            Command::CharSpacing(units) => vec![0x1b, 0x20, *units],
            Command::CharSize(magnification) => vec![0x1d, 0x21, magnification.to_byte()],
            Command::SplitWords(_) => vec![],
            Command::LeftMargin(margin) => {
                let mut res = vec![0x1d, 0x4c];
                res.append(&mut margin.to_le_bytes().to_vec());
                res
            }
            Command::Justification(justification) => vec![0x1b, 0x61, *justification as u8],
        }
    }
}
