use std::borrow::Cow;
use std::path::Path;
use std::str::FromStr;

use common::{
    UcdFile, UcdFileByCodepoint, Codepoints, CodepointIter,
    parse_codepoint_association,
};
use error::Error;

/// A single row in the `Scripts.txt` file.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Script<'a> {
    /// The codepoint or codepoint range for this entry.
    codepoints: Codepoints,
    /// The script name assigned to the codepoints in this entry.
    script: Cow<'a, str>,
}

impl UcdFile for Script<'static> {
    fn relative_file_path() -> &'static Path {
        Path::new("Scripts.txt")
    }
}

impl UcdFileByCodepoint for Script<'static> {
    fn codepoints(&self) -> CodepointIter {
        self.codepoints.into_iter()
    }
}

impl<'a> Script<'a> {
    /// Convert this record into an owned value such that it no longer
    /// borrows from the original line that it was parsed from.
    pub fn into_owned(self) -> Script<'static> {
        Script {
            codepoints: self.codepoints,
            script: Cow::Owned(self.script.into_owned()),
        }
    }

    /// Parse a single line.
    pub fn parse_line(line: &'a str) -> Result<Script<'a>, Error> {
        let (codepoints, script) = parse_codepoint_association(line)?;
        Ok(Script {
            codepoints: codepoints,
            script: Cow::Borrowed(script),
        })
    }
}

impl FromStr for Script<'static> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Script<'static>, Error> {
        Script::parse_line(s).map(|x| x.into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::Script;

    #[test]
    fn parse_single() {
        let line = "10A7F         ; Old_South_Arabian # Po       OLD SOUTH ARABIAN NUMERIC INDICATOR\n";
        let row: Script = line.parse().unwrap();
        assert_eq!(row.codepoints, 0x10A7F);
        assert_eq!(row.script, "Old_South_Arabian");
    }

    #[test]
    fn parse_range() {
        let line = "1200..1248    ; Ethiopic # Lo  [73] ETHIOPIC SYLLABLE HA..ETHIOPIC SYLLABLE QWA\n";
        let row: Script = line.parse().unwrap();
        assert_eq!(row.codepoints, (0x1200, 0x1248));
        assert_eq!(row.script, "Ethiopic");
    }
}
