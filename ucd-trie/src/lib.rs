// BREADCRUMBS:
//
// Continue polishing this crate. Write some documentation, but don't overdo
// it.
//
// Next step is setting up codegen in ucd-generate, and then write benchmarks
// using the trie approach. Let's hope it smokes the competition.
//
// I think the big hitch here is that it would be nice for the trie to just
// be on giant &[u8], which probably would use less size on disk. But doing
// that without sacrificing performance is a little tricky. See src/set2.rs.


#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate core;

use core::fmt;

#[cfg(feature = "std")]
pub use owned::TrieSetOwned;

#[cfg(feature = "std")]
mod owned;

const CHUNK_SIZE: usize = 64;

pub type TrieSet = TrieSetSlice<'static>;

#[derive(Clone, Copy)]
pub struct TrieSetSlice<'a> {
    pub tree1_level1: &'a [u64],
    pub tree2_level1: &'a [u8],
    pub tree2_level2: &'a [u64],
    pub tree3_level1: &'a [u8],
    pub tree3_level2: &'a [u8],
    pub tree3_level3: &'a [u64],
}

impl<'a> fmt::Debug for TrieSetSlice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TrieSetSlice(...)")
    }
}

impl<'a> TrieSetSlice<'a> {
    pub fn contains_char(&self, c: char) -> bool {
        self.contains(c as usize)
    }

    pub fn contains_u32(&self, cp: u32) -> bool {
        self.contains(cp as usize)
    }

    #[inline(always)]
    fn contains(&self, cp: usize) -> bool {
        if cp < 0x800 {
            self.chunk_contains(cp, self.tree1_level1[cp >> 6])
        } else if cp < 0x10000 {
            let leaf = self.tree2_level1[(cp >> 6) - 0x20];
            self.chunk_contains(cp, self.tree2_level2[leaf as usize])
        } else {
            let child = self.tree3_level1[(cp >> 12) - 0x10];
            let i = ((child as usize) * CHUNK_SIZE) + ((cp >> 6) & 0b111111);
            let leaf = self.tree3_level2[i];
            self.chunk_contains(cp, self.tree3_level3[leaf as usize])
        }
    }

    #[inline(always)]
    fn chunk_contains(&self, cp: usize, chunk: u64) -> bool {
        ((chunk >> (cp & 0b111111)) & 1) != 0
    }
}
