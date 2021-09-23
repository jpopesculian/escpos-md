use escpos_md::{PrinterConfig, Result};
use std::io;

fn main() -> Result<()> {
    PrinterConfig::tm_t20ii()
        .build(io::stdout())?
        .println("Hello world!")?
        .feed_lines(5)?
        .cut()?;
    Ok(())
}
