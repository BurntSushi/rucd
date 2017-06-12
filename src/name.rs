use std::collections::BTreeMap;

use fst::Streamer;
use ucd_util;

use codepoint::Codepoint;
use tables::fst::names::NAMES;

lazy_static! {
    static ref NAMES_WITH_ALIASES: BTreeMap<Codepoint, Names> = {
        let mut map = BTreeMap::new();
        let mut stream = NAMES.stream();
        while let Some((name, tagged)) = stream.next() {
            let cp = Codepoint::from_u64(tagged).unwrap();
            if tagged & (1<<33) > 0 {
                let name = String::from_utf8(name.to_vec()).unwrap();
                let mut entry = map.entry(cp).or_insert(Names::default());
                entry.explicit = Some(name);
            } else if tagged & (1<<34) > 0 {
                let name = String::from_utf8(name.to_vec()).unwrap();
                let mut entry = map.entry(cp).or_insert(Names::default());
                entry.aliases.push(name);
            }
        }
        map
    };
}

/// The set of explicit names and their aliases for a single codepoint.
#[derive(Debug, Default)]
struct Names {
    /// The name as listed in UnicodeData.txt.
    explicit: Option<String>,
    /// Any additional aliases defined in NameAliases.txt.
    aliases: Vec<String>,
}

impl Names {
    /// Pick a single name, if one exists.
    fn one_name(&self) -> Option<&str> {
        if let Some(ref name) = self.explicit {
            return Some(name);
        }
        self.aliases.get(0).map(|name| &**name)
    }
}

/// Return the name of the given codepoint, if it exists.
pub fn codepoint_name(cp: Codepoint) -> Option<String> {
    if let Some(name) = ucd_util::hangul_name(cp.value()) {
        return Some(name);
    }
    if let Some(name) = ucd_util::ideograph_name(cp.value()) {
        return Some(name);
    }
    NAMES_WITH_ALIASES.get(&cp).and_then(|x| x.one_name()).map(|x| x.to_owned())
}
