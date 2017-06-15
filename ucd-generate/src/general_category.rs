use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write};

use ucd_parse::{self, UnicodeDataExpander};

use args::ArgMatches;
use error::Result;
use util::{self, PropertyValues};

pub fn command(args: ArgMatches) -> Result<()> {
    let dir = args.ucd_dir()?;
    let propvals = PropertyValues::from_ucd_dir(&dir)?;
    let unexpanded = ucd_parse::parse(&dir)?;
    let rows: Vec<_> = UnicodeDataExpander::new(unexpanded).collect();

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

    let unassigned_name = propvals
        .canonical("gc", "unassigned")?
        .to_string();
    bycat.insert(unassigned_name.clone(), BTreeSet::new());
    for cp in 0..(0x10FFFF + 1) {
        if !assigned.contains(&cp) {
            bycat.get_mut(&unassigned_name).unwrap().insert(cp);
        }
    }

    let mut wtr = io::BufWriter::new(io::stdout());
    for (name, set) in bycat {
        let name = util::rust_const_name(&name);
        util::write_slice_btree_u32(&mut wtr, &name, &set)?;
        write!(wtr, "\n\n")?;
    }
    // BREADCRUMBS:
    //
    // 1. Codepoints should be written as ranges. Derp.
    // 2. Add FST output format.
    // 3. Add trie output format.
    // 4. Make this more configurable?
    Ok(())
}
