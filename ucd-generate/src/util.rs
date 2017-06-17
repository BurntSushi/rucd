use std::collections::BTreeMap;
use std::path::Path;
use std::str;

use ucd_parse::{PropertyAlias, PropertyValueAlias};
use ucd_util;

use error::Result;

/// A map from property name (including aliases) to a "canonical" or "long"
/// version of the property name.
///
/// All keys are normalized according to UAX44-LM3.
#[derive(Clone, Debug)]
pub struct PropertyNames(BTreeMap<String, String>);

impl PropertyNames {
    pub fn from_ucd_dir<P: AsRef<Path>>(ucd_dir: P) -> Result<PropertyNames> {
        use ucd_parse::UcdFile;

        let mut map = BTreeMap::new();
        for result in PropertyAlias::from_dir(ucd_dir)? {
            let a = result?;
            let canon = a.long.to_string();
            let make_key = |mut value| {
                ucd_util::symbolic_name_normalize(&mut value);
                value
            };

            for alias in a.aliases {
                map.insert(make_key(alias.into_owned()), canon.clone());
            }
            map.insert(make_key(a.abbreviation.into_owned()), canon.clone());
            map.insert(make_key(a.long.into_owned()), canon);
        }
        Ok(PropertyNames(map))
    }

    /// Return the "canonical" or "long" property name for the given property
    /// name. If no such property exists, return an error.
    pub fn canonical<'a>(&'a self, key: &str) -> Result<&'a str> {
        let mut key = key.to_string();
        ucd_util::symbolic_name_normalize(&mut key);
        match self.0.get(&key).map(|v| &**v) {
            Some(v) => Ok(v),
            None => err!("unrecognized property: {:?}", key),
        }
    }
}

/// A map from (property name, property value) to a "canonical" or "long"
/// version of the corresponding property value.
///
/// Property names and values are normalized according to UAX44-LM3.
#[derive(Clone, Debug)]
pub struct PropertyValues {
    property: PropertyNames,
    value: BTreeMap<String, BTreeMap<String, String>>,
}

impl PropertyValues {
    pub fn from_ucd_dir<P: AsRef<Path>>(ucd_dir: P) -> Result<PropertyValues> {
        use ucd_parse::UcdFile;

        let props = PropertyNames::from_ucd_dir(&ucd_dir)?;
        let mut outer_map = BTreeMap::new();
        for result in PropertyValueAlias::from_dir(ucd_dir)? {
            let a = result?;
            let prop = props.canonical(&a.property)?.to_string();
            let canon = a.long.to_string();
            let make_key = |mut value| {
                ucd_util::symbolic_name_normalize(&mut value);
                value
            };

            let mut inner_map = outer_map.entry(prop).or_insert(BTreeMap::new());
            if let Some(n) = a.numeric {
                inner_map.insert(make_key(n.to_string()), canon.clone());
            }
            for alias in a.aliases {
                inner_map.insert(make_key(alias.into_owned()), canon.clone());
            }
            inner_map.insert(make_key(a.abbreviation.into_owned()), canon.clone());
            inner_map.insert(make_key(a.long.into_owned()), canon);
        }
        Ok(PropertyValues { property: props, value: outer_map })
    }

    /// Return the "canonical" or "long" property value for the given property
    /// value for a specific property. If no such property exists or if not
    /// such property value exists, then return an error.
    ///
    /// Note that this does not apply to "string" or "miscellaneous" properties
    /// such as `Name` or `Case_Folding`.
    pub fn canonical<'a>(
        &'a self,
        property: &str,
        value: &str,
    ) -> Result<&'a str> {
        let property = self.property.canonical(property)?;
        let mut value = value.to_string();
        ucd_util::symbolic_name_normalize(&mut value);
        match self.value.get(&*property).and_then(|m| m.get(&value)) {
            Some(v) => Ok(v),
            None => err!(
                "unrecognized property name/value: {:?}", (property, value)),
        }
    }
}

/// Convert an iterator of codepoints into a vec of sorted ranges.
pub fn to_ranges<I: IntoIterator<Item=u32>>(it: I) -> Vec<(u32, u32)> {
    let mut codepoints: Vec<u32> = it.into_iter().collect();
    codepoints.sort();
    codepoints.dedup();

    let mut ranges = vec![];
    for cp in codepoints {
        range_add(&mut ranges, cp);
    }
    ranges
}

/// Push a codepoint onto a vec of ranges. If the codepoint belongs to the
/// most recently added range, then increase the range. Otherwise, add a new
/// range containing only the codepoint given.
///
/// This panics if the given codepoint is already in the ranges or if a
/// codepoint is given out of order.
pub fn range_add(ranges: &mut Vec<(u32, u32)>, codepoint: u32) {
    if let Some(&mut (_, ref mut end)) = ranges.last_mut() {
        assert!(*end < codepoint);
        if codepoint == *end + 1 {
            *end = codepoint;
            return;
        }
    }
    ranges.push((codepoint, codepoint));
}

/// Convert an iterator of codepoint-value associations into a vec of sorted
/// ranges.
///
/// This panics if the same codepoint is present multiple times.
pub fn to_range_values<I>(it: I) -> Vec<(u32, u32, u64)>
    where I: IntoIterator<Item=(u32, u64)>
{
    let mut codepoints: Vec<(u32, u64)> = it.into_iter().collect();
    codepoints.sort();
    codepoints.dedup();

    let mut ranges = vec![];
    for (cp, value) in codepoints {
        range_value_add(&mut ranges, cp, value);
    }
    ranges
}

/// Push a codepoint associated with a value onto a vec of ranges. If the
/// codepoint belongs to the most recently added range and its value
/// corresponds to the range's value, then increase the range to include this
/// codepoint. Otherwise, add a new range containingly only the codepoint and
/// value given.
///
/// This panics if the given codepoint is already in the ranges or if a
/// codepoint is given out of order.
pub fn range_value_add(
    ranges: &mut Vec<(u32, u32, u64)>,
    codepoint: u32,
    value: u64,
) {
    if let Some(&mut (_, ref mut end, value2)) = ranges.last_mut() {
        assert!(*end < codepoint);
        if codepoint == *end + 1 && value == value2 {
            *end = codepoint;
            return;
        }
    }
    ranges.push((codepoint, codepoint, value));
}
