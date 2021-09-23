use escpos_md::instruction::{BitMapAlgorithm, EscposImage, ImageOptions};
use escpos_md::{PrinterConfig, Result};
use std::io;

fn main() -> Result<()> {
    let img = image::open("./examples/lena.jpg").unwrap();
    PrinterConfig::tm_t20ii()
        .build(io::stdout())?
        .println("dithered:")?
        .image(&EscposImage::new(&img, &Default::default()))?
        .println("threshold:")?
        .image(&EscposImage::new(
            &img,
            &ImageOptions::default().bit_map_algorithm(BitMapAlgorithm::Threshold(80)),
        ))?
        .cut()?;
    Ok(())
}
