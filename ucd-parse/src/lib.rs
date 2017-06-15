/*!
A library for parsing the Unicode character database.
*/

#![deny(missing_docs)]

#[macro_use]
extern crate lazy_static;
extern crate regex;

pub use common::{
    UcdFile, Codepoint, UcdLineParser,
    parse, parse_by_codepoint, parse_many_by_codepoint,
};
pub use error::{Error, ErrorKind};

pub use jamo_short_name::JamoShortName;
pub use name_aliases::{NameAlias, NameAliasLabel};
pub use property_aliases::PropertyAlias;
pub use property_value_aliases::PropertyValueAlias;
pub use unicode_data::{
    UnicodeData, UnicodeDataNumeric,
    UnicodeDataDecomposition, UnicodeDataDecompositionTag,
    UnicodeDataExpander,
};

macro_rules! err {
    ($($tt:tt)*) => {
        Err(::error::error_parse(format!($($tt)*)))
    }
}

mod common;
mod error;

mod jamo_short_name;
mod name_aliases;
mod property_aliases;
mod property_value_aliases;
mod unicode_data;
