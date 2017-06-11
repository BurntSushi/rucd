use std::collections::BTreeMap;
use std::io;
use std::path::Path;

use fst::{Map, MapBuilder};
use ucd_parse::{UcdLineParser, Codepoint, JamoShortName};

use args::ArgMatches;
use error::Result;
use util;

pub fn command(args: ArgMatches) -> Result<()> {
    let dir = args.ucd_dir()?;
    let jamo_map = parse_jamo_short_name(dir)?;

    if !args.wants_fst() {
        let mut table = vec![];
        for (cp, name) in jamo_map {
            table.push((cp.value(), name));
        }
        util::write_slice_u32_to_string(io::stdout(), args.name(), &table)?;
    } else {
        let mut builder = MapBuilder::memory();
        for (cp, name) in jamo_map {
            let key = util::codepoint_key(cp);
            let value = jamo_name_to_u64(&name);
            builder.insert(key, value)?;
        }
        let fst = Map::from_bytes(builder.into_inner()?)?;
        args.write_fst_map(io::stdout(), args.name(), fst.as_fst())?;
    }
    Ok(())
}

/// Parse the contents of the Jamo.txt file.
fn parse_jamo_short_name<P: AsRef<Path>>(
    ucd_dir: P,
) -> Result<BTreeMap<Codepoint, String>> {
    let path = JamoShortName::from_dir(ucd_dir);
    let parser = UcdLineParser::from_path(path)?;
    let mut map = BTreeMap::new();
    for result in parser {
        let x: JamoShortName = result?;
        map.insert(x.codepoint, x.name.into_owned());
    }
    Ok(map)
}

/// Store a Jamo property value in the least significant bytes of a u64.
fn jamo_name_to_u64(name: &str) -> u64 {
    assert!(name.len() <= 3);
    let mut value = 0;
    for (i, &b) in name.as_bytes().iter().enumerate() {
        value |= (b as u64) << (8 * i as u64);
    }
    value
}

#[cfg(test)]
mod tests {
    use super::jamo_name_to_u64;

    fn u64_to_jamo_name(mut encoded: u64) -> String {
        let mut value = String::new();
        while encoded != 0 {
            value.push((encoded & 0xFF) as u8 as char);
            encoded = encoded >> 8;
        }
        value
    }

    #[test]
    fn jamo_name_encoding() {
        assert_eq!("G", u64_to_jamo_name(jamo_name_to_u64("G")));
        assert_eq!("GG", u64_to_jamo_name(jamo_name_to_u64("GG")));
        assert_eq!("YEO", u64_to_jamo_name(jamo_name_to_u64("YEO")));
        assert_eq!("", u64_to_jamo_name(jamo_name_to_u64("")));
    }
}
