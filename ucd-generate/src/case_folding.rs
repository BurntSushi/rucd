use std::collections::BTreeMap;

use ucd_parse::{self, CaseFold, CaseStatus, Codepoint};

use args::ArgMatches;
use error::Result;

pub fn command(args: ArgMatches) -> Result<()> {
    let dir = args.ucd_dir()?;
    let case_folding: BTreeMap<Codepoint, Vec<CaseFold>> =
        ucd_parse::parse_many_by_codepoint(dir)?;

    let mut wtr = args.writer("case_folding")?;
    if args.is_present("full") {
        let mut table = BTreeMap::new();
        for (&cp, case_folds) in &case_folding {
            let mapping_cp = match choose_fold(case_folds, true)? {
                None => continue,
                Some(case_fold) => &case_fold.mapping,
            };
            let mapping_u32 = mapping_cp.iter().map(|cp| cp.value()).collect();
            table.insert(cp.value(), mapping_u32);
        }
        wtr.codepoint_to_codepoints(args.name(), &table)?;
    } else {
        let mut table = BTreeMap::new();
        for (&cp, case_folds) in &case_folding {
            let mapping_cp = match choose_fold(case_folds, false)? {
                None => continue,
                Some(case_fold) => &case_fold.mapping,
            };
            assert_eq!(mapping_cp.len(), 1);
            table.insert(cp.value(), mapping_cp[0].value());
        }
        wtr.codepoint_to_codepoint(args.name(), &table)?;
    }
    Ok(())


    // let cp = Codepoint::from_u32(0x0130).unwrap();
    // println!("{:?}", case_folding[&cp]);
    // let mut map = BTreeMap::new();
    // for (cp, jamo) in jamo_map {
        // map.insert(cp.value(), jamo.name);
    // }
    // wtr.codepoint_to_string(args.name(), &map)?;
    // Ok(())
}

fn choose_fold(
    case_folds: &[CaseFold],
    full: bool,
) -> Result<Option<&CaseFold>> {
    let mut choice = None;
    for case_fold in case_folds {
        if (full && case_fold.status == CaseStatus::Full)
            || (!full && case_fold.status == CaseStatus::Simple)
            || case_fold.status == CaseStatus::Common
        {
            if choice.is_some() {
                return err!("found multiple matches from: {:?}", case_folds);
            }
            choice = Some(case_fold);
        }
    }
    Ok(choice)
}
