use std::io;

use fst::{Map, MapBuilder};
use ucd_parse::{self, JamoShortName};

use args::ArgMatches;
use error::Result;
use util;

pub fn command(args: ArgMatches) -> Result<()> {
    let dir = args.ucd_dir()?;
    let jamo_map = ucd_parse::parse_by_codepoint::<_, JamoShortName>(dir)?;

    if args.wants_fst() {
        let mut builder = MapBuilder::memory();
        for (cp, jamo) in jamo_map {
            let key = util::codepoint_key(cp);
            let value = jamo_name_to_u64(&jamo.name);
            builder.insert(key, value)?;
        }
        let fst = Map::from_bytes(builder.into_inner()?)?;
        args.write_fst_map(io::stdout(), args.name(), fst.as_fst())?;
    } else {
        let mut table = vec![];
        for (cp, jamo) in jamo_map {
            table.push((cp.value(), jamo.name.into_owned()));
        }
        util::write_header(io::stdout())?;
        util::write_slice_u32_to_string(io::stdout(), args.name(), &table)?;
    }
    Ok(())
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
