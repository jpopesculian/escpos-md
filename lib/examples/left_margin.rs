use escpos_md::instruction::EscposImage;
use escpos_md::{PrinterConfig, Result};
use std::io;

const LOREM_IPSUM: &str =
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec bibendum, turpis vitae feugiat";

fn main() -> Result<()> {
    let img = image::open("./examples/lena.jpg").unwrap();
    let escpos_img = EscposImage::new(&img, &Default::default());
    PrinterConfig::tm_t20ii()
        .build(io::stdout())?
        .reset()?
        .left_margin(100)?
        .println(format!("Left margin {}: {}", 100, LOREM_IPSUM))?
        .image(&escpos_img)?
        .left_margin(200)?
        .println(format!("Left margin {}: {}", 200, LOREM_IPSUM))?
        .left_margin(300)?
        .println(format!("Left margin {}: {}", 300, LOREM_IPSUM))?
        .reset()?
        .println(format!("Left margin {}: {}", 0, LOREM_IPSUM))?
        .cut()?;
    Ok(())
}
