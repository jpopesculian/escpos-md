use crate::config::width::DEFAULT_WIDTH;
use crate::error::Result;
use crate::{command::Command, instruction::EscposImage};
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
pub struct PrinterConfig {
    width: usize,
}

#[derive(Clone, Debug, Default)]
pub struct PrinterState {
    line_spacing: Option<u8>,
}

pub struct PrinterBuilder {
    pub width: usize,
}

impl PrinterBuilder {
    pub fn width(&mut self, width: usize) -> &mut Self {
        self.width = width;
        self
    }

    fn config(&self) -> PrinterConfig {
        PrinterConfig { width: self.width }
    }

    pub fn build<D>(&self, device: D) -> Printer<D> {
        Printer {
            device,
            config: self.config(),
            state: PrinterState::default(),
        }
    }
}

impl Default for PrinterBuilder {
    fn default() -> Self {
        Self {
            width: DEFAULT_WIDTH,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Printer<D> {
    device: D,
    config: PrinterConfig,
    state: PrinterState,
}

impl Printer<io::Stdout> {
    pub fn builder() -> PrinterBuilder {
        PrinterBuilder::default()
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
