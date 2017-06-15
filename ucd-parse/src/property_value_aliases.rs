use std::borrow::Cow;
use std::path::Path;
use std::str::FromStr;

use regex::Regex;

use common::UcdFile;
use error::Error;

/// A single row in the `PropertyValueAliases.txt` file.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PropertyValueAlias<'a> {
    /// The property name for which this value alias applies.
    pub property: Cow<'a, str>,
    /// A numeric abbreviation for this property value, if present. (This is
    /// seemingly only present for the `ccc`/`Canonical_Combining_Class`
    /// property.)
    pub numeric: Option<u8>,
    /// An abbreviation for this property value.
    pub abbreviation: Cow<'a, str>,
    /// The "long" form of this property value.
    pub long: Cow<'a, str>,
    /// Additional value aliases (if present).
    pub aliases: Vec<Cow<'a, str>>,
}

impl UcdFile for PropertyValueAlias<'static> {
    fn relative_file_path() -> &'static Path {
        Path::new("PropertyValueAliases.txt")
    }
}

impl<'a> PropertyValueAlias<'a> {
    /// Convert this record into an owned value such that it no longer
    /// borrows from the original line that it was parsed from.
    pub fn into_owned(self) -> PropertyValueAlias<'static> {
        let aliases = self.aliases.into_iter()
            .map(|x| Cow::Owned(x.into_owned()))
            .collect();
        PropertyValueAlias {
            property: Cow::Owned(self.property.into_owned()),
            numeric: self.numeric,
            abbreviation: Cow::Owned(self.abbreviation.into_owned()),
            long: Cow::Owned(self.long.into_owned()),
            aliases: aliases,
        }
    }

    /// Parse a single line.
    pub fn parse_line(line: &'a str) -> Result<PropertyValueAlias<'a>, Error> {
        lazy_static! {
            static ref PARTS: Regex = Regex::new(
                r"(?x)
                ^
                \s*(?P<prop>[^\s;]+)\s*;
                \s*(?P<abbrev>[^\s;]+)\s*;
                \s*(?P<long>[^\s;]+)\s*
                (?:;(?P<aliases>.*))?
                "
            ).unwrap();
            static ref PARTS_CCC: Regex = Regex::new(
                r"(?x)
                ^
                ccc;
                \s*(?P<num_class>[0-9]+)\s*;
                \s*(?P<abbrev>[^\s;]+)\s*;
                \s*(?P<long>[^\s;]+)
                "
            ).unwrap();
            static ref ALIASES: Regex = Regex::new(
                r"\s*(?P<alias>[^\s;]+)\s*;?\s*"
            ).unwrap();
        };

        if line.starts_with("ccc;") {
            let caps = match PARTS_CCC.captures(line.trim()) {
                Some(caps) => caps,
                None => return err!("invalid PropertyValueAliases (ccc) line"),
            };
            let n = match caps["num_class"].parse() {
                Ok(n) => n,
                Err(err) => return err!(
                    "failed to parse ccc number '{}': {}",
                    &caps["num_class"], err),
            };
            let abbrev = caps.name("abbrev").unwrap().as_str();
            let long = caps.name("long").unwrap().as_str();
            return Ok(PropertyValueAlias {
                property: Cow::Borrowed(&line[0..3]),
                numeric: Some(n),
                abbreviation: Cow::Borrowed(abbrev),
                long: Cow::Borrowed(long),
                aliases: vec![],
            });
        }

        let caps = match PARTS.captures(line.trim()) {
            Some(caps) => caps,
            None => return err!("invalid PropertyValueAliases line"),
        };
        let mut aliases = vec![];
        if let Some(m) = caps.name("aliases") {
            for acaps in ALIASES.captures_iter(m.as_str()) {
                let alias = acaps.name("alias").unwrap().as_str();
                aliases.push(Cow::Borrowed(alias));
            }
        }
        Ok(PropertyValueAlias {
            property: Cow::Borrowed(caps.name("prop").unwrap().as_str()),
            numeric: None,
            abbreviation: Cow::Borrowed(caps.name("abbrev").unwrap().as_str()),
            long: Cow::Borrowed(caps.name("long").unwrap().as_str()),
            aliases: aliases,
        })
    }
}

impl FromStr for PropertyValueAlias<'static> {
    type Err = Error;

    fn from_str(s: &str) -> Result<PropertyValueAlias<'static>, Error> {
        PropertyValueAlias::parse_line(s).map(|x| x.into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::PropertyValueAlias;

    #[test]
    fn parse1() {
        let line = "blk; Arabic_PF_A                      ; Arabic_Presentation_Forms_A      ; Arabic_Presentation_Forms-A\n";
        let row: PropertyValueAlias = line.parse().unwrap();
        assert_eq!(row.property, "blk");
        assert_eq!(row.numeric, None);
        assert_eq!(row.abbreviation, "Arabic_PF_A");
        assert_eq!(row.long, "Arabic_Presentation_Forms_A");
        assert_eq!(row.aliases, vec!["Arabic_Presentation_Forms-A"]);
    }

    #[test]
    fn parse2() {
        let line = "AHex; N                               ; No                               ; F                                ; False\n";
        let row: PropertyValueAlias = line.parse().unwrap();
        assert_eq!(row.property, "AHex");
        assert_eq!(row.numeric, None);
        assert_eq!(row.abbreviation, "N");
        assert_eq!(row.long, "No");
        assert_eq!(row.aliases, vec!["F", "False"]);
    }

    #[test]
    fn parse3() {
        let line = "age; 1.1                              ; V1_1\n";
        let row: PropertyValueAlias = line.parse().unwrap();
        assert_eq!(row.property, "age");
        assert_eq!(row.numeric, None);
        assert_eq!(row.abbreviation, "1.1");
        assert_eq!(row.long, "V1_1");
        assert!(row.aliases.is_empty());
    }

    #[test]
    fn parse4() {
        let line = "ccc;   0; NR                         ; Not_Reordered\n";
        let row: PropertyValueAlias = line.parse().unwrap();
        assert_eq!(row.property, "ccc");
        assert_eq!(row.numeric, Some(0));
        assert_eq!(row.abbreviation, "NR");
        assert_eq!(row.long, "Not_Reordered");
        assert!(row.aliases.is_empty());
    }

    #[test]
    fn parse5() {
        let line = "ccc; 133; CCC133                     ; CCC133 # RESERVED\n";
        let row: PropertyValueAlias = line.parse().unwrap();
        assert_eq!(row.property, "ccc");
        assert_eq!(row.numeric, Some(133));
        assert_eq!(row.abbreviation, "CCC133");
        assert_eq!(row.long, "CCC133");
        assert!(row.aliases.is_empty());
    }
}
