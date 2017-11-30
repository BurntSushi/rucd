use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fmt;
use std::io;

use clap;
use fst::{self, Streamer};
use regex::bytes::{Regex, RegexBuilder};

use codepoint::Codepoint;
use display::ShortWriter;
use error::Result;
use tables::fst::names::NAMES;

pub fn command(args: &clap::ArgMatches) -> Result<()> {
    let mut search_builder = SearchBuilder::new();
    search_builder
        .case_insensitive(!args.is_present("case-sensitive"));

    if let Some(os_pat) = args.value_of_os("pattern") {
        search_builder.pattern(Some(pattern_to_str(os_pat)?));
    }

    let searcher = search_builder.build()?;
    let results = ResultSink::from_search(searcher);
    if !args.is_present("allow-large") && results.len() > 10_000 {
        return err!("{} results found, which is too large to print. \
                     Pass the -A flag to forcefully print them.",
                     results.len());
    }
    let mut wtr = ShortWriter::new(io::stdout());
    for cp in results.codepoints {
        wtr.write_codepoint(cp)?;
    }
    wtr.flush()?;
    Ok(())
}

#[derive(Debug)]
struct ResultSink {
    codepoints: BTreeSet<Codepoint>,
}

impl ResultSink {
    fn from_search(search: Search) -> ResultSink {
        ResultSink {
            codepoints: search.collect(),
        }
    }

    fn len(&self) -> usize {
        self.codepoints.len()
    }
}

#[derive(Debug)]
struct Search {
    source: SearchSource,
}

impl Iterator for Search {
    type Item = Codepoint;

    fn next(&mut self) -> Option<Codepoint> {
        self.source.next()
    }
}

enum SearchSource {
    AllCodepoints(u32),
    ByPatternMatch {
        re: Regex,
        stream: fst::map::Stream<'static>,
    },
}

impl Iterator for SearchSource {
    type Item = Codepoint;

    fn next(&mut self) -> Option<Codepoint> {
        match *self {
            SearchSource::AllCodepoints(ref mut cp) => {
                let next = match Codepoint::from_u32(*cp) {
                    Err(_) => return None,
                    Ok(cp) => cp,
                };
                *cp += 1;
                Some(next)
            }
            SearchSource::ByPatternMatch { ref re, ref mut stream } => {
                loop {
                    let (name, tagged) = match stream.next() {
                        None => return None,
                        Some(x) => x,
                    };
                    if !re.is_match(name) {
                        continue;
                    }
                    return Some(Codepoint::from_u64(tagged).unwrap());
                }
            }
        }
    }
}

#[derive(Debug)]
struct SearchBuilder {
    pattern: Option<String>,
    case_insensitive: bool,
}

impl SearchBuilder {
    fn new() -> SearchBuilder {
        SearchBuilder::default()
    }

    fn build(&self) -> Result<Search> {
        let source = match self.pattern {
            None => SearchSource::AllCodepoints(0),
            Some(ref pattern) => {
                let re = RegexBuilder::new(pattern)
                    .case_insensitive(self.case_insensitive)
                    .build()?;
                SearchSource::ByPatternMatch {
                    re: re,
                    stream: NAMES.stream(),
                }
            }
        };
        Ok(Search {
            source: source,
        })
    }

    fn pattern<S: AsRef<str>>(
        &mut self,
        pattern: Option<S>,
    ) -> &mut SearchBuilder {
        self.pattern = pattern.map(|s| s.as_ref().to_string());
        self
    }

    fn case_insensitive(&mut self, yes: bool) -> &mut SearchBuilder {
        self.case_insensitive = yes;
        self
    }
}

impl fmt::Debug for SearchSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SearchSource::AllCodepoints(cp) => {
                write!(f, "SearchSource::AllCodepoints({:?})", cp)
            }
            SearchSource::ByPatternMatch { ref re, .. } => {
                write!(f, "SearchSource::ByPatternMatch({:?})", re)
            }
        }
    }
}

impl Default for SearchBuilder {
    fn default() -> SearchBuilder {
        SearchBuilder {
            pattern: None,
            case_insensitive: true,
        }
    }
}

fn pattern_to_str(s: &OsStr) -> Result<&str> {
    match s.to_str() {
        Some(s) => Ok(s),
        None => err!(
            "Argument '{}' is not valid UTF-8. \
             Use hex escape sequences to match arbitrary \
             bytes in a pattern (e.g., \\xFF).",
             s.to_string_lossy()),
    }
}
