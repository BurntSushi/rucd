use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write};

use fst::{Set, SetBuilder};
// use fst::{Map, MapBuilder};
use ucd_parse::{self, UnicodeDataExpander};

use args::ArgMatches;
use error::Result;
use util::{self, PropertyValues};

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

    let mut wtr = io::BufWriter::new(io::stdout());
    util::write_header(&mut wtr)?;
    if args.wants_fst() {
        for (name, set) in bycat {
            // let mut builder = MapBuilder::memory();
            // for (s, e) in util::to_ranges(set) {
                // builder.insert(util::u32_key(s), e as u64)?;
            // }

            let mut builder = SetBuilder::memory();
            // for (s, e) in util::to_ranges(set) {
                // builder.insert(util::u32_key(s))?;
                // builder.insert(util::u32_key(e))?;
            // }
            builder.extend_iter(set.into_iter().map(util::u32_key))?;
            // let fst = Map::from_bytes(builder.into_inner()?)?;
            let fst = Set::from_bytes(builder.into_inner()?)?;

            let name = util::rust_const_name(&name);
            util::write_fst_set(&mut wtr, &name, fst.as_fst())?;
        }
    } else {
        for (name, set) in bycat {
            let name = util::rust_const_name(&name);
            let ranges = util::to_ranges(set);
            util::write_slice_ranges_u32(
                &mut wtr, &name, &ranges, args.is_present("chars"))?;
            write!(wtr, "\n")?;
        }
    }

    // BREADCRUMBS:
    //
    // FSTs aren't as much of a space savings as I thought they might be. In
    // fact, when they're embedded into Rust source code, they use more space
    // than a simple table. Blech. If we switch to representing ranges instead
    // of storing every codepoint, then accesses get trickier, which is a
    // bummer.
    //
    // A lot of plumbing needs to happen too:
    //
    // 1. Writing FSTs to Rust source doesn't really work right now, since
    //    emitting each FST also includes `use` statements. Probably just
    //    use absolute qualification inline.
    // 2. Add a "previous key" (and maybe "next key"?) methods to FSTs. That
    //    will enable efficient lookup when the FST stores ranges.
    // 3. Permit writing FSTs to separate files as a first class option.
    // 4. Perhaps tries will save us all.

    Ok(())
}
