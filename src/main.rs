#![allow(dead_code)]

extern crate byteorder;
#[macro_use]
extern crate clap;
extern crate fst;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate tabwriter;
extern crate ucd_util;
extern crate unicode_width;

use std::process;

use error::Result;

macro_rules! eprintln {
    ($($tt:tt)*) => {{
        use std::io::Write;
        writeln!(&mut ::std::io::stderr(), $($tt)*).unwrap();
    }}
}

macro_rules! err {
    ($($tt:tt)*) => {
        Err(::error::Error::Other(format!($($tt)*)))
    }
}

mod app;
mod codepoint;
mod display;
mod error;
mod name;
mod search;
mod tables;

fn main() {
    if let Err(err) = run() {
        if err.is_broken_pipe() {
            process::exit(0);
        }
        eprintln!("{}", err);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let matches = app::app().get_matches();
    match matches.subcommand() {
        ("search", Some(m)) => {
            search::command(m)
        }
        ("", _) => {
            app::app().print_help()?;
            println!("");
            Ok(())
        }
        (unknown, _) => err!("unrecognized command: {}", unknown),
    }
}
