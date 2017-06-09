use std::ffi::OsStr;
use std::ops;

use clap;

use error::Result;

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
}
