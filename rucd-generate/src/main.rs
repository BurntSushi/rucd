extern crate ucd_parse;

use std::env;
use std::error::Error;
use std::fs::File;
// use std::io::{self, Write};
use std::process;

use ucd_parse::UnicodeDataParser;

macro_rules! eprintln {
    ($($tt:tt)*) => {{
        use std::io::Write;
        writeln!(&mut ::std::io::stderr(), $($tt)*).unwrap();
    }}
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<Error>> {
    // let stdout = io::stdout();
    // let mut stdout = stdout.lock();

    let fpath = env::args_os().nth(1).unwrap();
    let file = File::open(fpath).unwrap();
    let mut parser = UnicodeDataParser::new(file);
    // let mut sum = 0;
    while let Some(result) = parser.parse_next() {
        let x = result.unwrap();
        println!("{:#?}\n--------------------------------", x);
        // sum += x.codepoint.value();
        // writeln!(&mut stdout, "{:?}", x).unwrap();
    }
    // println!("{:?}", sum);
    Ok(())
}
