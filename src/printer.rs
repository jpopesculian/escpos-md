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

pub struct PrinterBuilder {
    pub width: usize,
}

impl PrinterBuilder {
    pub fn width(&mut self, width: usize) -> &mut Self {
        self.width = width;
        self
    }

    pub fn build<D>(&mut self, device: D) -> Printer<D> {
        Printer {
            device,
            width: self.width,
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

pub struct Printer<D> {
    device: D,
    width: usize,
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
        self.raw(content)
    }

    pub fn println(&mut self, text: impl ToString) -> Result<&mut Self> {
        self.print(text.to_string() + "\n")
    }

    pub fn jump(&mut self, lines: usize) -> Result<&mut Self> {
        let mut bytes = Vec::with_capacity(lines);
        bytes.resize(lines, b'\n');
        self.raw(bytes)
    }

    pub fn cut(&mut self) -> Result<&mut Self> {
        self.command(&Command::Cut)
    }

    pub fn command(&mut self, cmd: &Command) -> Result<&mut Self> {
        self.raw(&cmd.as_bytes())
    }

    pub fn image(&mut self, image: &EscposImage) -> Result<&mut Self> {
        self.raw(image.as_bytes(self.width))
    }

    pub fn raw(&mut self, data: impl AsRef<[u8]>) -> Result<&mut Self> {
        self.device.write_all(data.as_ref())?;
        Ok(self)
    }
}
