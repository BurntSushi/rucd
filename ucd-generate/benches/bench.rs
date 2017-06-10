#![feature(test)]

extern crate byteorder;
extern crate fst;
#[macro_use]
extern crate lazy_static;
extern crate test;

use byteorder::{ByteOrder, BigEndian as BE};
use test::Bencher;

mod tables;

#[bench]
fn jamo_short_name_fst(b: &mut Bencher) {
    let slice = tables::jamo_short_name::JAMO_SHORT_NAME_SLICE;
    let fst = &tables::jamo_short_name::JAMO_SHORT_NAME_FST;
    let mut i = 0;
    b.iter(|| {
        let cp = slice[i].0;
        i = (i + 1) % slice.len();

        let mut key = [0; 4];
        BE::write_u32(&mut key, cp);
        let found = fst.get(key).unwrap();
        assert!(cp == 4363 || found > 0);
    });
}

#[bench]
fn jamo_short_name_slice(b: &mut Bencher) {
    let slice = tables::jamo_short_name::JAMO_SHORT_NAME_SLICE;
    let mut i = 0;
    b.iter(|| {
        let cp = slice[i].0;
        let name = slice[i].1;
        i = (i + 1) % slice.len();

        let found = slice[slice.binary_search_by_key(&cp, |x| x.0).unwrap()];
        assert_eq!(found.1, name);
    });
}
