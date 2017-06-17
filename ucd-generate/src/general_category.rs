use std::collections::{BTreeMap, BTreeSet};

use ucd_parse::{self, UnicodeDataExpander};

use args::ArgMatches;
use error::Result;
use util::PropertyValues;

pub fn command(args: ArgMatches) -> Result<()> {
    let dir = args.ucd_dir()?;
    let propvals = PropertyValues::from_ucd_dir(&dir)?;
    let unexpanded = ucd_parse::parse(&dir)?;

    // Expand all of our UnicodeData rows. This results in one big list of
    // all assigned codepoints.
    let rows: Vec<_> = UnicodeDataExpander::new(unexpanded).collect();

    // Collect each general category into an ordered set.
    let mut bycat: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();
    let mut assigned = BTreeSet::new();
    for row in rows {
        assigned.insert(row.codepoint.value());
        let gc = propvals
            .canonical("gc", &row.general_category)?
            .to_string();
        bycat.entry(gc)
            .or_insert(BTreeSet::new())
            .insert(row.codepoint.value());
    }
    // As a special case, collect all unassigned codepoints.
    if !args.is_present("no-unassigned") {
        let unassigned_name = propvals
            .canonical("gc", "unassigned")?
            .to_string();
        bycat.insert(unassigned_name.clone(), BTreeSet::new());
        for cp in 0..(0x10FFFF + 1) {
            if !assigned.contains(&cp) {
                bycat.get_mut(&unassigned_name).unwrap().insert(cp);
            }
        }
    }

    let mut wtr = args.writer("general_category")?;
    if args.is_present("enum") {
        wtr.ranges_to_enum("general_category", &bycat)?;
    } else {
        for (name, set) in bycat {
            wtr.ranges(&name, &set)?;
        }
    }

    Ok(())
}
