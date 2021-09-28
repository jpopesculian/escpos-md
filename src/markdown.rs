use crate::error::{Error, Result};
use crate::instruction::{EscposImage, ImageOptions};
use crate::printer::{Printer, PrinterDevice};
use crate::pulldown_cmark_ext::{EventExt, TagExt};
use crate::style::{StyleSheet, StyleTag};
use pulldown_cmark::{Event, Tag};

#[derive(Debug, Clone, Default)]
pub struct MarkdownRenderOptions {
    styles: StyleSheet,
    image: ImageOptions,
}

#[derive(Debug, Clone, Copy)]
pub enum TagState {
    Stateless,
    Item(u64),
}

impl TagState {
    pub fn num(&self) -> Option<u64> {
        match self {
            Self::Stateless => None,
            Self::Item(num) => Some(*num),
        }
    }
}

#[derive(Default)]
struct RendererState<'a> {
    tree: Vec<(Tag<'a>, TagState)>,
}

impl<'a> RendererState<'a> {
    fn push_tag(&mut self, tag: Tag<'a>) -> Result<()> {
        let state = match tag {
            Tag::List(Some(num)) => TagState::Item(num),
            Tag::Item => {
                match self
                    .tag_state_mut()
                    .ok_or_else(|| Error::UnexpectedTag(tag.clone().to_static()))?
                {
                    TagState::Stateless => TagState::Stateless,
                    TagState::Item(num) => {
                        let state = TagState::Item(*num);
                        *num += 1;
                        state
                    }
                }
            }
            _ => TagState::Stateless,
        };
        self.tree.push((tag, state));
        Ok(())
    }

    fn pop_tag(&mut self, tag: &Tag<'a>) -> Result<()> {
        if self.tag() != Some(tag) {
            Err(Error::UnexpectedTag(tag.clone().to_static()))
        } else {
            self.tree.pop();
            Ok(())
        }
    }

    fn tag(&self) -> Option<&Tag<'a>> {
        self.tree.last().map(|item| &item.0)
    }

    fn tag_state(&self) -> Option<&TagState> {
        self.tree.last().map(|item| &item.1)
    }

    fn tag_state_mut(&mut self) -> Option<&mut TagState> {
        self.tree.last_mut().map(|item| &mut item.1)
    }

    fn style_tags(&self) -> Result<Vec<StyleTag>> {
        self.tree.iter().map(|(tag, _)| tag.style_tag()).collect()
    }
}

impl<D> Printer<D>
where
    D: PrinterDevice,
{
    pub fn markdown<'a, I>(&mut self, iter: I, opts: &MarkdownRenderOptions) -> Result<&mut Self>
    where
        I: Iterator<Item = Event<'a>>,
    {
        let mut state = RendererState::default();
        for event in iter {
            eprintln!("=> {:?}", event);
            match event {
                Event::Start(tag) => {
                    state.push_tag(tag.clone())?;
                    let style_tags = state.style_tags()?;
                    let style = opts.styles.get(&style_tags);
                    self.font_style(&style)?;
                    self.begin_block_style(&style, state.tag_state())?;

                    match tag {
                        Tag::Image(_, filename, _) => {
                            let img = image::open(filename.as_ref())?;
                            let escpos_img = EscposImage::new(&img, &opts.image);
                            self.image(&escpos_img)?;

                            let mut img_caption_tags = style_tags;
                            img_caption_tags.push(StyleTag::ImgCaption);
                            let img_caption_style = opts.styles.get(&img_caption_tags);
                            self.font_style(&img_caption_style)?;
                            self.begin_block_style(&img_caption_style, None)?;
                        }
                        _ => {}
                    }
                }
                Event::End(tag) => {
                    let style_tags = state.style_tags()?;
                    match tag {
                        Tag::Image(..) => {
                            let mut img_caption_tags = style_tags.clone();
                            img_caption_tags.push(StyleTag::ImgCaption);
                            let img_caption_style = opts.styles.get(&img_caption_tags);
                            self.end_block_style(&img_caption_style)?;
                        }
                        _ => {}
                    }
                    let style = opts.styles.get(&style_tags);
                    self.end_block_style(&style)?;
                    state.pop_tag(&tag)?;
                    let style = opts.styles.get(&state.style_tags()?);
                    self.font_style(&style)?;
                }
                Event::Text(text) => {
                    self.print(text)?;
                }
                Event::Code(text) => {
                    let mut style_tags = state.style_tags()?;
                    style_tags.push(StyleTag::Code);
                    let style = opts.styles.get(&style_tags);
                    self.font_style(&style)?;
                    self.begin_block_style(&style, None)?;

                    self.print(text)?;

                    self.end_block_style(&style)?;
                    let style = opts.styles.get(&state.style_tags()?);
                    self.font_style(&style)?;
                }
                Event::SoftBreak => {
                    self.print(" ")?;
                }
                Event::HardBreak => {
                    self.println("")?;
                }
                Event::Rule => {
                    let mut style_tags = state.style_tags()?;
                    style_tags.push(StyleTag::Hr);
                    let style = opts.styles.get(&style_tags);
                    self.font_style(&style)?;
                    self.begin_block_style(&style, None)?;

                    let num_bars = self.printable_width() / self.calc_char_size();
                    self.println(vec!["─"; num_bars].join(""))?;

                    self.end_block_style(&style)?;
                    let style = opts.styles.get(&state.style_tags()?);
                    self.font_style(&style)?;
                }
                event => return Err(Error::MarkdownEventUnimplemented(event.to_static())),
            }
        }
        Ok(self)
    }
}
