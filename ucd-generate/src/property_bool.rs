use std::collections::{BTreeMap, BTreeSet};

use ucd_parse::{self, CoreProperty, Property};

use args::ArgMatches;
use error::Result;
use util::PropertyNames;

pub fn command(args: ArgMatches) -> Result<()> {
    let dir = args.ucd_dir()?;
    let properties = PropertyNames::from_ucd_dir(&dir)?;
    let filter = args.filter(|name| properties.canonical(name))?;
    let mut by_name: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();

    let prop_list: Vec<Property> = ucd_parse::parse(&dir)?;
    for x in &prop_list {
        by_name
            .entry(x.property.clone())
            .or_insert(BTreeSet::new())
            .extend(x.codepoints.into_iter().map(|c| c.value()));
    }

    let core_prop: Vec<CoreProperty> = ucd_parse::parse(&dir)?;
    for x in &core_prop {
        by_name
            .entry(x.property.clone())
            .or_insert(BTreeSet::new())
            .extend(x.codepoints.into_iter().map(|c| c.value()));
    }

    // TODO: PropList.txt and DerivedCoreProperties.txt cover the majority
    // of boolean properties, but UAX44 S5.3 Table 9 lists a smattering of
    // others that we should include here as well. (Some will need support in
    // ucd-parse, for example, the ones found in DerivedNormalizationProps.txt
    // while others, like Bidi_Mirrored, are derived from UnicodeData.txt.
    // Even still, others like Composition_Exclusion have their own file
    // (CompositionExclusions.txt).

    if args.is_present("list-properties") {
        for name in by_name.keys() {
            println!("{}", name);
        }
        return Ok(());
    }
    let mut wtr = args.writer("prop_list")?;
    for (name, set) in by_name {
        if filter.contains(&name) {
            wtr.ranges(&name, &set)?;
        }
    }
    Ok(())
}
