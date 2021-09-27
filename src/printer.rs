use crate::command::{
    CharMagnification, Charset, CodeTable, Command, Font, Justification, UnderlineThickness,
};
use crate::config::PrinterConfig;
use crate::error::{Error, Result};
use crate::instruction::EscposImage;
use crate::split_words::split_words;
use codepage_437::{IntoCp437, CP437_CONTROL};
use std::io;

pub trait PrinterDevice {
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()>;
}

impl<T> PrinterDevice for T
where
    T: io::Write,
{
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.write_all(buf)
    }
}

#[derive(Clone, Debug)]
pub struct PrinterState {
    pub(crate) char_spacing: u8,
    pub(crate) line_spacing: Option<u8>,
    pub(crate) font: Font,
    pub(crate) left_offset: usize,
    pub(crate) split_words: bool,
    pub(crate) left_margin: u16,
    pub(crate) justification: Justification,
    pub(crate) char_magnification: CharMagnification,
}

#[derive(Clone, Debug)]
pub struct Printer<D> {
    pub(crate) device: D,
    pub(crate) config: PrinterConfig,
    pub(crate) state: PrinterState,
}

impl<D> Printer<D> {
    pub fn builder() -> PrinterConfig {
        PrinterConfig::default()
    }

    fn reduce_spacing_param(spacing: usize) -> Result<u8> {
        if spacing > u8::MAX as usize {
            Err(Error::InvalidSpacingParam)
        } else {
            Ok(spacing as u8)
        }
    }

    pub fn new(device: D, config: PrinterConfig) -> Result<Self> {
        let state = PrinterState {
            char_spacing: Self::reduce_spacing_param(config.char_spacing)?,
            line_spacing: None,
            font: Font::default(),
            left_offset: 0,
            split_words: true,
            left_margin: 0,
            justification: Justification::default(),
            char_magnification: CharMagnification::default(),
        };
        Ok(Printer {
            device,
            config,
            state,
        })
    }

    fn calc_char_size(&self) -> usize {
        (self.config.font_widths.get(&self.state.font) + self.state.char_spacing as usize)
            * self.state.char_magnification.width() as usize
    }

    fn printable_width(&self) -> usize {
        self.config.width - (self.state.left_margin as usize).min(self.config.width)
    }
}

impl PrinterConfig {
    pub fn build<D>(&self, device: D) -> Result<Printer<D>> {
        Printer::new(device, self.clone())
    }
}

macro_rules! cmd_fn {
    ($name:ident, $cmd:ident) => {
        #[inline]
        pub fn $name(&mut self) -> Result<&mut Self> {
            self.command(&Command::$cmd)
        }
    };
    ($name:ident, $cmd:ident, $param:ident, $param_ty:ty) => {
        #[inline]
        pub fn $name(&mut self, $param: $param_ty) -> Result<&mut Self> {
            self.command(&Command::$cmd($param))
        }
    };
}

impl<D> Printer<D>
where
    D: PrinterDevice,
{
    cmd_fn!(cut, Cut);
    cmd_fn!(init, Init);
    cmd_fn!(print_mode_default, PrintModeDefault);
    cmd_fn!(charset, Charset, charset, Charset);
    cmd_fn!(code_table, CodeTable, code_table, CodeTable);
    cmd_fn!(font, Font, font, Font);
    cmd_fn!(underline, Underline, thickness, UnderlineThickness);
    cmd_fn!(bold, Bold, enabled, bool);
    cmd_fn!(double_strike, DoubleStrike, enabled, bool);
    cmd_fn!(white_black_reverse, WhiteBlackReverse, enabled, bool);
    cmd_fn!(char_size, CharSize, magnification, CharMagnification);
    cmd_fn!(split_words, SplitWords, enabled, bool);
    cmd_fn!(left_margin, LeftMargin, margin, u16);
    cmd_fn!(justification, Justification, justification, Justification);

    pub fn reset(&mut self) -> Result<&mut Self> {
        self.state.split_words = true;
        let og_char_spacing = self.config.char_spacing;
        self.init()?
            .print_mode_default()?
            .white_black_reverse(false)?
            .double_strike(false)?
            .char_spacing(og_char_spacing)?
            .line_spacing(None)?
            .left_margin(0)?
            .justification(Justification::default())
    }

    pub fn print(&mut self, text: impl ToString) -> Result<&mut Self> {
        let mut content = text.to_string().into_cp437(&CP437_CONTROL)?;

        let new_offset = if self.state.split_words {
            split_words(
                &mut content,
                self.state.left_offset,
                self.printable_width(),
                self.calc_char_size(),
            )
        } else {
            (self.state.left_offset + content.len() * self.calc_char_size())
                % self.printable_width()
        };

        unsafe {
            self.raw(content)?;
        }
        self.state.left_offset = new_offset;
        Ok(self)
    }

    pub fn println(&mut self, text: impl ToString) -> Result<&mut Self> {
        self.print(text.to_string() + "\n")
    }

    pub fn feed_lines(&mut self, lines: usize) -> Result<&mut Self> {
        for _ in 0..lines / u8::MAX as usize {
            self.command(&Command::FeedLines(u8::MAX))?;
        }
        self.command(&Command::FeedLines(lines as u8))
    }

    pub fn feed_paper(&mut self, units: usize) -> Result<&mut Self> {
        for _ in 0..units / u8::MAX as usize {
            self.command(&Command::FeedPaper(u8::MAX))?;
        }
        self.command(&Command::FeedPaper(units as u8))
    }

    pub fn char_spacing(&mut self, char_spacing: usize) -> Result<&mut Self> {
        self.command(&Command::CharSpacing(Self::reduce_spacing_param(
            char_spacing,
        )?))
    }

    pub fn line_spacing(&mut self, line_spacing: Option<usize>) -> Result<&mut Self> {
        let cmd = if let Some(line_spacing) = line_spacing {
            Command::LineSpacing(Self::reduce_spacing_param(line_spacing)?)
        } else {
            Command::DefaultLineSpacing
        };
        self.command(&cmd)
    }

    pub fn command(&mut self, cmd: &Command) -> Result<&mut Self> {
        unsafe {
            self.raw(&cmd.as_bytes())?;
        }
        match cmd {
            Command::LineSpacing(units) => self.state.line_spacing = Some(*units),
            Command::DefaultLineSpacing => self.state.line_spacing = None,
            Command::CharSpacing(units) => self.state.char_spacing = *units,
            Command::CharSize(magnification) => self.state.char_magnification = *magnification,
            Command::Font(font) => self.state.font = *font,
            Command::SplitWords(split) => self.state.split_words = *split,
            Command::LeftMargin(margin) => self.state.left_margin = *margin,
            Command::Justification(justification) => self.state.justification = *justification,
            Command::FeedPaper(_) | Command::FeedLines(_) => {
                self.state.left_offset = 0;
            }
            Command::Init => {
                self.state.char_magnification = CharMagnification::default();
                self.state.font = Font::default();
            }
            _ => {} // do nothing
        }
        Ok(self)
    }

    pub fn image(&mut self, image: &EscposImage) -> Result<&mut Self> {
        unsafe {
            self.raw(image.as_bytes(
                self.printable_width(),
                self.state.justification,
                self.state.line_spacing,
            ))?;
        }
        self.state.left_offset = 0;
        Ok(self)
    }

    pub unsafe fn raw(&mut self, data: impl AsRef<[u8]>) -> Result<&mut Self> {
        self.device.write_all(data.as_ref())?;
        Ok(self)
    }
}
