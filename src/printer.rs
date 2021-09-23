use crate::command::{CharMagnification, Command};
use crate::config::PrinterConfig;
use crate::error::Result;
use crate::instruction::EscposImage;
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

#[derive(Clone, Debug, Default)]
pub struct PrinterState {
    char_spacing: u8,
    line_spacing: Option<u8>,
    char_magnification: CharMagnification,
}

#[derive(Clone, Debug)]
pub struct Printer<D> {
    device: D,
    config: PrinterConfig,
    state: PrinterState,
}

impl<D> Printer<D> {
    pub fn builder() -> PrinterConfig {
        PrinterConfig::default()
    }

    pub fn new(device: D, config: PrinterConfig) -> Self {
        let state = PrinterState {
            char_spacing: config.char_spacing as u8,
            line_spacing: None,
            char_magnification: CharMagnification::default(),
        };
        Printer {
            device,
            config,
            state,
        }
    }
}

impl PrinterConfig {
    pub fn build<D>(&self, device: D) -> Printer<D> {
        Printer::new(device, self.clone())
    }
}

impl<D> Printer<D>
where
    D: PrinterDevice,
{
    pub fn print(&mut self, text: impl ToString) -> Result<&mut Self> {
        let content = text.to_string().into_cp437(&CP437_CONTROL)?;
        unsafe { self.raw(content) }
    }

    pub fn println(&mut self, text: impl ToString) -> Result<&mut Self> {
        self.print(text.to_string() + "\n")
    }

    pub fn feed_lines(&mut self, lines: usize) -> Result<&mut Self> {
        for _ in 0..lines / u8::MAX as usize {
            self.command(&Command::FeedLines { lines: u8::MAX })?;
        }
        self.command(&Command::FeedLines { lines: lines as u8 })
    }

    pub fn feed_paper(&mut self, units: usize) -> Result<&mut Self> {
        for _ in 0..units / u8::MAX as usize {
            self.command(&Command::FeedPaper { units: u8::MAX })?;
        }
        self.command(&Command::FeedPaper { units: units as u8 })
    }

    pub fn cut(&mut self) -> Result<&mut Self> {
        self.command(&Command::Cut)
    }

    pub fn command(&mut self, cmd: &Command) -> Result<&mut Self> {
        unsafe {
            self.raw(&cmd.as_bytes())?;
        }
        match cmd {
            Command::LineSpacing { units } => self.state.line_spacing = Some(*units),
            Command::DefaultLineSpacing => self.state.line_spacing = None,
            Command::CharSpacing { units } => self.state.char_spacing = *units,
            Command::CharSize { magnification } => self.state.char_magnification = *magnification,
            _ => {} // do nothing
        }
        Ok(self)
    }

    pub fn image(&mut self, image: &EscposImage) -> Result<&mut Self> {
        unsafe { self.raw(image.as_bytes(self.config.width, self.state.line_spacing)) }
    }

    pub unsafe fn raw(&mut self, data: impl AsRef<[u8]>) -> Result<&mut Self> {
        self.device.write_all(data.as_ref())?;
        Ok(self)
    }
}
