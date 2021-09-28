use escpos_md::{MarkdownParser, PrinterConfig, Result};
use std::io;

fn main() -> Result<()> {
    let parser = MarkdownParser::new("Hello, World!");
    PrinterConfig::tm_t20ii()
        .build(io::stdout())?
        .reset()?
        .markdown(parser, &Default::default())?
        .cut()?;
    Ok(())
}
