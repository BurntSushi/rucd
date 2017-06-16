use std::collections::BTreeMap;
use std::io;

use fst::{Map, MapBuilder};
use ucd_parse::{self, Codepoint, UnicodeData, NameAlias};
use ucd_util;

use args::ArgMatches;
use error::Result;
use util;

pub fn command(args: ArgMatches) -> Result<()> {
    let dir = args.ucd_dir()?;
    let data = ucd_parse::parse_by_codepoint(&dir)?;
    let aliases =
        if args.is_present("no-aliases") {
            None
        } else {
            Some(ucd_parse::parse_many_by_codepoint(&dir)?)
        };
    let mut names = names_to_codepoint(
        &data,
        &aliases,
        !args.is_present("no-ideograph"),
        !args.is_present("no-hangul"));
    if args.is_present("normalize") {
        names = names.into_iter().map(|(mut name, tagged)| {
            ucd_util::character_name_normalize(&mut name);
            (name, tagged)
        }).collect();
    }
    let codepoint = |tag: NameTag, cp: u32| -> u64 {
        if args.is_present("tagged") {
            tag.with_codepoint(cp)
        } else {
            cp as u64
        }
    };

    if args.wants_fst() {
        let mut builder = MapBuilder::memory();
        for (name, (tag, cp)) in names {
            builder.insert(name.as_bytes(), codepoint(tag, cp))?;
        }
        let fst = Map::from_bytes(builder.into_inner()?)?;
        args.write_fst_map(io::stdout(), args.name(), fst.as_fst())?;
    } else {
        let mut table = vec![];
        for (name, (tag, cp)) in names {
            table.push((name, codepoint(tag, cp)));
        }
        util::write_header(io::stdout())?;
        util::write_slice_string_to_u64(io::stdout(), args.name(), &table)?;
    }
    Ok(())
}

/// A tag indicating how the name of a codepoint was found.
///
/// When a name has both an algorithmically generated name and an
/// explicit/alias name, then the algorithmically generated tag is preferred.
#[derive(Debug)]
enum NameTag {
    /// The name is listed explicitly in UnicodeData.txt.
    Explicit,
    /// The name was taken from NameAliases.txt.
    Alias,
    /// The name is an algorithmically generated Hangul syllable.
    Hangul,
    /// The name is an algorithmically generated ideograph.
    Ideograph,
}

impl NameTag {
    fn with_codepoint(&self, cp: u32) -> u64 {
        use self::NameTag::*;
        match *self {
            Explicit => (1<<33) | (cp as u64),
            Alias => (1<<34) | (cp as u64),
            Hangul => (1<<35) | (cp as u64),
            Ideograph => (1<<36) | (cp as u64),
        }
    }
}

/// Build one big map in memory from every possible name of a character to its
/// corresponding codepoint. One codepoint may be pointed to by multiple names.
///
/// The return value maps each name to its corresponding codepoint, along with
/// a tag associated with how that mapping was generated.
fn names_to_codepoint(
    data: &BTreeMap<Codepoint, UnicodeData<'static>>,
    aliases: &Option<BTreeMap<Codepoint, Vec<NameAlias<'static>>>>,
    ideograph: bool,
    hangul: bool,
) -> BTreeMap<String, (NameTag, u32)> {
    let mut map = BTreeMap::new();
    if let Some(ref alias_map) = *aliases {
        for (cp, aliases) in alias_map {
            for name_alias in aliases {
                let v = (NameTag::Alias, cp.value());
                map.insert(name_alias.alias.clone().into_owned(), v);
            }
        }
    }
    for (cp, datum) in data {
        let isnull =
            datum.name.is_empty()
            || (datum.name.starts_with('<') && datum.name.ends_with('>'));
        if !isnull {
            let v = (NameTag::Explicit, cp.value());
            map.insert(datum.name.clone().into_owned(), v);
        }
    }
    if ideograph {
        for &(start, end) in ucd_util::RANGE_IDEOGRAPH {
            for cp in start..end + 1 {
                let v = (NameTag::Ideograph, cp);
                map.insert(ucd_util::ideograph_name(cp).unwrap(), v);
            }
        }
    }
    if hangul {
        for &(start, end) in ucd_util::RANGE_HANGUL_SYLLABLE {
            for cp in start..end + 1 {
                let v = (NameTag::Hangul, cp);
                map.insert(ucd_util::hangul_name(cp).unwrap(), v);
            }
        }
    }
    map
}
