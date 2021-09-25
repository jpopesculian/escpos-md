use escpos_md::style::StyleSheet;
use escpos_md::{MarkdownParser, PrinterConfig, Result};
use std::io;

const TEST_MD: &str = r#"
# Heading 1

Hello world! This is _an italic phrase_ and this is a __bold__ one.

This is an unordered list:

* One Item
*
* Another Item of the list that is fairly long
    * With an embedded list
        * and a third tier
    * In that list
* And the last item
"#;

fn main() -> Result<()> {
    let parser = MarkdownParser::new(TEST_MD);
    let styles = StyleSheet::default();
    PrinterConfig::tm_t20ii()
        .build(io::stdout())?
        .reset()?
        .markdown(parser, &styles)?
        .cut()?;
    Ok(())
}
