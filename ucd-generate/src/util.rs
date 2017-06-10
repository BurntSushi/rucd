use std::ascii;
use std::io;

use byteorder::{ByteOrder, BigEndian as BE};
use fst::raw::Fst;
use ucd_parse::Codepoint;

use error::Result;

/// Write the given FST map as a lazy static to the given writer. The given
/// name is used as the name of the static.
pub fn write_fst_map<W: io::Write>(
    mut wtr: W,
    name: &str,
    fst: &Fst,
) -> Result<()> {
    writeln!(wtr, "use fst::raw::Fst;")?;
    writeln!(wtr, "use fst::Map;")?;
    writeln!(wtr, "")?;
    writeln!(wtr, "lazy_static! {{")?;
    writeln!(wtr, "  pub static ref {}: Map = ", name)?;
    writeln!(wtr, "    Map::from(Fst::from_static_slice(")?;
    writeln!( wtr, "      {}_BYTES).unwrap());", name)?;
    writeln!(wtr, "}}")?;
    writeln!(wtr, "")?;
    write_fst_bytes(wtr, name, fst)?;
    Ok(())
}

fn write_fst_bytes<W: io::Write>(
    mut wtr: W,
    name: &str,
    fst: &Fst,
) -> Result<()> {
    writeln!(wtr, "const {}_BYTES: &'static [u8] = b\"\\", name)?;
    let mut column = 0;
    for b in fst.to_vec() {
        let escaped = if (b as char).is_whitespace() {
            format!("\\x{:02x}", b)
        } else {
            escape_input(b)
        };
        if column + escaped.len() >= 79 {
            column = 0;
            write!(wtr, "\\\n")?;
        }
        column += escaped.len();
        write!(wtr, "{}", escaped)?;
    }
    writeln!(wtr, "\\\n\";")?;
    Ok(())
}

pub fn write_slice_u32_to_string<W: io::Write>(
    mut wtr: W,
    name: &str,
    table: &[(u32, String)],
) -> Result<()> {
    writeln!(wtr, "pub const {}: &'static [(u32, &'static str)] = &[", name)?;

    let mut line = "  ".to_string();
    for &(cp, ref s) in table {
        let next = format!("({}, {:?}), ", cp, s);
        if !line.trim().is_empty() && line.len() + next.len() > 79 {
            writeln!(wtr, "{}", line.trim_right())?;
            line.clear();
            line.push_str("  ");
        }
        line.push_str(&next);
    }
    if !line.is_empty() {
        writeln!(wtr, "{}", line.trim_right())?;
    }

    writeln!(wtr, "];")?;
    Ok(())
}

/// Return the given byte as its escaped string form.
pub fn escape_input(b: u8) -> String {
    String::from_utf8(ascii::escape_default(b).collect::<Vec<_>>()).unwrap()
}

/// Return the given codepoint encoded in big-endian.
pub fn codepoint_key(cp: Codepoint) -> [u8; 4] {
    let mut key = [0; 4];
    BE::write_u32(&mut key, cp.value());
    key
}
