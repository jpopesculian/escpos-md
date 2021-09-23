use escpos_md::config::width::TM_T20II_80MM_WIDTH;
use escpos_md::{Printer, Result};
use std::io;

fn main() -> Result<()> {
    Printer::builder()
        .width(TM_T20II_80MM_WIDTH)
        .build(io::stdout())
        .println("Hello world!")?
        .feed_lines(5)?
        .cut()?;
    Ok(())
}
