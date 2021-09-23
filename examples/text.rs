use escpos_md::command::{CharMagnification, Font, UnderlineThickness};
use escpos_md::{PrinterConfig, Result};
use std::io;

fn main() -> Result<()> {
    let mut printer = PrinterConfig::tm_t20ii().build(io::stdout())?;

    macro_rules! example {
        ($header:literal, $func:ident, $param:expr) => {
            printer
                .reset()?
                .println(format!("═══ {} ═══", $header))?
                .$func($param)?
                .println("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec bibendum, turpis vitae feugiat")?
                .println("")?;
        };
    }

    example!("FontA", font, Font::FontA);
    example!("FontB", font, Font::FontB);
    example!("Bold", bold, true);
    example!("Underline 1 Dot", underline, UnderlineThickness::OneDot);
    example!("Underline 2 Dot", underline, UnderlineThickness::TwoDot);
    example!("Double Strike", double_strike, true);
    example!("White/Black Reversed", white_black_reverse, true);
    example!("Double Width", char_size, CharMagnification::new(2, 1)?);
    example!("Double Height", char_size, CharMagnification::new(1, 2)?);
    example!(
        "Double Width/Height",
        char_size,
        CharMagnification::new(2, 2)?
    );
    example!("Char Spacing 6", char_spacing, 6);
    example!("Line Spacing 100", line_spacing, Some(100));
    example!("Split Words Disabled", split_words, false);

    printer.cut()?;

    Ok(())
}
