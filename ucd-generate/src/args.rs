use std::ffi::OsStr;
use std::io;
use std::ops;

use clap;
use fst::raw::Fst;

use error::Result;
use util;

/// Wraps clap matches and provides convenient accessors to various parameters.
pub struct ArgMatches<'a>(&'a clap::ArgMatches<'a>);

impl<'a> ops::Deref for ArgMatches<'a> {
    type Target = clap::ArgMatches<'a>;
    fn deref(&self) -> &clap::ArgMatches<'a> { &self.0 }
}

impl<'a> ArgMatches<'a> {
    pub fn new(matches: &'a clap::ArgMatches<'a>) -> ArgMatches<'a> {
        ArgMatches(matches)
    }

    pub fn ucd_dir(&self) -> Result<&OsStr> {
        match self.value_of_os("ucd-dir") {
            Some(x) => Ok(x),
            None => err!("missing UCD directory"),
        }
    }

    pub fn name(&self) -> &str {
        self.value_of("name").expect("the name of the table")
    }

    pub fn wants_fst(&self) -> bool {
        self.is_present("raw-fst") || self.is_present("rust-fst")
    }

    pub fn write_fst_map<W: io::Write>(
        &self,
        mut wtr: W,
        name: &str,
        fst: &Fst,
    ) -> Result<()> {
        if self.is_present("raw-fst") {
            wtr.write_all(&fst.to_vec())?;
        } else {
            util::write_header(&mut wtr)?;
            util::write_fst_map(wtr, name, &fst)?;
        }
        Ok(())
    }
}
