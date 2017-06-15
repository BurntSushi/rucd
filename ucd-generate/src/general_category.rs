use ucd_parse::{self, UnicodeDataExpander};

use args::ArgMatches;
use error::Result;
use util::{PropertyNames, PropertyValues};

pub fn command(args: ArgMatches) -> Result<()> {
    let dir = args.ucd_dir()?;
    let props = PropertyNames::from_ucd_dir(&dir)?;
    let propvals = PropertyValues::from_ucd_dir(&dir)?;
    let unexpanded = ucd_parse::parse(&dir)?;
    let rows: Vec<_> = UnicodeDataExpander::new(unexpanded).collect();
    println!("{:#?}", props);
    println!("{:#?}", propvals);
    println!("{:?}", rows.len());
    println!("{:?}", propvals.canonical("g EnErALCATE go rY", "UNASSIGNED"));
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
