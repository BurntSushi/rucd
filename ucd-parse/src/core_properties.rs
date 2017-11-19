use std::borrow::Cow;
use std::path::Path;
use std::str::FromStr;

use common::{
    UcdFile, UcdFileByCodepoint, Codepoints, CodepointIter,
    parse_codepoint_association,
};
use error::Error;

/// A single row in the `DerivedCoreProperties.txt` file.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CoreProperty<'a> {
    /// The codepoint or codepoint range for this entry.
    codepoints: Codepoints,
    /// The property name assigned to the codepoints in this entry.
    property: Cow<'a, str>,
}

impl UcdFile for CoreProperty<'static> {
    fn relative_file_path() -> &'static Path {
        Path::new("DerivedCoreProperties.txt")
    }
}

impl UcdFileByCodepoint for CoreProperty<'static> {
    fn codepoints(&self) -> CodepointIter {
        self.codepoints.into_iter()
    }
}

impl<'a> CoreProperty<'a> {
    /// Convert this record into an owned value such that it no longer
    /// borrows from the original line that it was parsed from.
    pub fn into_owned(self) -> CoreProperty<'static> {
        CoreProperty {
            codepoints: self.codepoints,
            property: Cow::Owned(self.property.into_owned()),
        }
    }

    /// Parse a single line.
    pub fn parse_line(line: &'a str) -> Result<CoreProperty<'a>, Error> {
        let (codepoints, property) = parse_codepoint_association(line)?;
        Ok(CoreProperty {
            codepoints: codepoints,
            property: Cow::Borrowed(property),
        })
    }
}

impl FromStr for CoreProperty<'static> {
    type Err = Error;

    fn from_str(s: &str) -> Result<CoreProperty<'static>, Error> {
        CoreProperty::parse_line(s).map(|x| x.into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::CoreProperty;

    #[test]
    fn parse_single() {
        let line = "1163D         ; Case_Ignorable # Mn       MODI SIGN ANUSVARA\n";
        let row: CoreProperty = line.parse().unwrap();
        assert_eq!(row.codepoints, 0x1163D);
        assert_eq!(row.property, "Case_Ignorable");
    }

    #[test]
    fn parse_range() {
        let line = "11133..11134  ; Grapheme_Link # Mn   [2] CHAKMA VIRAMA..CHAKMA MAAYYAA\n";
        let row: CoreProperty = line.parse().unwrap();
        assert_eq!(row.codepoints, (0x11133, 0x11134));
        assert_eq!(row.property, "Grapheme_Link");
    }
}
