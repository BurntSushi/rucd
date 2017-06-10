#![feature(test)]

extern crate byteorder;
extern crate fst;
#[macro_use]
extern crate lazy_static;
extern crate test;

use byteorder::{ByteOrder, BigEndian as BE};
use test::Bencher;

mod tables;

fn codepoint_key(cp: u32) -> [u8; 4] {
    let mut key = [0; 4];
    BE::write_u32(&mut key, cp);
    key
}

#[bench]
fn jamo_short_name_fst(b: &mut Bencher) {
    let slice = tables::jamo_short_name_slice::JAMO_SHORT_NAME;
    let fst = &tables::jamo_short_name_fst::JAMO_SHORT_NAME;
    let mut i = 0;
    let mut value = String::new();
    b.iter(|| {
        let (cp, name) = slice[i];
        i = (i + 1) % slice.len();

        let mut found = fst.get(codepoint_key(cp)).unwrap();
        value.clear();
        while found != 0 {
            value.push((found & 0xFF) as u8 as char);
            found = found >> 8;
        }
        assert_eq!(value, name);
    });
}

#[bench]
fn jamo_short_name_slice(b: &mut Bencher) {
    let slice = tables::jamo_short_name_slice::JAMO_SHORT_NAME;
    let mut i = 0;
    b.iter(|| {
        let (cp, name) = slice[i];
        i = (i + 1) % slice.len();

        let found = slice[slice.binary_search_by_key(&cp, |x| x.0).unwrap()];
        assert_eq!(found.1, name);
    });
}

#[bench]
fn jamo_short_name_slice_linear(b: &mut Bencher) {
    let slice = tables::jamo_short_name_slice::JAMO_SHORT_NAME;
    let mut i = 0;
    b.iter(|| {
        let (cp, name) = slice[i];
        i = (i + 1) % slice.len();

        let found = slice.iter().find(|p| p.0 == cp).unwrap();
        assert_eq!(found.1, name);
    });
}
