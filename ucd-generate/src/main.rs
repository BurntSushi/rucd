#[macro_use]
extern crate clap;
extern crate ucd_parse;

use std::io::{self, Write};
use std::process;

use ucd_parse::UnicodeDataParser;

use args::ArgMatches;
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
mod args;
mod error;

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
        ("test-unicode-data", Some(m)) => {
            cmd_test_unicode_data(ArgMatches::new(m))
        }
        ("", _) => {
            app::app().print_help()?;
            println!("");
            Ok(())
        }
        (unknown, _) => err!("unrecognized command: {}", unknown),
    }
}

fn cmd_test_unicode_data(args: ArgMatches) -> Result<()> {
    let mut stdout = io::stdout();

    let dir = args.ucd_dir()?;
    let mut parser = UnicodeDataParser::from_dir(dir).unwrap();
    while let Some(result) = parser.parse_next() {
        let x = result.unwrap();
        writeln!(stdout, "{}", x)?;
    }
    Ok(())
}
