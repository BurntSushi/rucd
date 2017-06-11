// This module defines various common things used throughout the UCD.

use std::char;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};
use std::marker::PhantomData;
use std::path::Path;
use std::str::FromStr;

use error::{Error, error_set_line};

/// A simple interface for parsing a single datum from a single line in a
/// UCD file.
pub trait UcdLineDatum<'a>: Default {
    /// Parse a single line in a UCD file into a single datum.
    fn parse_line(line: &'a str) -> Result<Self, Error>;
}

/// A line oriented parser for a particular UCD file.
///
/// The `R` type parameter refers to the underlying `io::Read` implementation
/// from which the UCD data is read.
#[derive(Debug)]
pub struct UcdLineParser<R, D> {
    rdr: io::BufReader<R>,
    line: String,
    line_number: u64,
    _data: PhantomData<D>,
}

impl<D> UcdLineParser<File, D> {
    /// Create a new parser from the given file path.
    pub fn from_path<P: AsRef<Path>>(
        path: P,
    ) -> Result<UcdLineParser<File, D>, Error> {
        let file = File::open(path)?;
        Ok(UcdLineParser::new(file))
    }
}

impl<R: io::Read, D> UcdLineParser<R, D> {
    /// Create a new parser that parses the reader given.
    ///
    /// The type of data parsed is determined when the `parse_next` function
    /// is called by virtue of the type requested.
    ///
    /// Note that the reader is buffered internally, so the caller does not
    /// need to provide their own buffering.
    pub fn new(rdr: R) -> UcdLineParser<R, D> {
        UcdLineParser {
            rdr: io::BufReader::new(rdr),
            line: String::new(),
            line_number: 0,
            _data: PhantomData,
        }
    }
}

impl<R: io::Read, D: FromStr<Err=Error>> Iterator for UcdLineParser<R, D> {
    type Item = Result<D, Error>;

    fn next(&mut self) -> Option<Result<D, Error>> {
        loop {
            self.line_number += 1;
            self.line.clear();
            let n = match self.rdr.read_line(&mut self.line) {
                Err(err) => return Some(Err(Error::from(err))),
                Ok(n) => n,
            };
            if n == 0 {
                return None;
            }
            if !self.line.starts_with('#') && !self.line.trim().is_empty() {
                break;
            }
        }
        let line_number = self.line_number;
        Some(self.line.parse().map_err(|mut err| {
            error_set_line(&mut err, Some(line_number));
            err
        }))
    }
}

/// A single Unicode codepoint.
///
/// This type's string representation is a hexadecimal number. It is guaranteed
/// to be in the range `[0, 10FFFF]`.
///
/// Note that unlike Rust's `char` type, this may be a surrogate codepoint.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Codepoint(u32);

impl Codepoint {
    /// Create a new codepoint from a `u32`.
    ///
    /// If the given number is not a valid codepoint, then this returns an
    /// error.
    pub fn from_u32(n: u32) -> Result<Codepoint, Error> {
        if n > 0x10FFFF {
            err!("{:x} is not a valid Unicode codepoint", n)
        } else {
            Ok(Codepoint(n))
        }
    }

    /// Return the underlying `u32` codepoint value.
    pub fn value(self) -> u32 { self.0 }

    /// Attempt to convert this codepoint to a Unicode scalar value.
    ///
    /// If this is a surrogate codepoint, then this returns `None`.
    pub fn scalar(self) -> Option<char> { char::from_u32(self.0) }
}

impl FromStr for Codepoint {
    type Err = Error;

    fn from_str(s: &str) -> Result<Codepoint, Error> {
        match u32::from_str_radix(s, 16) {
            Ok(n) => Codepoint::from_u32(n),
            Err(err) => {
                return err!(
                    "failed to parse '{}' as a hexadecimal codepoint: {}",
                    s, err);
            }
        }
    }
}

impl fmt::Display for Codepoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04X}", self.0)
    }
}

impl PartialEq<u32> for Codepoint {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

impl PartialEq<Codepoint> for u32 {
    fn eq(&self, other: &Codepoint) -> bool {
        *self == other.0
    }
}
