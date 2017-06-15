use std::ascii;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::str;

use byteorder::{ByteOrder, BigEndian as BE};
use fst::raw::Fst;
use ucd_parse::{Codepoint, PropertyAlias, PropertyValueAlias};
use ucd_util;

use error::Result;

/// Write a header to a Rust source file. The header includes a warning about
/// the file being auto-generated and includes the command that was used to
/// generate it.
pub fn write_header<W: io::Write>(mut wtr: W) -> Result<()> {
    let mut argv = vec![];
    argv.push(
        env::current_exe()?
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned());
    for arg in env::args_os().skip(1) {
        let x = arg.to_string_lossy();
        argv.push(x.into_owned());
    }
    writeln!(wtr, "// DO NOT EDIT THIS FILE. \
                 IT WAS AUTOMATICALLY GENERATED BY:")?;
    writeln!(wtr, "//")?;
    writeln!(wtr, "//  {}", argv.join(" "))?;
    writeln!(wtr, "//")?;
    writeln!(wtr, "// ucd-generate is available on crates.io.")?;
    writeln!(wtr, "")?;
    Ok(())
}

/// Write the given FST map as a lazy static to the given writer. The given
/// name is used as the name of the static.
pub fn write_fst_map<W: io::Write>(
    mut wtr: W,
    name: &str,
    fst: &Fst,
) -> Result<()> {
    write_header(&mut wtr)?;
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

pub fn write_slice_btree_u32<W: io::Write>(
    mut wtr: W,
    name: &str,
    table: &BTreeSet<u32>,
) -> Result<()> {
    write_header(&mut wtr)?;
    writeln!(wtr, "pub const {}: &'static [u32] = &[", name)?;

    {
        let mut linewtr = LineWriter::new(&mut wtr);
        for &cp in table {
            linewtr.write_str(format!("{}, ", cp))?;
        }
        linewtr.flush()?;
    }

    writeln!(wtr, "];")?;
    Ok(())
}

pub fn write_slice_u64_to_string<W: io::Write>(
    mut wtr: W,
    name: &str,
    table: &[(u64, String)],
) -> Result<()> {
    write_header(&mut wtr)?;
    writeln!(wtr, "pub const {}: &'static [(u32, &'static str)] = &[", name)?;

    {
        let mut linewtr = LineWriter::new(&mut wtr);
        for &(cp, ref s) in table {
            linewtr.write_str(format!("({}, {:?}), ", cp, s))?;
        }
        linewtr.flush()?;
    }

    writeln!(wtr, "];")?;
    Ok(())
}

pub fn write_slice_string_to_u64<W: io::Write>(
    mut wtr: W,
    name: &str,
    table: &[(String, u64)],
) -> Result<()> {
    write_header(&mut wtr)?;
    writeln!(wtr, "pub const {}: &'static [(&'static str, u32)] = &[", name)?;

    {
        let mut linewtr = LineWriter::new(&mut wtr);
        for &(ref s, cp) in table {
            linewtr.write_str(format!("({:?}, {}), ", s, cp))?;
        }
        linewtr.flush()?;
    }

    writeln!(wtr, "];")?;
    Ok(())
}

struct LineWriter<W> {
    wtr: W,
    line: String,
    indent: String,
    columns: usize,
}

impl<W: io::Write> LineWriter<W> {
    fn new(wtr: W) -> LineWriter<W> {
        let indent = "  ".to_string();
        LineWriter {
            wtr: wtr,
            line: indent.clone(),
            indent: indent,
            columns: 79,
        }
    }

    fn write_str(&mut self, s: String) -> io::Result<()> {
        self.write_all(s.as_bytes())
    }

    fn flush_inner(&mut self) -> io::Result<()> {
        self.wtr.write_all(self.line.trim_right().as_bytes())?;
        self.wtr.write_all(b"\n")?;
        self.line.clear();
        self.line.push_str(&self.indent);
        Ok(())
    }
}

impl<W: io::Write> io::Write for LineWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let buf = match str::from_utf8(buf) {
            Ok(buf) => buf,
            Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
        };
        if !self.line.trim().is_empty()
            && self.line.len() + buf.len() > self.columns
        {
            self.flush_inner()?;
        }
        self.line.push_str(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if !self.line.is_empty() {
            self.flush_inner()?;
        }
        self.wtr.flush()
    }
}

