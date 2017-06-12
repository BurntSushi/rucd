// This module defines various common things used throughout the UCD.

use std::char;
use std::collections::BTreeMap;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use error::{Error, error_set_line};

/// Parse a particular file in the UCD into a map from codepoint to the record.
///
/// The given directory should be the directory to the UCD.
pub fn parse_by_codepoint<P, D>(
    ucd_dir: P,
) -> Result<BTreeMap<Codepoint, D>, Error>
where P: AsRef<Path>, D: UcdFileByCodepoint
{
    let mut map = BTreeMap::new();
    for result in D::from_dir(ucd_dir)? {
        let x = result?;
        map.insert(x.codepoint(), x);
    }
    Ok(map)
}

/// Parse a particular file in the UCD into a map from codepoint to all
/// records associated with that codepoint.
///
/// This is useful for files that have multiple records for each codepoint.
/// For example, the `NameAliases.txt` file lists multiple aliases for some
/// codepoints.
///
/// The given directory should be the directory to the UCD.
pub fn parse_many_by_codepoint<P, D>(
    ucd_dir: P,
) -> Result<BTreeMap<Codepoint, Vec<D>>, Error>
where P: AsRef<Path>, D: UcdFileByCodepoint
{
    let mut map = BTreeMap::new();
    for result in D::from_dir(ucd_dir)? {
        let x = result?;
        map.entry(x.codepoint()).or_insert(vec![]).push(x);
    }
    Ok(map)
}

/// A trait that describes a single UCD file.
pub trait UcdFile: fmt::Debug + Default + Eq + FromStr<Err=Error> + PartialEq {
    /// The file path corresponding to this file, relative to the UCD
    /// directory.
    fn relative_file_path() -> &'static Path;

    /// The full file path corresponding to this file given the UCD directory
    /// path.
    fn file_path<P: AsRef<Path>>(ucd_dir: P) -> PathBuf {
        ucd_dir.as_ref().join(Self::relative_file_path())
    }

    /// Create an iterator over each record in this UCD file.
    ///
    /// The parameter should correspond to the directory containing the UCD.
    fn from_dir<P: AsRef<Path>>(
        ucd_dir: P,
    ) -> Result<UcdLineParser<File, Self>, Error> {
        UcdLineParser::from_path(Self::file_path(ucd_dir))
    }
}

/// A trait that describes a single UCD file where every record in the file
/// has a single codepoint associated with it.
pub trait UcdFileByCodepoint: UcdFile {
    /// Returns the codepoint associated with this record.
    fn codepoint(&self) -> Codepoint;
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
