#![allow(dead_code)]

// BREADCRUMBS:
//
// Move all output decisions into the Writer. Basic idea is that we specify
// common data structures and do the FST conversion inside the writer.
//
// Remove --raw-fst and make --rust-fst always emit FSTs to separate files
// They use much less space on disk, i.e., by avoiding encoding bytes in a
// byte literal.
//
// Problem: --rust-slice writes directly to a single file and it make sense
// to emit it to stdout. But --rust-fst wants to write at least two files:
// one for Rust source and another for the FST. An obvious solution is to leave
// --rust-slice untouched but require a directory argument for --rust-fst.
// It's a bit incongruous, which is unfortunate, but probably makes the most
// sense. However, if --rust-slice is always the default (which seems
// reasonable), then we could just remove --rust-slice altogether and that,
// I think, removes some of the incongruity.

use std::ascii;
use std::char;
use std::env;
use std::io::{self, Write};
use std::str;

use fst::raw::Fst;

use error::Result;

pub struct WriterBuilder {
    columns: u64,
    scalars: bool,
}

pub struct Writer<W> {
    wtr: W,
    wrote_header: bool,
}

impl Writer<io::Stdout> {
    pub fn stdout() -> Writer<io::Stdout> {
        Writer::from_writer(io::stdout())
    }
}

impl<W: io::Write> Writer<W> {
    pub fn from_writer(wtr: W) -> Writer<W> {
        Writer { wtr: wtr, wrote_header: false }
    }

    pub fn slice_ranges_u32(
        &mut self,
        name: &str,
        table: &[(u32, u32)],
        as_chars: bool,
    ) -> Result<()> {
        if as_chars {
            writeln!(
                self.wtr, "pub const {}: &'static [(char, char)] = &[", name)?;
        } else {
            writeln!(
                self.wtr, "pub const {}: &'static [(u32, u32)] = &[", name)?;
        }

        {
            let mut linewtr = LineWriter::new(&mut self.wtr);
            for &(start, end) in table {
                if !as_chars {
                    linewtr.write_str(format!("({}, {}), ", start, end))?;
                } else {
                    let start = match char::from_u32(start) {
                        None => continue,
                        Some(start) => start,
                    };
                    let end = match char::from_u32(end) {
                        None => continue,
                        Some(end) => end,
                    };
                    linewtr.write_str(format!("({:?}, {:?}), ", start, end))?;
                }
            }
            linewtr.flush()?;
        }
        writeln!(self.wtr, "];")?;
        Ok(())
    }

    pub fn slice_u32_to_string(
        &mut self,
        name: &str,
        table: &[(u32, String)],
    ) -> Result<()> {
        writeln!(
            self.wtr,
            "pub const {}: &'static [(u32, &'static str)] = &[",
            name)?;
        {
            let mut linewtr = LineWriter::new(&mut self.wtr);
            for &(cp, ref s) in table {
                linewtr.write_str(format!("({}, {:?}), ", cp, s))?;
            }
            linewtr.flush()?;
        }
        writeln!(self.wtr, "];")?;
        Ok(())
    }

    pub fn slice_string_to_u64(
        &mut self,
        name: &str,
        table: &[(String, u64)],
    ) -> Result<()> {
        writeln!(
            self.wtr,
            "pub const {}: &'static [(&'static str, u32)] = &[",
            name)?;
        {
            let mut linewtr = LineWriter::new(&mut self.wtr);
            for &(ref s, cp) in table {
                linewtr.write_str(format!("({:?}, {}), ", s, cp))?;
            }
            linewtr.flush()?;
        }
        writeln!(self.wtr, "];")?;
        Ok(())
    }

    /// Write the given FST map as a lazy static to the given writer. The given
    /// name is used as the name of the static.
    pub fn fst_map(&mut self, name: &str, fst: &Fst) -> Result<()> {
        writeln!(self.wtr, "use fst::raw::Fst;")?;
        writeln!(self.wtr, "use fst::Map;")?;
        writeln!(self.wtr, "")?;
        writeln!(self.wtr, "lazy_static! {{")?;
        writeln!(self.wtr, "  pub static ref {}: Map = ", name)?;
        writeln!(self.wtr, "    Map::from(Fst::from_static_slice(")?;
        writeln!(self.wtr, "      {}_BYTES).unwrap());", name)?;
        writeln!(self.wtr, "}}")?;
        writeln!(self.wtr, "")?;
        self.fst_bytes(name, fst)
    }

    /// Write the given FST set as a lazy static to the given writer. The given
    /// name is used as the name of the static.
    pub fn fst_set(&mut self, name: &str, fst: &Fst) -> Result<()> {
        writeln!(self.wtr, "use fst::raw::Fst;")?;
        writeln!(self.wtr, "use fst::Set;")?;
        writeln!(self.wtr, "")?;
        writeln!(self.wtr, "lazy_static! {{")?;
        writeln!(self.wtr, "  pub static ref {}: Set = ", name)?;
        writeln!(self.wtr, "    Set::from(Fst::from_static_slice(")?;
        writeln!(self.wtr, "      {}_BYTES).unwrap());", name)?;
        writeln!(self.wtr, "}}")?;
        writeln!(self.wtr, "")?;
        self.fst_bytes(name, fst)
    }

    fn fst_bytes(&mut self, name: &str, fst: &Fst) -> Result<()> {
        writeln!(self.wtr, "const {}_BYTES: &'static [u8] = b\"\\", name)?;
        let mut column = 0;
        for b in fst.to_vec() {
            let escaped = if (b as char).is_whitespace() {
                format!("\\x{:02x}", b)
            } else {
                escape_input(b)
            };
            if column + escaped.len() >= 79 {
                column = 0;
                write!(self.wtr, "\\\n")?;
            }
            column += escaped.len();
            write!(self.wtr, "{}", escaped)?;
        }
        writeln!(self.wtr, "\\\n\";")?;
        Ok(())
    }

    fn header(&mut self) -> Result<()> {
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
        writeln!(self.wtr, "// DO NOT EDIT THIS FILE. \
                               IT WAS AUTOMATICALLY GENERATED BY:")?;
        writeln!(self.wtr, "//")?;
        writeln!(self.wtr, "//  {}", argv.join(" "))?;
        writeln!(self.wtr, "//")?;
        writeln!(self.wtr, "// ucd-generate is available on crates.io.")?;
        writeln!(self.wtr, "")?;
        self.wrote_header = true;
        Ok(())
    }
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

/// Return the given byte as its escaped string form.
fn escape_input(b: u8) -> String {
    String::from_utf8(ascii::escape_default(b).collect::<Vec<_>>()).unwrap()
}
