use std::borrow::Cow;
use std::path::Path;
use std::str::FromStr;

use regex::Regex;

use common::{UcdFile, UcdFileByCodepoint, Codepoint};
use error::Error;

/// A single row in the `Jamo.txt` file.
///
/// The `Jamo.txt` file defines the `Jamo_Short_Name` property.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct JamoShortName<'a> {
    /// The codepoint corresponding to this row.
    pub codepoint: Codepoint,
    /// The actual "Jamo Short Name." This string contains at most 3 bytes and
    /// may be empty.
    pub name: Cow<'a, str>,
}

impl UcdFile for JamoShortName<'static> {
    fn relative_file_path() -> &'static Path {
        Path::new("Jamo.txt")
    }
}

impl UcdFileByCodepoint for JamoShortName<'static> {
    fn codepoint(&self) -> Codepoint {
        self.codepoint
    }
}

impl<'a> JamoShortName<'a> {
    /// Convert this record into an owned value such that it no longer
    /// borrows from the original line that it was parsed from.
    pub fn into_owned(self) -> JamoShortName<'static> {
        JamoShortName {
            codepoint: self.codepoint,
            name: Cow::Owned(self.name.into_owned()),
        }
    }

    /// Parse a single line.
    pub fn parse_line(line: &'a str) -> Result<JamoShortName<'a>, Error> {
        lazy_static! {
            static ref PARTS: Regex = Regex::new(
                r"(?x)
                ^
                (?P<codepoint>[A-Z0-9]+);
                \s*
                (?P<name>[A-Z]*)
                "
            ).unwrap();
        };

        let caps = match PARTS.captures(line.trim()) {
            Some(caps) => caps,
            None => return err!("invalid Jamo_Short_name line"),
        };
        Ok(JamoShortName {
            codepoint: caps["codepoint"].parse()?,
            name: Cow::Borrowed(caps.name("name").unwrap().as_str()),
        })
    }
}

impl FromStr for JamoShortName<'static> {
    type Err = Error;

    fn from_str(s: &str) -> Result<JamoShortName<'static>, Error> {
        JamoShortName::parse_line(s).map(|x| x.into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::JamoShortName;

    #[test]
    fn parse1() {
        let line = "1164; YAE # HANGUL JUNGSEONG YAE\n";
        let row: JamoShortName = line.parse().unwrap();
        assert_eq!(row.codepoint, 0x1164);
        assert_eq!(row.name, "YAE");
    }

    #[test]
    fn parse2() {
        let line = "110B;     # HANGUL CHOSEONG IEUNG\n";
        let row: JamoShortName = line.parse().unwrap();
        assert_eq!(row.codepoint, 0x110B);
        assert_eq!(row.name, "");
    }
}
