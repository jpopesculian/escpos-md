use clap::Clap;
use escpos_md::{MarkdownParser, PrinterConfig, Result};
use std::io::{self, Read};

#[derive(Clap)]
struct Opts {
    #[clap(short, long)]
    no_cut: bool,
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    let mut md = String::new();
    io::stdin().read_to_string(&mut md)?;
    let parser = MarkdownParser::new(&md);

    let mut printer = PrinterConfig::tm_t20ii().build(io::stdout())?;

    printer.reset()?.markdown(parser, &Default::default())?;
    if !opts.no_cut {
        printer.cut()?;
    }

    Ok(())
}
