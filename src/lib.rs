#[macro_use]
extern crate thiserror;

pub mod command;
pub mod config;
mod error;
pub mod instruction;
mod printer;

pub use error::{Error, Result};
pub use printer::{Printer, PrinterDevice};
