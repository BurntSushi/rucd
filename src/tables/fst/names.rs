// DO NOT EDIT THIS FILE. IT WAS AUTOMATICALLY GENERATED BY:
//
//  ucd-generate names /tmp/ucd-9.0.0/ --rust-fst --tagged
//
// ucd-generate is available on crates.io.

use fst::raw::Fst;
use fst::Map;

lazy_static! {
  pub static ref NAMES: Map =
    Map::from(Fst::from_static_slice(
      NAMES_BYTES).unwrap());
}

const NAMES_BYTES: &'static [u8] = include_bytes!("names.fst");