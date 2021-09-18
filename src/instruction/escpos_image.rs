use crate::command::Command;
use crate::error::{Error, Result};
use image::Pixel;

#[derive(Debug, Clone)]
pub enum BitMapAlgorithm {
    Threshold(u8),
    Dithering,
}

#[derive(Debug, Clone)]
pub enum Justification {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
pub struct ImageOptions {
    bit_map_algorithm: BitMapAlgorithm,
    justification: Justification,
    scale: f64,
    filter_type: image::imageops::FilterType,
}

impl ImageOptions {
    pub fn bit_map_algorithm(&mut self, bit_map_algorithm: BitMapAlgorithm) -> &mut Self {
        self.bit_map_algorithm = bit_map_algorithm;
        self
    }
    pub fn scale(&mut self, scale: f64) -> Result<&mut Self> {
        if scale <= 0. || scale > 1. {
            Err(Error::InvalidImageScale)
        } else {
            self.scale = scale;
            Ok(self)
        }
    }
    pub fn justification(&mut self, justification: Justification) -> &mut Self {
        self.justification = justification;
        self
    }
    pub fn filter_type(&mut self, filter_type: image::imageops::FilterType) -> &mut Self {
        self.filter_type = filter_type;
        self
    }
}

impl Default for ImageOptions {
    fn default() -> Self {
        Self {
            bit_map_algorithm: BitMapAlgorithm::Dithering,
            justification: Justification::Left,
            scale: 1.,
            filter_type: image::imageops::FilterType::Gaussian,
        }
    }
}

pub struct EscposImage {
    img: image::GrayImage,
    opts: ImageOptions,
}

impl EscposImage {
    pub fn new(img: &image::DynamicImage, opts: &ImageOptions) -> Self {
        let mut img = img.to_luma8();

        let (im_width, im_height) = img.dimensions();
        let aspect_ratio = (im_width as f64) / (im_height as f64);
        let sc_width = (im_width as f64) * opts.scale;
        let sc_height = ((sc_width) / aspect_ratio).floor() as u32;
        let sc_width = sc_width.floor() as u32;
        let x_offset = match opts.justification {
            Justification::Left => 0,
            Justification::Center => (im_width - sc_width) / 2,
            Justification::Right => im_width - sc_width,
        };

        let mut composite = image::GrayImage::from_pixel(im_width, sc_height, [255].into());
        image::imageops::overlay(
            &mut composite,
            &image::imageops::resize(&img, sc_width, sc_height, opts.filter_type),
            x_offset,
            0,
        );
        img = image::imageops::crop(&mut composite, 0, 0, im_width, sc_height).to_image();

        Self {
            img,
            opts: opts.clone(),
        }
    }

    pub fn as_bytes(&self, printer_width: usize) -> Vec<u8> {
        let mut feed = Vec::new();
        feed.extend_from_slice(&Command::NoLine.as_bytes());

        let (im_width, im_height) = self.img.dimensions();
        // We redefine the aspect ratio
        let aspect_ratio = (im_width as f64) / (im_height as f64);

        // Each row will contain the information of 8 rows from the picture
        //const printer_width: usize = 384;
        //const printer_width: usize = 576;
        //let mut printer_rows: Vec<[u8; printer_width]> = Vec::new();
        let mut printer_rows: Vec<Vec<u8>> = Vec::new();

        // El *3 es por la baja densidad de impresión vertical (1 byte en lugar de 3)
        let new_height = ((printer_width as f64) / (aspect_ratio * 3.0)).floor() as u32;

        let mut img = image::imageops::resize(
            &self.img,
            printer_width as u32,
            new_height,
            self.opts.filter_type,
        );
        img = match self.opts.bit_map_algorithm {
            BitMapAlgorithm::Dithering => {
                image::imageops::dither(&mut img, &image::imageops::BiLevel);
                img
            }
            BitMapAlgorithm::Threshold(threshold) => image::GrayImage::from_raw(
                img.width(),
                img.height(),
                img.into_raw()
                    .into_iter()
                    .map(|intensity| if intensity > threshold { 255 } else { 0 })
                    .collect(),
            )
            .unwrap(),
        };

        // We will turn the image into a grayscale boolean matrix
        for (y, pixel_row) in img.enumerate_rows() {
            // Here we iterate over each row of the image.
            if y % 8 == 0 {
                printer_rows.push(vec![0; printer_width as usize]);
            }
            let row = printer_rows.get_mut((y / 8) as usize).unwrap();
            // Here, we iterate horizontally this time
            for (x, y, pixel) in pixel_row {
                let ps = pixel.channels();
                // We get the color as a boolean
                let mut color = if ps[0] == 0 { 0x01 } else { 0x00 };
                // We shift the boolean by 7 - y%8 positions in the register
                color <<= 7 - y % 8;
                // An or operation preserves the previous pixels in the rows
                row[x as usize] |= color;
            }
        }

        // Finally, we push each row to the feed vector
        for (_idx, printer_row) in printer_rows.iter().enumerate() {
            // We first, declare a bitmap mode
            feed.extend_from_slice(&Command::Bitmap.as_bytes());
            // Now, we pass m
            let m = 0x01;
            feed.push(m);
            // The formula on how many pixels we will do, is nL + nH * 256
            feed.push((printer_width % 256) as u8); // nL
            feed.push((printer_width / 256) as u8); // nH
                                                    // feed.push(0x80); // nL
                                                    // feed.push(0x01); // nH
            feed.extend_from_slice(printer_row);
            feed.push(b'\n'); // Line feed and print
        }
        feed.extend_from_slice(&Command::ResetLine.as_bytes());
        feed.extend_from_slice(&Command::Reset.as_bytes());

        feed
    }
}
