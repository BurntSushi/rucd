// BREADCRUMBS:
//
// OK, so we're kind of at an inflection point here.
//
// One path is to build out rucd. Perhaps there are a few simple things we
// can add. e.g., Search by codepoint, UTF-8 conversions, view more complete
// codepoint info, case conversions, normalization, etc.
//
// Another path is to start adding Unicode support to regex-syntax-2. Start
// with case folding and building out the AST to handle this:
// http://www.unicode.org/reports/tr18/#Categories
// Be more principled with handling ranges of codepoints. Write more fine
// grained tests. Do more things in place.
//
// It's looking like we're not going to use FSTs in regex-syntax-2. If we
// committed to adding `\N{...}`, then I think we would use FSTs for that,
// but I think it's best to punt on that for now.

use byteorder::{ByteOrder, BE};

use tables::slice::case_folding_simple::{CASE_FOLDING_SIMPLE as CASE_SLICE};
use tables::fst::case_folding_simple::{CASE_FOLDING_SIMPLE as CASE_FST};

/// TODO
pub fn simple_fold_fst(c: char) -> SimpleFoldFstIter {
    SimpleFoldFstIter { start: c, cur: c }
}

/// TODO
#[derive(Debug)]
pub struct SimpleFoldFstIter {
    start: char,
    cur: char,
}

impl Iterator for SimpleFoldFstIter {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        use std::char;

        let next = match CASE_FST.get(&codepoint_key(self.cur)) {
            None => return None,
            Some(next) => char::from_u32(next as u32).unwrap(),
        };
        if next == self.start {
            None
        } else {
            self.cur = next;
            Some(next)
        }
    }
}

fn codepoint_key(cp: char) -> [u8; 4] {
    let mut key = [0; 4];
    BE::write_u32(&mut key, cp as u32);
    key
}

/// Return an iterator over the equivalence class of simple case mappings for
/// the given codepoint. The equivalence class does not include the given
/// codepoint.
pub fn simple_fold(c: char) -> SimpleFoldIter {
    let slice = CASE_SLICE
        .binary_search_by_key(&c, |&(c1, _)| c1)
        .ok()
        .map(|i| CASE_SLICE[i].1)
        .unwrap_or(&[]);
    SimpleFoldIter(slice.iter())
}

/// An iterator over a codepoint's simple case equivalence class.
#[derive(Debug)]
pub struct SimpleFoldIter(::std::slice::Iter<'static, char>);

impl Iterator for SimpleFoldIter {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        self.0.next().map(|c| *c)
    }
}

/*
/// Return an iterator over the equivalence class of simple case mappings for
/// the given codepoint. The equivalence class does not include the given
/// codepoint.
pub fn simple_fold(c: char) -> SimpleFoldIter {
    let cur = simple_fold_search(c).ok();
    SimpleFoldIter { start: c, cur: cur }
}

/// An iterator over a codepoint's simple case equivalence class.
#[derive(Debug)]
pub struct SimpleFoldIter {
    start: char,
    cur: Option<usize>,
}

impl Iterator for SimpleFoldIter {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        let cur = match self.cur {
            None => return None,
            Some(cur) => cur,
        };
        let (c1, c2) = CASE_SLICE[cur];
        if c1 != self.start {
            self.cur = None;
            return None;
        }
        self.cur = Some(cur + 1);
        Some(c2)
    }
}

fn simple_fold_search(c: char) -> Result<usize, usize> {
    let i = binary_search(CASE_SLICE, |&(c1, _)| c <= c1);
    if i >= CASE_SLICE.len() || CASE_SLICE[i].0 != c {
        Err(i)
    } else {
        Ok(i)
    }
}

/// Binary search to find first element such that `pred(T) == true`.
///
/// Assumes that if `pred(xs[i]) == true` then `pred(xs[i+1]) == true`.
///
/// If all elements yield `pred(T) == false`, then `xs.len()` is returned.
///
/// We implement this ourselves because the case folding table contains
/// multiple entries with the same key, and the standard library binary search
/// doesn't make any guarantees about which key it finds. This binary search
/// always finds the first one.
fn binary_search<T, F>(xs: &[T], mut pred: F) -> usize
        where F: FnMut(&T) -> bool {
    let (mut left, mut right) = (0, xs.len());
    while left < right {
        let mid = (left + right) / 2;
        if pred(unsafe { xs.get_unchecked(mid) }) {
            right = mid;
        } else {
            left = mid + 1;
        }
    }
    left
}
*/

#[cfg(test)]
mod tests {
    use super::{simple_fold, simple_fold_fst};

    #[test]
    fn simple_fold_k() {
        let xs: Vec<char> = simple_fold('k').collect();
        assert_eq!(xs, vec!['K', 'K']);

        let xs: Vec<char> = simple_fold('K').collect();
        assert_eq!(xs, vec!['k', 'K']);

        let xs: Vec<char> = simple_fold('K').collect();
        assert_eq!(xs, vec!['K', 'k']);
    }

    #[test]
    fn simple_fold_a() {
        let xs: Vec<char> = simple_fold('a').collect();
        assert_eq!(xs, vec!['A']);

        let xs: Vec<char> = simple_fold('A').collect();
        assert_eq!(xs, vec!['a']);
    }

    #[test]
    fn simple_fold_question() {
        let xs: Vec<char> = simple_fold('?').collect();
        assert_eq!(xs, vec![]);
    }

    #[test]
    fn simple_fold_fst_k() {
        let mut xs: Vec<char> = simple_fold_fst('k').collect();
        xs.sort();
        assert_eq!(xs, vec!['K', 'K']);

        let mut xs: Vec<char> = simple_fold_fst('K').collect();
        xs.sort();
        assert_eq!(xs, vec!['k', 'K']);

        let mut xs: Vec<char> = simple_fold_fst('K').collect();
        xs.sort();
        assert_eq!(xs, vec!['K', 'k']);
    }

    #[test]
    fn simple_fold_fst_a() {
        let xs: Vec<char> = simple_fold_fst('a').collect();
        assert_eq!(xs, vec!['A']);

        let xs: Vec<char> = simple_fold_fst('A').collect();
        assert_eq!(xs, vec!['a']);
    }

    #[test]
    fn simple_fold_fst_question() {
        let xs: Vec<char> = simple_fold_fst('?').collect();
        assert_eq!(xs, vec![]);
    }
}
