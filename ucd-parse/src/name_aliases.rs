use std::borrow::Cow;
use std::path::Path;
use std::str::FromStr;

use regex::Regex;

use common::{UcdFile, UcdFileByCodepoint, Codepoint};
use error::Error;

/// A single row in the `NameAliases.txt` file.
///
/// Note that there are multiple rows for some codepoint. Each row provides a
/// new alias.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NameAlias<'a> {
    /// The codepoint corresponding to this row.
    pub codepoint: Codepoint,
    /// The alias.
    pub alias: Cow<'a, str>,
    /// The label of this alias.
    pub label: NameAliasLabel,
}

impl UcdFile for NameAlias<'static> {
    fn relative_file_path() -> &'static Path {
        Path::new("NameAliases.txt")
    }
}

impl UcdFileByCodepoint for NameAlias<'static> {
    fn codepoint(&self) -> Codepoint {
        self.codepoint
    }
}

impl<'a> NameAlias<'a> {
    /// Convert this record into an owned value such that it no longer
    /// borrows from the original line that it was parsed from.
    pub fn into_owned(self) -> NameAlias<'static> {
        NameAlias {
            codepoint: self.codepoint,
            alias: Cow::Owned(self.alias.into_owned()),
            label: self.label,
        }
    }

    /// Parse a single line.
    pub fn parse_line(line: &'a str) -> Result<NameAlias<'a>, Error> {
        lazy_static! {
            static ref PARTS: Regex = Regex::new(
                r"(?x)
                ^
                (?P<codepoint>[A-Z0-9]+);
                \s*
                (?P<alias>[^;]+);
                \s*
                (?P<label>\S+)
                "
            ).unwrap();
        };

        let caps = match PARTS.captures(line.trim()) {
            Some(caps) => caps,
            None => return err!("invalid NameAliases line"),
        };
        Ok(NameAlias {
            codepoint: caps["codepoint"].parse()?,
            alias: Cow::Borrowed(caps.name("alias").unwrap().as_str()),
            label: caps["label"].parse()?,
        })
    }
}

impl FromStr for NameAlias<'static> {
    type Err = Error;

    fn from_str(s: &str) -> Result<NameAlias<'static>, Error> {
        NameAlias::parse_line(s).map(|x| x.into_owned())
    }
}

/// The label of a name alias.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NameAliasLabel {
    /// Corrections for serious problems in a character name.
    Correction,
    /// ISO 6429 names for C0 and C1 control functions and other commonly
    /// occurring names for control codes.
    Control,
    /// A few widely used alternate names for format characters.
    Alternate,
    /// Several documented labels for C1 control code points which were
    /// never actually approved in any standard.
    Figment,
    /// Commonly occurring abbreviations (or acronyms) for control codes,
    /// format characters, spaces and variation selectors.
    Abbreviation,
}

impl Default for NameAliasLabel {
    fn default() -> NameAliasLabel {
        // This is arbitrary, but the Default impl is convenient.
        NameAliasLabel::Correction
    }
}

impl FromStr for NameAliasLabel {
    type Err = Error;

    fn from_str(s: &str) -> Result<NameAliasLabel, Error> {
        match s {
            "correction" => Ok(NameAliasLabel::Correction),
            "control" => Ok(NameAliasLabel::Control),
            "alternate" => Ok(NameAliasLabel::Alternate),
            "figment" => Ok(NameAliasLabel::Figment),
            "abbreviation" => Ok(NameAliasLabel::Abbreviation),
            unknown => err!("unknown name alias label: '{}'", unknown),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NameAlias, NameAliasLabel};

    #[test]
    fn parse1() {
        let line = "0000;NULL;control\n";
        let row: NameAlias = line.parse().unwrap();
        assert_eq!(row.codepoint, 0x0);
        assert_eq!(row.alias, "NULL");
        assert_eq!(row.label, NameAliasLabel::Control);
    }

    #[test]
    fn parse2() {
        let line = "000B;VERTICAL TABULATION;control\n";
        let row: NameAlias = line.parse().unwrap();
        assert_eq!(row.codepoint, 0xB);
        assert_eq!(row.alias, "VERTICAL TABULATION");
        assert_eq!(row.label, NameAliasLabel::Control);
    }

    #[test]
    fn parse3() {
        let line = "0081;HIGH OCTET PRESET;figment\n";
        let row: NameAlias = line.parse().unwrap();
        assert_eq!(row.codepoint, 0x81);
        assert_eq!(row.alias, "HIGH OCTET PRESET");
        assert_eq!(row.label, NameAliasLabel::Figment);
    }

    #[test]
    fn parse4() {
        let line = "E01EF;VS256;abbreviation\n";
        let row: NameAlias = line.parse().unwrap();
        assert_eq!(row.codepoint, 0xE01EF);
        assert_eq!(row.alias, "VS256");
        assert_eq!(row.label, NameAliasLabel::Abbreviation);
    }
}
