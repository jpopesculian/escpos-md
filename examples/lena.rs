use escpos_md::config::width::TM_T20II_80MM_WIDTH;
use escpos_md::instruction::{BitMapAlgorithm, EscposImage, ImageOptions};
use escpos_md::{Printer, Result};
use std::io;

fn main() -> Result<()> {
    let img = image::open("./examples/lena.jpg").unwrap();
    Printer::builder()
        .width(TM_T20II_80MM_WIDTH)
        .build(io::stdout())
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
