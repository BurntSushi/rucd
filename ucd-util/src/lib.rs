/*!
The `ucd-util` crate contains a smattering of utility functions that implement
various algorithms specified by Unicode. There is no specific goal for
exhaustiveness. Instead, implementations should be added on an as-needed basis.
*/

#![deny(missing_docs)]

extern crate byteorder;
extern crate fst;
#[macro_use]
extern crate lazy_static;

mod tables;

mod case;
mod hangul;
mod ideograph;
mod name;
mod property;

pub use case::{
    SimpleFoldIter, SimpleFoldFstIter,
    simple_fold, simple_fold_fst,
};
pub use hangul::{
    RANGE_HANGUL_SYLLABLE, hangul_name, hangul_full_canonical_decomposition,
};
pub use ideograph::{RANGE_IDEOGRAPH, ideograph_name};
pub use name::{character_name_normalize, symbolic_name_normalize};
pub use property::{
    PropertyTable, PropertyValueTable, PropertyValues,
    canonical_property_name, property_values, canonical_property_value,
};
