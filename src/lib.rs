#[macro_use]
extern crate thiserror;

pub extern crate pulldown_cmark;

pub mod command;
pub mod config;
mod error;
pub mod instruction;
mod markdown;
mod printer;
mod split_words;
pub mod style;
mod tag_ext;

pub use config::PrinterConfig;
pub use error::{Error, Result};
pub use printer::{Printer, PrinterDevice};
pub use pulldown_cmark::{Options as MarkdownParserOptions, Parser as MarkdownParser};
