use std::collections::BTreeMap;
use std::io;
use std::path::Path;

use byteorder::{ByteOrder, BigEndian as BE};
use fst::{Map, MapBuilder};
use ucd_parse::{UcdLineParser, Codepoint, JamoShortName};

use args::ArgMatches;
use error::Result;
use util;

pub fn command(args: ArgMatches) -> Result<()> {
    let dir = args.ucd_dir()?;
    let jamo_map = parse_jamo_short_name(dir)?;

    if args.is_present("slice-table") {
        let mut table = vec![];
        for (cp, name) in jamo_map {
            table.push((cp.value(), name));
        }
        util::write_slice_u32_to_string(io::stdout(), args.name(), &table)?;
    } else {
        let mut builder = MapBuilder::memory();
        for (cp, name) in jamo_map {
            let mut key = [0; 4];
            BE::write_u32(&mut key, cp.value());

            let mut value = 0u64;
            for (i, &b) in name.as_bytes().iter().enumerate() {
                value |= (b as u64) << (i as u64);
            }

            builder.insert(key, value)?;
        }
        let fst = Map::from_bytes(builder.into_inner()?)?;
        args.write_fst(io::stdout(), args.name(), fst.as_fst())?;
    }
    Ok(())
}

/// Parse the contents of the Jamo.txt file.
fn parse_jamo_short_name<P: AsRef<Path>>(
    ucd_dir: P,
) -> Result<BTreeMap<Codepoint, String>> {
    let path = JamoShortName::from_dir(ucd_dir);
    let mut parser = UcdLineParser::from_path(path)?;

    let mut map = BTreeMap::new();
    while let Some(result) = parser.parse_next() {
        let x: JamoShortName = result?;
        map.insert(x.codepoint, x.name.into_owned());
    }
    Ok(map)
}
