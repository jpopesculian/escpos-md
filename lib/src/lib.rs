#[macro_use]
extern crate thiserror;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;

pub extern crate pulldown_cmark;

pub mod command;
pub mod config;
mod error;
pub mod instruction;
mod markdown;
mod printer;
mod pulldown_cmark_ext;
mod split_words;
pub mod style;

pub use config::PrinterConfig;
pub use error::{Error, Result};
pub use markdown::MarkdownRenderOptions;
pub use printer::{Printer, PrinterDevice};
pub use pulldown_cmark::{Options as MarkdownParserOptions, Parser as MarkdownParser};
