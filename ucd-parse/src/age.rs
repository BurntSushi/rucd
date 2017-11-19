
use std::borrow::Cow;
use std::path::Path;
use std::str::FromStr;

use common::{
    UcdFile, UcdFileByCodepoint, Codepoints, CodepointIter,
    parse_codepoint_association,
};
use error::Error;

/// A single row in the `DerivedAge.txt` file.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Age<'a> {
    /// The codepoint or codepoint range for this entry.
    codepoints: Codepoints,
    /// The age assigned to the codepoints in this entry.
    age: Cow<'a, str>,
}

impl UcdFile for Age<'static> {
    fn relative_file_path() -> &'static Path {
        Path::new("DerivedAge.txt")
    }
}

impl UcdFileByCodepoint for Age<'static> {
    fn codepoints(&self) -> CodepointIter {
        self.codepoints.into_iter()
    }
}

impl<'a> Age<'a> {
    /// Convert this record into an owned value such that it no longer
    /// borrows from the original line that it was parsed from.
    pub fn into_owned(self) -> Age<'static> {
        Age {
            codepoints: self.codepoints,
            age: Cow::Owned(self.age.into_owned()),
        }
    }

    /// Parse a single line.
    pub fn parse_line(line: &'a str) -> Result<Age<'a>, Error> {
        let (codepoints, script) = parse_codepoint_association(line)?;
        Ok(Age {
            codepoints: codepoints,
            age: Cow::Borrowed(script),
        })
    }
}

impl FromStr for Age<'static> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Age<'static>, Error> {
        Age::parse_line(s).map(|x| x.into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::Age;

    #[test]
    fn parse_single() {
        let line = "2BD2          ; 10.0 #       GROUP MARK\n";
        let row: Age = line.parse().unwrap();
        assert_eq!(row.codepoints, 0x2BD2);
        assert_eq!(row.age, "10.0");
    }

    #[test]
    fn parse_range() {
        let line = "11D0B..11D36  ; 10.0 #  [44] MASARAM GONDI LETTER AU..MASARAM GONDI VOWEL SIGN VOCALIC R\n";
        let row: Age = line.parse().unwrap();
        assert_eq!(row.codepoints, (0x11D0B, 0x11D36));
        assert_eq!(row.age, "10.0");
    }
}
