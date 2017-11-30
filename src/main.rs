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
mod list;
mod name;
mod search;
mod tables;

fn main() {
    if let Err(err) = try_main() {
        if err.is_broken_pipe() {
            process::exit(0);
        }
        eprintln!("{}", err);
        process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let matches = app::app().get_matches();
    match matches.subcommand() {
        ("search", Some(m)) => {
            search::command(m)
        }
        ("list-properties", Some(m)) => {
            list::command_list_properties(m)
        }
        ("list-property-values", Some(m)) => {
            list::command_list_property_values(m)
        }
        ("", _) => {
            app::app().print_help()?;
            println!("");
            Ok(())
        }
        (unknown, _) => err!("unrecognized command: {}", unknown),
    }
}
