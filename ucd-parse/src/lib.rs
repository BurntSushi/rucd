/*!
A library for parsing the Unicode character database.
*/

#![deny(missing_docs)]

#[macro_use]
extern crate lazy_static;
extern crate regex;

pub use common::Codepoint;
pub use error::{Error, ErrorKind};

pub use unicode_data::{
    UnicodeDataParser,
    UnicodeData, UnicodeDataNumeric,
    UnicodeDataDecomposition, UnicodeDataDecompositionTag,
};

macro_rules! err {
    ($($tt:tt)*) => {
        Err(::error::error_parse(format!($($tt)*)))
    }
}

mod common;
mod error;

mod unicode_data;
