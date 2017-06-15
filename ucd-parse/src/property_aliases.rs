use std::borrow::Cow;
use std::path::Path;
use std::str::FromStr;

use regex::Regex;

use common::UcdFile;
use error::Error;

/// A single row in the `PropertyAliases.txt` file.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PropertyAlias<'a> {
    /// An abbreviation for this property.
    pub abbreviation: Cow<'a, str>,
    /// The "long" name of this property.
    pub long: Cow<'a, str>,
    /// Additional aliases (if present).
    pub aliases: Vec<Cow<'a, str>>,
}

impl UcdFile for PropertyAlias<'static> {
    fn relative_file_path() -> &'static Path {
        Path::new("PropertyAliases.txt")
    }
}

impl<'a> PropertyAlias<'a> {
    /// Convert this record into an owned value such that it no longer
    /// borrows from the original line that it was parsed from.
    pub fn into_owned(self) -> PropertyAlias<'static> {
        let aliases = self.aliases.into_iter()
            .map(|x| Cow::Owned(x.into_owned()))
            .collect();
        PropertyAlias {
            abbreviation: Cow::Owned(self.abbreviation.into_owned()),
            long: Cow::Owned(self.long.into_owned()),
            aliases: aliases,
        }
    }

    /// Parse a single line.
    pub fn parse_line(line: &'a str) -> Result<PropertyAlias<'a>, Error> {
        lazy_static! {
            static ref PARTS: Regex = Regex::new(
                r"(?x)
                ^
                \s*(?P<abbrev>[^\s;]+)\s*;
                \s*(?P<long>[^\s;]+)\s*
                (?:;(?P<aliases>.*))?
                "
            ).unwrap();
            static ref ALIASES: Regex = Regex::new(
                r"\s*(?P<alias>[^\s;]+)\s*;?\s*"
            ).unwrap();
        };

        let caps = match PARTS.captures(line.trim()) {
            Some(caps) => caps,
            None => return err!("invalid PropertyAliases line"),
        };
        let mut aliases = vec![];
        if let Some(m) = caps.name("aliases") {
            for acaps in ALIASES.captures_iter(m.as_str()) {
                let alias = acaps.name("alias").unwrap().as_str();
                aliases.push(Cow::Borrowed(alias));
            }
        }
        Ok(PropertyAlias {
            abbreviation: Cow::Borrowed(caps.name("abbrev").unwrap().as_str()),
            long: Cow::Borrowed(caps.name("long").unwrap().as_str()),
            aliases: aliases,
        })
    }
}

impl FromStr for PropertyAlias<'static> {
    type Err = Error;

    fn from_str(s: &str) -> Result<PropertyAlias<'static>, Error> {
        PropertyAlias::parse_line(s).map(|x| x.into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::PropertyAlias;

    #[test]
    fn parse1() {
        let line = "cjkAccountingNumeric     ; kAccountingNumeric\n";
        let row: PropertyAlias = line.parse().unwrap();
        assert_eq!(row.abbreviation, "cjkAccountingNumeric");
        assert_eq!(row.long, "kAccountingNumeric");
        assert!(row.aliases.is_empty());
    }

    #[test]
    fn parse2() {
        let line = "nv                       ; Numeric_Value\n";
        let row: PropertyAlias = line.parse().unwrap();
        assert_eq!(row.abbreviation, "nv");
        assert_eq!(row.long, "Numeric_Value");
        assert!(row.aliases.is_empty());
    }

    #[test]
    fn parse3() {
        let line = "scf                      ; Simple_Case_Folding         ; sfc\n";
        let row: PropertyAlias = line.parse().unwrap();
        assert_eq!(row.abbreviation, "scf");
        assert_eq!(row.long, "Simple_Case_Folding");
        assert_eq!(row.aliases, vec!["sfc"]);
    }

    #[test]
    fn parse4() {
        let line = "cjkRSUnicode             ; kRSUnicode                  ; Unicode_Radical_Stroke; URS\n";
        let row: PropertyAlias = line.parse().unwrap();
        assert_eq!(row.abbreviation, "cjkRSUnicode");
        assert_eq!(row.long, "kRSUnicode");
        assert_eq!(row.aliases, vec!["Unicode_Radical_Stroke", "URS"]);
    }
}
