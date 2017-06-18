use std::collections::HashMap;

const CHUNK_SIZE: usize = 64;
const CHUNKS: usize = 0x110000 / CHUNK_SIZE;

pub struct TrieSet {
    oneortwo: OneOrTwo,
    three: Three,
    four: Four,
}

struct OneOrTwo(Vec<u64>);

struct Three {
    level1: Vec<u8>,
    level2: Vec<u64>,
}

struct Four {
    level1: Vec<u8>,
    level2: Vec<u8>,
    level3: Vec<u64>,
}

pub struct TrieSetSlice<'a> {
    oneortwo: OneOrTwoSlice<'a>,
    three: ThreeSlice<'a>,
    four: FourSlice<'a>,
}

struct OneOrTwoSlice<'a>(&'a [u64]);

struct ThreeSlice<'a> {
    level1: &'a [u8],
    level2: &'a [u64],
}

struct FourSlice<'a> {
    level1: &'a [u8],
    level2: &'a [u8],
    level3: &'a [u64],
}

impl TrieSet {
    pub fn from_scalars(scalars: &[char]) -> TrieSet {
        let mut all = vec![false; 0x110000];
        for &s in scalars {
            all[s as usize] = true;
        }
        TrieSet::new(&all)
    }

    pub fn from_codepoints(codepoints: &[u32]) -> TrieSet {
        let mut all = vec![false; 0x110000];
        for &cp in codepoints {
            all[cp as usize] = true;
        }
        TrieSet::new(&all)
    }

    pub fn as_slice(&self) -> TrieSetSlice {
        TrieSetSlice {
            oneortwo: OneOrTwoSlice(&self.oneortwo.0),
            three: ThreeSlice {
                level1: &self.three.level1,
                level2: &self.three.level2,
            },
            four: FourSlice {
                level1: &self.four.level1,
                level2: &self.four.level2,
                level3: &self.four.level3,
            },
        }
    }

    fn new(all: &[bool]) -> TrieSet {
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

        let oneortwo = OneOrTwo(
            bitvectors.iter()
                .cloned()
                .take(0x800 / CHUNK_SIZE)
                .collect());

        let (level1, level2) = compress_postfix_leaves(
            &bitvectors[0x800 / CHUNK_SIZE..0x10000 / CHUNK_SIZE]);
        let three = Three { level1: level1, level2: level2 };

        let (mid, level3) = compress_postfix_leaves(
            &bitvectors[0x10000 / CHUNK_SIZE..0x110000 / CHUNK_SIZE]);
        let (level1, level2) = compress_postfix_mid(&mid, 64);
        let four = Four {
            level1: level1, level2: level2, level3: level3,
        };

        TrieSet {
            oneortwo: oneortwo,
            three: three,
            four: four,
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
        if cp < 0x800 {
            self.chunk_contains(cp, self.oneortwo.0[cp >> 6])
        } else if cp < 0x10000 {
            let leaf = self.three.level1[(cp >> 6) - 0x20];
            self.chunk_contains(cp, self.three.level2[leaf as usize])
        } else {
            let child = self.four.level1[(cp >> 12) - 0x10];
            let i = ((child as usize) << 6) + ((cp >> 6) & 0x3F);
            let leaf = self.four.level2[i];
            self.chunk_contains(cp, self.four.level3[leaf as usize])
        }
    }

    #[inline(always)]
    fn chunk_contains(&self, cp: usize, chunk: u64) -> bool {
        ((chunk >> (cp & 63)) & 1) != 0
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
            self.chunk_contains(cp, self.oneortwo.0[cp >> 6])
        } else if cp < 0x10000 {
            let leaf = self.three.level1[(cp >> 6) - 0x20];
            self.chunk_contains(cp, self.three.level2[leaf as usize])
        } else {
            let child = self.four.level1[(cp >> 12) - 0x10];
            let i = ((child as usize) << 6) + ((cp >> 6) & 0x3F);
            let leaf = self.four.level2[i];
            self.chunk_contains(cp, self.four.level3[leaf as usize])
        }
    }

    #[inline(always)]
    fn chunk_contains(&self, cp: usize, chunk: u64) -> bool {
        ((chunk >> (cp & 63)) & 1) != 0
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
    use super::TrieSet;

    #[test]
    fn set1() {
        let set = TrieSet::from_scalars(&['a']);
        assert!(set.contains_char('a'));
        assert!(!set.contains_char('b'));
        assert!(!set.contains_char('β'));
        assert!(!set.contains_char('☃'));
        assert!(!set.contains_char('😼'));
    }

    #[test]
    fn set_combined() {
        let set = TrieSet::from_scalars(&['a', 'b', 'β', '☃', '😼']);
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