/// A map from property name (including aliases) to a "canonical" or "long"
/// version of the property name.
///
/// All keys are normalized according to UAX44-LM3.
#[derive(Clone, Debug)]
pub struct PropertyNames(BTreeMap<String, String>);

impl PropertyNames {
    pub fn from_ucd_dir<P: AsRef<Path>>(ucd_dir: P) -> Result<PropertyNames> {
        use ucd_parse::UcdFile;

        let mut map = BTreeMap::new();
        for result in PropertyAlias::from_dir(ucd_dir)? {
            let a = result?;
            let canon = a.long.to_string();
            let make_key = |mut value| {
                ucd_util::symbolic_name_normalize(&mut value);
                value
            };

            for alias in a.aliases {
                map.insert(make_key(alias.into_owned()), canon.clone());
            }
            map.insert(make_key(a.abbreviation.into_owned()), canon.clone());
            map.insert(make_key(a.long.into_owned()), canon);
        }
        Ok(PropertyNames(map))
    }

    /// Return the "canonical" or "long" property name for the given property
    /// name. If no such property exists, return an error.
    pub fn canonical<'a>(&'a self, key: &str) -> Result<&'a str> {
        let mut key = key.to_string();
        ucd_util::symbolic_name_normalize(&mut key);
        match self.0.get(&key).map(|v| &**v) {
            Some(v) => Ok(v),
            None => err!("unrecognized property: {:?}", key),
        }
    }
}

/// A map from (property name, property value) to a "canonical" or "long"
/// version of the corresponding property value.
///
/// Property names and values are normalized according to UAX44-LM3.
#[derive(Clone, Debug)]
pub struct PropertyValues {
    property: PropertyNames,
    value: BTreeMap<String, BTreeMap<String, String>>,
}

impl PropertyValues {
    pub fn from_ucd_dir<P: AsRef<Path>>(ucd_dir: P) -> Result<PropertyValues> {
        use ucd_parse::UcdFile;

        let props = PropertyNames::from_ucd_dir(&ucd_dir)?;
        let mut outer_map = BTreeMap::new();
        for result in PropertyValueAlias::from_dir(ucd_dir)? {
            let a = result?;
            let prop = props.canonical(&a.property)?.to_string();
            let canon = a.long.to_string();
            let make_key = |mut value| {
                ucd_util::symbolic_name_normalize(&mut value);
                value
            };

            let mut inner_map = outer_map.entry(prop).or_insert(BTreeMap::new());
            if let Some(n) = a.numeric {
                inner_map.insert(make_key(n.to_string()), canon.clone());
            }
            for alias in a.aliases {
                inner_map.insert(make_key(alias.into_owned()), canon.clone());
            }
            inner_map.insert(make_key(a.abbreviation.into_owned()), canon.clone());
            inner_map.insert(make_key(a.long.into_owned()), canon);
        }
        Ok(PropertyValues { property: props, value: outer_map })
    }

    /// Return the "canonical" or "long" property value for the given property
    /// value for a specific property. If no such property exists or if not
    /// such property value exists, then return an error.
    ///
    /// Note that this does not apply to "string" or "miscellaneous" properties
    /// such as `Name` or `Case_Folding`.
    pub fn canonical<'a>(
        &'a self,
        property: &str,
        value: &str,
    ) -> Result<&'a str> {
        let property = self.property.canonical(property)?;
        let mut value = value.to_string();
        ucd_util::symbolic_name_normalize(&mut value);
        match self.value.get(&*property).and_then(|m| m.get(&value)) {
            Some(v) => Ok(v),
            None => err!(
                "unrecognized property name/value: {:?}", (property, value)),
        }
    }
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

/// Heuristically produce an appropriate constant Rust name.
pub fn rust_const_name(s: &str) -> String {
    use std::ascii::AsciiExt;

    // Property names/values seem pretty uniform, particularly the
    // "canonical" variants we use to produce variable names. So we
    // don't need to do much.
    let mut s = s.to_string();
    s.make_ascii_uppercase();
    s
}
