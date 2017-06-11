use std::collections::BTreeMap;
use std::io;
use std::path::Path;

use fst::{Map, MapBuilder};
use ucd_parse::{Codepoint, UcdLineParser, UnicodeData, NameAlias};
use ucd_util;

use args::ArgMatches;
use error::Result;
use util;

pub fn command(args: ArgMatches) -> Result<()> {
    let dir = args.ucd_dir()?;
    let data = util::parse_unicode_data(&dir)?;
    let aliases =
        if args.is_present("no-aliases") {
            None
        } else {
            Some(parse_name_aliases(&dir)?)
        };
    let mut names = names_to_codepoint(
        &data,
        &aliases,
        !args.is_present("no-ideograph"),
        !args.is_present("no-hangul"));
    if args.is_present("normalize") {
        names = names.into_iter().map(|(mut name, cp)| {
            ucd_util::character_name_normalize(&mut name);
            (name, cp)
        }).collect();
    }

    if !args.wants_slice() {
        let mut builder = MapBuilder::memory();
        for (name, cp) in names {
            builder.insert(name.as_bytes(), cp as u64)?;
        }
        let fst = Map::from_bytes(builder.into_inner()?)?;
        args.write_fst_map(io::stdout(), args.name(), fst.as_fst())?;
    } else {
        let table: Vec<(String, u32)> = names.into_iter().collect();
        util::write_slice_string_to_u32(io::stdout(), args.name(), &table)?;
    }
    Ok(())
}

/// Build one big map in memory from every possible name of a character to its
/// corresponding codepoint. Note that one codepoint may have multiple names.
fn names_to_codepoint(
    data: &BTreeMap<Codepoint, UnicodeData<'static>>,
    aliases: &Option<BTreeMap<Codepoint, NameAlias<'static>>>,
    ideograph: bool,
    hangul: bool,
) -> BTreeMap<String, u32> {
    let mut map = BTreeMap::new();
    for (cp, datum) in data {
        let isnull =
            datum.name.is_empty()
            || (datum.name.starts_with('<') && datum.name.ends_with('>'));
        if !isnull {
            map.insert(datum.name.clone().into_owned(), cp.value());
        }
        if !datum.unicode1_name.is_empty() && (isnull || aliases.is_some()) {
            map.insert(datum.unicode1_name.clone().into_owned(), cp.value());
        }
    }
    if let Some(ref aliases) = *aliases {
        for (cp, name_alias) in aliases {
            map.insert(name_alias.alias.clone().into_owned(), cp.value());
        }
    }
    if ideograph {
        for &(start, end) in ucd_util::RANGE_IDEOGRAPH {
            for cp in start..end + 1 {
                map.insert(ucd_util::ideograph_name(cp).unwrap(), cp);
            }
        }
    }
    if hangul {
        for &(start, end) in ucd_util::RANGE_HANGUL_SYLLABLE {
            for cp in start..end + 1 {
                map.insert(ucd_util::hangul_name(cp).unwrap(), cp);
            }
        }
    }
    map
}

fn parse_name_aliases<P: AsRef<Path>>(
    ucd_dir: P,
) -> Result<BTreeMap<Codepoint, NameAlias<'static>>> {
    let path = NameAlias::from_dir(ucd_dir);
    let parser = UcdLineParser::from_path(path)?;
    let mut map = BTreeMap::new();
    for result in parser {
        let x: NameAlias = result?;
        map.insert(x.codepoint, x.into_owned());
    }
    Ok(map)
}
