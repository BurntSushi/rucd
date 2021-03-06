use std::char;
use std::fmt;
use std::str::FromStr;

use error::{Error, Result};

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
    pub fn from_u32(n: u32) -> Result<Codepoint> {
        if n > 0x10FFFF {
            err!("{:x} is not a valid Unicode codepoint", n)
        } else {
            Ok(Codepoint(n))
        }
    }

    /// Create a new codepoint from a `u64` such that only the least
    /// significant 32 bits are used.
    pub fn from_u64(n: u64) -> Result<Codepoint> {
        Codepoint::from_u32((n & 0xFFFFFFFF) as u32)
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

    fn from_str(s: &str) -> Result<Codepoint> {
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
