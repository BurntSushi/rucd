use std::ascii;
use std::io;

use fst::raw::Fst;

use error::Result;

/// Write the given FST as a lazy static to the given writer. The given name
/// is used as the name of the static.
pub fn write_fst<W: io::Write>(
    mut wtr: W,
    name: &str,
    fst: &Fst,
) -> Result<()> {
    writeln!(wtr, "lazy_static! {{")?;
    writeln!(wtr, "    pub static ref {}: ::fst::raw::Fst = ", name)?;
    writeln!(
        wtr,
        "        ::fst::raw::Fst::from_static_slice({}_BYTES).unwrap();",
        name)?;
    writeln!(wtr, "}}\n")?;

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

    let mut line = String::new();
    for &(cp, ref s) in table {
        let next = format!("({}, {:?}), ", cp, s);
        if !line.is_empty() && line.len() + next.len() > 79 {
            writeln!(wtr, "{}", line.trim())?;
            line.clear();
        }
        line.push_str(&next);
    }
    if !line.is_empty() {
        writeln!(wtr, "{}", line.trim())?;
    }

    writeln!(wtr, "];")?;
    Ok(())
}

/// Return the given byte as its escaped string form.
pub fn escape_input(b: u8) -> String {
    String::from_utf8(ascii::escape_default(b).collect::<Vec<_>>()).unwrap()
}
