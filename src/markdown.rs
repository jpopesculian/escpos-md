use crate::error::{Error, Result};
use crate::printer::{Printer, PrinterDevice};
use crate::style::{Display, Style, StyleSheet, StyleTag};
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

    fn pop_tag(&mut self, tag: &Tag<'a>) -> Option<Tag<'a>> {
        let pos = self.tree.iter().rposition(|t| tag == t)?;
        Some(self.tree.remove(pos))
    }

    fn list_depth(&self) -> usize {
        self.tree
            .iter()
            .filter(|tag| match tag {
                Tag::List(_) => true,
                _ => false,
            })
            .count()
    }

    fn last_list(&self) -> Option<&Tag> {
        self.tree.iter().rev().find(|tag| match tag {
            Tag::List(_) => true,
            _ => false,
        })
    }

    fn last_tag(&self) -> Option<&Tag> {
        self.tree.last()
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
                    let style_tag = tag.style_tag()?;
                    match style_tag.display() {
                        Display::Block => {
                            let style = styles
                                .calc_tags(state.style_tags()?.iter().chain(Some(&style_tag)));
                            self.feed_paper(style.margin_top)?;
                        }
                        Display::Inline => {}
                    }
                    match tag {
                        Tag::List(_) => {
                            // Handle List within list by adding appropriate margin
                            if state.last_list().is_some() {
                                let style = styles.calc_tags(state.style_tags()?.iter());
                                self.feed_paper(style.margin_bottom)?;
                            }
                        }
                        Tag::Item => match state.last_list() {
                            Some(Tag::List(None)) => {
                                let depth = state.list_depth() - 1;
                                let symbol = styles.list_style.get_symbol(depth);
                                let left_margin = styles.list_style.get_left_margin(depth);
                                self.left_margin(left_margin as u16)?;
                                self.bold(true)?;
                                self.print(symbol)?;
                                self.print(" ")?;
                                self.bold(false)?;
                            }
                            Some(Tag::List(Some(_))) => {
                                // TODO implemente ordered lists
                                unimplemented!()
                            }
                            _ => return Err(Error::UnexpectedTag(tag.to_static())),
                        },
                        _ => {}
                    }
                    state.push_tag(tag.clone());
                }
                Event::End(tag) => {
                    match tag.style_tag()?.display() {
                        Display::Block => {
                            let style = styles.calc_tags(state.style_tags()?.iter());
                            self.feed_paper(style.margin_bottom)?;
                        }
                        Display::Inline => {}
                    }
                    let popped = state.pop_tag(&tag);
                    if popped.is_none() {
                        return Err(Error::UnexpectedTag(tag.to_static()));
                    }
                }
                Event::Text(text) => {
                    let style_tag = state
                        .last_tag()
                        .ok_or_else(|| Error::EmptyRenderTree)?
                        .style_tag()?;
                    let style = styles.calc_tags(state.style_tags()?.iter());
                    self.style(&style, &style_tag.display())?;
                    self.print(text)?;
                }
                _ => unimplemented!(),
            }
        }
        Ok(self)
    }
}
