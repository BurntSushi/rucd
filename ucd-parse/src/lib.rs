/*!
A library for parsing the Unicode character database.
*/

#![deny(missing_docs)]

#[macro_use]
extern crate lazy_static;
extern crate regex;

pub use common::{Codepoint, UcdLineDatum, UcdLineParser};
pub use error::{Error, ErrorKind};

// pub use jamo_short_name::{JamoShortNameParser, JamoShortName};
pub use jamo_short_name::{JamoShortName};
pub use unicode_data::{
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

mod jamo_short_name;
mod unicode_data;
