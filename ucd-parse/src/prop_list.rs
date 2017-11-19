use std::borrow::Cow;
use std::path::Path;
use std::str::FromStr;

use common::{
    UcdFile, UcdFileByCodepoint, Codepoints, CodepointIter,
    parse_codepoint_association,
};
use error::Error;

/// A single row in the `PropList.txt` file.
///
/// The `PropList.txt` file is the source of truth on several Unicode
/// properties.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Property<'a> {
    /// The codepoint or codepoint range for this entry.
    codepoints: Codepoints,
    /// The property name assigned to the codepoints in this entry.
    property: Cow<'a, str>,
}

impl UcdFile for Property<'static> {
    fn relative_file_path() -> &'static Path {
        Path::new("PropList.txt")
    }
}

impl UcdFileByCodepoint for Property<'static> {
    fn codepoints(&self) -> CodepointIter {
        self.codepoints.into_iter()
    }
}

impl<'a> Property<'a> {
    /// Convert this record into an owned value such that it no longer
    /// borrows from the original line that it was parsed from.
    pub fn into_owned(self) -> Property<'static> {
        Property {
            codepoints: self.codepoints,
            property: Cow::Owned(self.property.into_owned()),
        }
    }

    /// Parse a single line.
    pub fn parse_line(line: &'a str) -> Result<Property<'a>, Error> {
        let (codepoints, property) = parse_codepoint_association(line)?;
        Ok(Property {
            codepoints: codepoints,
            property: Cow::Borrowed(property),
        })
    }
}

impl FromStr for Property<'static> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Property<'static>, Error> {
        Property::parse_line(s).map(|x| x.into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::Property;

    #[test]
    fn parse_single() {
        let line = "061C          ; Bidi_Control # Cf       ARABIC LETTER MARK\n";
        let row: Property = line.parse().unwrap();
        assert_eq!(row.codepoints, 0x061C);
        assert_eq!(row.property, "Bidi_Control");
    }

    #[test]
    fn parse_range() {
        let line = "0009..000D    ; White_Space # Cc   [5] <control-0009>..<control-000D>\n";
        let row: Property = line.parse().unwrap();
        assert_eq!(row.codepoints, (0x0009, 0x000D));
        assert_eq!(row.property, "White_Space");
    }
}
