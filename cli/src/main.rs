use clap::Clap;
use escpos_md::instruction::{BitMapAlgorithm, ImageOptions};
use escpos_md::style::{RelativeStyle, StyleSheet};
use escpos_md::{
    MarkdownParser, MarkdownParserOptions, MarkdownRenderOptions, PrinterConfig, Result,
};
use image::imageops::FilterType;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Clap)]
struct Opts {
    #[clap(short, long)]
    no_cut: bool,
    #[clap(long)]
    strikethrough: bool,
    #[clap(long)]
    smart_punctuation: bool,
    #[clap(long)]
    margin_top: Option<usize>,
    #[clap(long)]
    margin_left: Option<usize>,
    #[clap(long)]
    margin_bottom: Option<usize>,
    #[clap(long, default_value = "1")]
    image_scale: f64,
    #[clap(long, possible_values = &["nearest", "linear", "cubic", "gaussian", "lanczos"], default_value = "lanczos")]
    image_filter_type: String,
    #[clap(long, possible_values = &["threshold", "dithering"], default_value = "dithering")]
    image_bit_map_algo: String,
    #[clap(long, default_value = "100")]
    image_threshold: u8,
    #[clap(short, long)]
    stylesheet: Option<PathBuf>,
    file: Option<PathBuf>,
}

pub type StyleSheetDefinition = HashMap<String, RelativeStyle>;

fn main() -> Result<()> {
    let opts = Opts::parse();

    let mut stylesheet = StyleSheet::default();
    if let Some(stylesheet_path) = opts.stylesheet {
        let stylesheet_def: StyleSheetDefinition =
            serde_yaml::from_reader(File::open(stylesheet_path)?)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        for (rule, def) in stylesheet_def {
            stylesheet
                .push(rule, def)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        }
    }

    let bit_map_algorithm = match opts.image_bit_map_algo.as_str() {
        "threshold" => BitMapAlgorithm::Threshold(opts.image_threshold),
        "dithering" => BitMapAlgorithm::Dithering,
        _ => unreachable!(),
    };
    let filter_type = match opts.image_filter_type.as_str() {
        "nearest" => FilterType::Nearest,
        "linear" => FilterType::Triangle,
        "cubic" => FilterType::CatmullRom,
        "gaussian" => FilterType::Gaussian,
        "lanczos" => FilterType::Lanczos3,
        _ => unreachable!(),
    };
    let mut image_opts = ImageOptions::default();
    image_opts
        .filter_type(filter_type)
        .bit_map_algorithm(bit_map_algorithm)
        .scale(opts.image_scale)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let mut render_opts = MarkdownRenderOptions::default();
    render_opts.styles = stylesheet;
    render_opts.image = image_opts;

    let mut md = String::new();
    if let Some(file_path) = opts.file {
        md = fs::read_to_string(file_path)?;
    } else {
        io::stdin().read_to_string(&mut md)?;
    }

    let mut parser_opts = MarkdownParserOptions::empty();
    if opts.strikethrough {
        parser_opts.insert(MarkdownParserOptions::ENABLE_STRIKETHROUGH);
    }
    if opts.smart_punctuation {
        parser_opts.insert(MarkdownParserOptions::ENABLE_SMART_PUNCTUATION);
    }
    let parser = MarkdownParser::new_ext(&md, parser_opts);

    let mut printer = PrinterConfig::tm_t20ii().build(io::stdout())?;

    printer.reset()?;
    if let Some(margin_top) = opts.margin_top {
        printer.feed_lines(margin_top)?;
    }
    if let Some(margin_left) = opts.margin_left {
        printer.left_margin(margin_left as u16)?;
    }
    printer.reset()?.markdown(parser, &render_opts)?;
    if let Some(margin_bottom) = opts.margin_bottom {
        printer.feed_lines(margin_bottom)?;
    }
    if !opts.no_cut {
        printer.cut()?;
    }

    Ok(())
}
