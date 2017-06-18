#![feature(test)]

extern crate test;
extern crate ucd_trie;

use ucd_trie::TrieSet;

#[bench]
fn bench_trie_set(b: &mut test::Bencher) {
    // let chars = &['a', 'β', '☃', '😼'];
    let chars = &['a'];
    let set = TrieSet::from_scalars(chars);

    let mut i = 0;
    b.iter(|| {
        let c = chars[i];
        i = (i + 1) % chars.len();

        for _ in 0..1000 {
            assert!(set.contains_char(c));
        }
    });
}

#[bench]
fn bench_trie_set_slice(b: &mut test::Bencher) {
    // let chars = &['a', 'β', '☃', '😼'];
    let chars = &['a'];
    let set = TrieSet::from_scalars(chars);
    let set = set.as_slice();

    let mut i = 0;
    b.iter(|| {
        let c = chars[i];
        i = (i + 1) % chars.len();

        for _ in 0..1000 {
            assert!(set.contains_char(c));
        }
    });
}
