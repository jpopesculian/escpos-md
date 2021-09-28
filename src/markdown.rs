use crate::error::{Error, Result};
use crate::printer::{Printer, PrinterDevice};
use crate::style::{StyleSheet, StyleTag};
use crate::tag_ext::TagExt;
use pulldown_cmark::{Event, Tag};

#[derive(Default)]
pub struct RendererState<'a> {
    tree: Vec<Tag<'a>>,
}

impl<'a> RendererState<'a> {
    fn push_tag(&mut self, tag: Tag<'a>) {
        self.tree.push(tag)
    }

    fn pop_tag(&mut self, tag: &Tag<'a>) -> Result<()> {
        if self.tree.last() != Some(tag) {
            Err(Error::UnexpectedTag(tag.clone().to_static()))
        } else {
            self.tree.pop();
            Ok(())
        }
    }

    fn style_tags(&self) -> Result<Vec<StyleTag>> {
        self.tree.iter().map(|tag| tag.style_tag()).collect()
    }
}

impl<D> Printer<D>
where
    D: PrinterDevice,
{
    pub fn markdown<'a, I>(&mut self, iter: I, styles: &StyleSheet) -> Result<&mut Self>
    where
        I: Iterator<Item = Event<'a>>,
    {
        let mut state = RendererState::default();
        for event in iter {
            eprintln!("=> {:?}", event);
            match event {
                Event::Start(tag) => {
                    state.push_tag(tag.clone());
                    let style = styles.get(&state.style_tags()?);
                    self.font_style(&style)?;
                    self.begin_block_style(&style)?;
                }
                Event::End(tag) => {
                    let style = styles.get(&state.style_tags()?);
                    self.end_block_style(&style)?;
                    state.pop_tag(&tag)?;
                    let style = styles.get(&state.style_tags()?);
                    self.font_style(&style)?;
                }
                Event::Text(text) => {
                    self.print(text)?;
                }
                Event::Code(text) => {
                    let mut style_tags = state.style_tags()?;
                    style_tags.push(StyleTag::Code);
                    let style = styles.get(&style_tags);
                    self.font_style(&style)?;
                    self.begin_block_style(&style)?;

                    self.print(text)?;

                    self.end_block_style(&style)?;
                    let style = styles.get(&state.style_tags()?);
                    self.font_style(&style)?;
                }
                _ => unimplemented!(),
            }
        }
        Ok(self)
    }
}
