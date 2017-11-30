use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write};

use clap;
use tabwriter::TabWriter;
use ucd_util::{
    canonical_property_name, property_values, symbolic_name_normalize,
};

use app::arg_to_str;
use error::Result;
use tables::slice::property_names::PROPERTY_NAMES;
use tables::slice::property_values::PROPERTY_VALUES;

pub fn command_list_properties(_: &clap::ArgMatches) -> Result<()> {
    print_assoc_list(&by_canonical_symbolic_name(PROPERTY_NAMES))
}

pub fn command_list_property_values(args: &clap::ArgMatches) -> Result<()> {
    let given_prop = arg_to_str("property", args.value_of_os("property"))?;
    let canon_prop = find_canonical_property_name(&given_prop)?;
    let values = find_property_values(canon_prop)?;
    print_assoc_list(&by_canonical_symbolic_name(values))
}

fn print_assoc_list(
    assocs: &BTreeMap<String, BTreeSet<String>>,
) -> Result<()> {
    let mut order: Vec<&str> = assocs.keys().map(|k| &**k).collect();
    order.sort_by_key(|k| k.to_lowercase());

    let longest = assocs.values().map(|v| v.len()).max().unwrap_or(0);
    let mut wtr = TabWriter::new(io::stdout());
    for &key in &order {
        let mut vals: Vec<&str> = assocs[key].iter().map(|v| &**v).collect();
        for _ in 0..longest - vals.len() {
            vals.push("");
        }
        writeln!(wtr, "{}\t{}", key, vals.join("\t"))?;
    }
    wtr.flush()?;
    Ok(())
}

fn find_canonical_property_name(given: &str) -> Result<&'static str> {
    let mut norm = given.to_string();
    symbolic_name_normalize(&mut norm);
    canonical_property_name(PROPERTY_NAMES, &norm)
        .map_or(err!("could not find property matching '{}'", given), Ok)
}

fn find_property_values(
    canonical_property_name: &str,
) -> Result<&'static [(&'static str, &'static str)]> {
    let name = canonical_property_name;
    property_values(PROPERTY_VALUES, name)
        .map_or(err!("could not find property values for '{}'", name), Ok)
}

fn by_canonical_symbolic_name(
    assocs: &[(&str, &str)],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut by_canon: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for &(alias, canon) in assocs {
        if !by_canon.contains_key(canon) {
            by_canon.insert(canon.to_string(), BTreeSet::new());
        }
        let mut norm_canon = canon.to_string();
        symbolic_name_normalize(&mut norm_canon);
        if norm_canon != alias {
            by_canon.get_mut(canon).unwrap().insert(alias.to_string());
        }
    }
    by_canon
}
