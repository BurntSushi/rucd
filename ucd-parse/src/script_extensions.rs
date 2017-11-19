use std::borrow::Cow;
use std::path::Path;
use std::str::FromStr;

use common::{
    UcdFile, UcdFileByCodepoint, Codepoints, CodepointIter,
    parse_codepoint_association,
};
use error::Error;

/// A single row in the `ScriptExtensions.txt` file.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ScriptExtension<'a> {
    /// The codepoint or codepoint range for this entry.
    codepoints: Codepoints,
    /// The script extension names assigned to the codepoints in this entry.
    scripts: Vec<Cow<'a, str>>,
}

impl UcdFile for ScriptExtension<'static> {
    fn relative_file_path() -> &'static Path {
        Path::new("ScriptExtensions.txt")
    }
}

impl UcdFileByCodepoint for ScriptExtension<'static> {
    fn codepoints(&self) -> CodepointIter {
        self.codepoints.into_iter()
    }
}

impl<'a> ScriptExtension<'a> {
    /// Convert this record into an owned value such that it no longer
    /// borrows from the original line that it was parsed from.
    pub fn into_owned(self) -> ScriptExtension<'static> {
        let scripts = self.scripts.into_iter()
            .map(|x| Cow::Owned(x.into_owned()))
            .collect();
        ScriptExtension {
            codepoints: self.codepoints,
            scripts: scripts,
        }
    }

    /// Parse a single line.
    pub fn parse_line(line: &'a str) -> Result<ScriptExtension<'a>, Error> {
        let (codepoints, scripts) = parse_codepoint_association(line)?;
        Ok(ScriptExtension {
            codepoints: codepoints,
            scripts: scripts.split_whitespace().map(Cow::Borrowed).collect(),
        })
    }
}

impl FromStr for ScriptExtension<'static> {
    type Err = Error;

    fn from_str(s: &str) -> Result<ScriptExtension<'static>, Error> {
        ScriptExtension::parse_line(s).map(|x| x.into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::ScriptExtension;

    #[test]
    fn parse_single() {
        let line = "060C          ; Arab Syrc Thaa # Po       ARABIC COMMA\n";
        let row: ScriptExtension = line.parse().unwrap();
        assert_eq!(row.codepoints, 0x060C);
        assert_eq!(row.scripts, vec!["Arab", "Syrc", "Thaa"]);
    }

    #[test]
    fn parse_range() {
        let line = "A836..A837    ; Deva Gujr Guru Kthi Mahj Modi Sind Takr Tirh # So   [2] NORTH INDIC QUARTER MARK..NORTH INDIC PLACEHOLDER MARK\n";
        let row: ScriptExtension = line.parse().unwrap();
        assert_eq!(row.codepoints, (0xA836, 0xA837));
        assert_eq!(row.scripts, vec![
            "Deva", "Gujr", "Guru", "Kthi", "Mahj", "Modi", "Sind", "Takr",
            "Tirh",
        ]);
    }
}
