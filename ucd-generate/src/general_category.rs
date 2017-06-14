use ucd_parse::{self, UnicodeDataExpander};

use args::ArgMatches;
use error::Result;

pub fn command(args: ArgMatches) -> Result<()> {
    let dir = args.ucd_dir()?;
    let unexpanded = ucd_parse::parse(dir)?;
    let rows: Vec<_> = UnicodeDataExpander::new(unexpanded).collect();
    println!("{:?}", rows.len());
    // BREADCRUMBS:
    //
    // `rows` should contain every codepoint sans unassigned codepoints.
    // Group these into general categories, and then write out a set of
    // codepoints for each category. We should use the "long form" general
    // category value names. Use ucd_util::symbolic_name_normalize.
    //
    // Don't forget to write out the smattering of special purpose categories
    // which correspond to a union of other categories.
    Ok(())
}
