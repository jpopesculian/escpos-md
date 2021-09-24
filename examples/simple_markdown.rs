use escpos_md::MarkdownParser;

const TEST_MD: &str = r#"
This is a `code *block*`

This is a

```
code fence
```
"#;

fn main() {
    let parser = MarkdownParser::new(TEST_MD);
    for event in parser {
        println!("{:?}", event);
    }
}
