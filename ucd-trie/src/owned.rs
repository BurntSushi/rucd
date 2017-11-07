use std::collections::HashMap;
use std::fmt;

use super::{CHUNK_SIZE, TrieSetSlice};

const CHUNKS: usize = 0x110000 / CHUNK_SIZE;

#[derive(Clone)]
pub struct TrieSetOwned {
    pub tree1_level1: Vec<u64>,
    pub tree2_level1: Vec<u8>,
    pub tree2_level2: Vec<u64>,
    pub tree3_level1: Vec<u8>,
    pub tree3_level2: Vec<u8>,
    pub tree3_level3: Vec<u64>,
}

impl fmt::Debug for TrieSetOwned {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TrieSetOwned(...)")
    }
}

impl TrieSetOwned {
    fn new(all: &[bool]) -> TrieSetOwned {
        let mut bitvectors = Vec::with_capacity(CHUNKS);
        for i in 0..CHUNKS {
            let mut bitvector = 0u64;
            for j in 0..CHUNK_SIZE {
                if all[i * CHUNK_SIZE + j] {
                    bitvector |= 1 << j;
                }
            }
            bitvectors.push(bitvector);
        }

        let tree1_level1 = bitvectors
            .iter()
            .cloned()
            .take(0x800 / CHUNK_SIZE)
            .collect();

        let (tree2_level1, tree2_level2) = compress_postfix_leaves(
            &bitvectors[0x800 / CHUNK_SIZE..0x10000 / CHUNK_SIZE]);

        let (mid, tree3_level3) = compress_postfix_leaves(
            &bitvectors[0x10000 / CHUNK_SIZE..0x110000 / CHUNK_SIZE]);
        let (tree3_level1, tree3_level2) = compress_postfix_mid(&mid, 64);

        TrieSetOwned {
            tree1_level1,
            tree2_level1,
            tree2_level2,
            tree3_level1,
            tree3_level2,
            tree3_level3,
        }
    }

    pub fn from_scalars(scalars: &[char]) -> TrieSetOwned {
        let mut all = vec![false; 0x110000];
        for &s in scalars {
            all[s as usize] = true;
        }
        TrieSetOwned::new(&all)
    }

    pub fn from_codepoints(codepoints: &[u32]) -> TrieSetOwned {
        let mut all = vec![false; 0x110000];
        for &cp in codepoints {
            all[cp as usize] = true;
        }
        TrieSetOwned::new(&all)
    }

    #[inline(always)]
    pub fn as_slice(&self) -> TrieSetSlice {
        TrieSetSlice {
            tree1_level1: &self.tree1_level1,
            tree2_level1: &self.tree2_level1,
            tree2_level2: &self.tree2_level2,
            tree3_level1: &self.tree3_level1,
            tree3_level2: &self.tree3_level2,
            tree3_level3: &self.tree3_level3,
        }
    }

    pub fn contains_char(&self, c: char) -> bool {
        self.contains(c as usize)
    }

    pub fn contains_u32(&self, cp: u32) -> bool {
        self.contains(cp as usize)
    }

    #[inline(always)]
    fn contains(&self, cp: usize) -> bool {
        self.as_slice().contains(cp)
    }
}

fn compress_postfix_leaves(chunks: &[u64]) -> (Vec<u8>, Vec<u64>) {
    let mut root = vec![];
    let mut children = vec![];
    let mut bychild = HashMap::new();
    for &chunk in chunks {
        if !bychild.contains_key(&chunk) {
            let start = bychild.len();
            assert!(start < ::std::u8::MAX as usize);
            bychild.insert(chunk, start as u8);
            children.push(chunk);
        }
        root.push(bychild[&chunk]);
    }
    (root, children)
}

fn compress_postfix_mid(
    chunks: &[u8],
    chunk_size: usize,
) -> (Vec<u8>, Vec<u8>) {
    let mut root = vec![];
    let mut children = vec![];
    let mut bychild = HashMap::new();
    for i in 0..(chunks.len() / chunk_size) {
        let chunk = &chunks[i * chunk_size..(i+1) * chunk_size];
        if !bychild.contains_key(chunk) {
            let start = bychild.len();
            assert!(start < ::std::u8::MAX as usize);
            bychild.insert(chunk, start as u8);
            children.extend(chunk);
        }
        root.push(bychild[chunk]);
    }
    (root, children)
}

#[cfg(test)]
mod tests {
    use super::TrieSetOwned;

    #[test]
    fn set1() {
        let set = TrieSetOwned::from_scalars(&['a']);
        assert!(set.contains_char('a'));
        assert!(!set.contains_char('b'));
        assert!(!set.contains_char('β'));
        assert!(!set.contains_char('☃'));
        assert!(!set.contains_char('😼'));
    }

    #[test]
    fn set_combined() {
        let set = TrieSetOwned::from_scalars(&['a', 'b', 'β', '☃', '😼']);
        assert!(set.contains_char('a'));
        assert!(set.contains_char('b'));
        assert!(set.contains_char('β'));
        assert!(set.contains_char('☃'));
        assert!(set.contains_char('😼'));

        assert!(!set.contains_char('c'));
        assert!(!set.contains_char('θ'));
        assert!(!set.contains_char('⛇'));
        assert!(!set.contains_char('🐲'));
    }
}
