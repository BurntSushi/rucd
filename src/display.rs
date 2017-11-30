use std::io::{self, Write};

use tabwriter::TabWriter;

use codepoint::Codepoint;
use error::Result;
use name::codepoint_name;

pub struct ShortWriter<W: io::Write> {
    wtr: TabWriter<io::BufWriter<W>>,
    wrote_header: bool,
}

impl<W: io::Write> ShortWriter<W> {
    pub fn new(wtr: W) -> ShortWriter<W> {
        ShortWriter {
            wtr: TabWriter::new(io::BufWriter::new(wtr)),
            wrote_header: false,
        }
    }

    fn write_header(&mut self) -> Result<()> {
        writeln!(self.wtr, "codepoint\tcharacter\tutf8\tname")?;
        self.wrote_header = true;
        Ok(())
    }

    pub fn write_codepoint(&mut self, cp: Codepoint) -> Result<()> {
        if !self.wrote_header {
            self.write_header()?;
        }
        let name = codepoint_name(cp).unwrap_or("".to_string());
        let scalar = nice_char(cp).unwrap_or("".to_string());
        let utf8 = utf8_hex(cp).unwrap_or("".to_string());
        writeln!(
            self.wtr,
            "U+{:04X}\t{}\t{}\t{}",
            cp.value(), scalar, utf8, name)?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.wtr.flush()?;
        Ok(())
    }
}

/// Converts the given codepoint to a human readable hexadecimal string
/// corresponding to its UTF-8 encoding.
///
/// If the codepoint is a surrogate, then `None` is returned.
fn utf8_hex(cp: Codepoint) -> Option<String> {
    let scalar = match cp.scalar() {
        None => return None,
        Some(scalar) => scalar,
    };
    let mut hexstr = String::new();
    let mut buf = [0; 4];
    for b in scalar.encode_utf8(&mut buf).as_bytes() {
        hexstr.push_str(&format!("\\x{:02X}", b));
    }
    Some(hexstr)
}

/// Converts the given codepoint into a displayable form. If the codepoint is
/// whitespace or control, then its escaped form is returned. If the codepoint
/// is not a scalar value, then `None` is returned.
fn nice_char(cp: Codepoint) -> Option<String> {
    let scalar = match cp.scalar() {
        None => return None,
        Some(scalar) => scalar,
    };
    Some(if scalar.is_whitespace() || scalar.is_control() {
        scalar.escape_default().collect()
    } else {
        scalar.to_string()
    })
}
