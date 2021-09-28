use escpos_md::{MarkdownParser, MarkdownParserOptions, PrinterConfig, Result};
use std::io;

const TEST_MD: &str = r#"
# Heading 1

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec bibendum, turpis vitae feugiat

## Heading 2

This is a paragraph with a __bold value__ as well as an _emphasized value_
and a ~~strikethrough~~ value and a `code value` and
the whole thing is split\
with a hard break and then a horizontal rule

---

This is an unordered list

* With an item
* And another item
    * And some nested item
* And then a not nested item
    * Nesting Level 2
        * Nesting Level 3
* Nesting Level 1

### Heading 3

This is an ordered list

1. An item of the list
2. The second item of the list
    * An unordered bit under the list
3. Third item of the list
    1. Sub numbers under the list
    2. Some more numbers

#### Heading 4

And thes are some `code blocks`. For example a much larger code block:

```
This is something a little bigger

    Maybe with some weird alignments
```

##### Heading 5

###### Heading 6

And then there's also some block quotes

> Like this block quote example
> With numerous lines

And of course an image

![lena](./examples/lena.jpg "With explanation")
"#;

fn main() -> Result<()> {
    let mut options = MarkdownParserOptions::empty();
    options.insert(MarkdownParserOptions::ENABLE_STRIKETHROUGH);
    let parser = MarkdownParser::new_ext(TEST_MD, options);
    PrinterConfig::tm_t20ii()
        .build(io::stdout())?
        .reset()?
        .markdown(parser, &Default::default())?
        .cut()?;
    Ok(())
}
