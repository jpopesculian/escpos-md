use escpos_md::command::Justification;
use escpos_md::instruction::{EscposImage, ImageOptions};
use escpos_md::{PrinterConfig, Result};
use std::io;

fn main() -> Result<()> {
    let img = image::open("./examples/lena.jpg").unwrap();
    let escpos_img = EscposImage::new(&img, ImageOptions::default().scale(0.3)?);
    PrinterConfig::tm_t20ii()
        .build(io::stdout())?
        .reset()?
        .println("Left: Hello world!")?
        .image(&escpos_img)?
        .justification(Justification::Center)?
        .println("Center: Hello world!")?
        .image(&escpos_img)?
        .justification(Justification::Right)?
        .println("Right: Hello world!")?
        .image(&escpos_img)?
        .cut()?;
    Ok(())
}
