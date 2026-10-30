//! Checks the invariants of a run's census as part of `validate`, which
//! debug builds run before anything reads it: each source's place, the order and context of every name hit, the qualified
//! paths and their indexes, and the declarations.

use crate::analysis::AnalysisRun;
use crate::census::{Context, Scan, Source};
use crate::census_lex::is_keyword;

/// Tells whether a hit in `context` may be in `source`: a lexed source of
/// that context, or a test-only node of a module tree's file.
fn counted(context: Context, source: &Source) -> bool {
    source.scan == Scan::Lexed
        && (context == source.context
            || (source.context == Context::Production && context == Context::Test))
}

impl AnalysisRun<'_> {
    /// Checks the census: its sources, every hit, the qualified paths and
    /// their indexes, and the declarations.
    pub(crate) fn check_census(&self) -> Result<(), String> {
        self.check_sources()?;
        self.check_tokens()?;
        self.check_paths()
    }

    /// Checks that each tree file has its source, that each source is in
    /// its place.
    fn check_sources(&self) -> Result<(), String> {
        let census = &self.census;
        for (t, krate) in self.krates.iter().enumerate() {
            for (f, file) in krate.files.iter().enumerate() {
                let source = census.sources.get(census.tree_source(t, f));
                let fits = source.is_some_and(|s| {
                    s.target == Some(t)
                        && s.file == Some(f)
                        && s.rel == file.rel
                        && s.context == Context::Production
                        && s.package == self.targets[t].package
                });
                if !fits {
                    return Err(format!("{} of {} has no tree source", file.rel, krate.name));
                }
            }
        }
        for source in &census.sources {
            let target = source.target.is_none_or(|t| {
                self.targets
                    .get(t)
                    .is_some_and(|info| info.package == source.package)
            });
            let file = source.file.is_none_or(|f| {
                (source.target).is_some_and(|t| {
                    self.krates[t]
                        .files
                        .get(f)
                        .is_some_and(|file| file.rel == source.rel)
                })
            });
            let placed = match source.context {
                Context::Production | Context::Doctest => source.file.is_some(),
                _ => source.file.is_none(),
            };
            if !(target && file && placed && source.line > 0) {
                return Err(format!("source {} is out of place", source.rel));
            }
        }
        Ok(())
    }

    /// Checks the order, count, and context of every name hit, and that
    /// each declaration's name is hit at least once per declaration when
    /// every source was read.
    fn check_tokens(&self) -> Result<(), String> {
        let census = &self.census;
        for (name, hits) in &census.tokens {
            let ordered = hits.windows(2).all(|w| {
                (w[0].source, w[0].context, w[0].form, w[0].in_macro)
                    < (w[1].source, w[1].context, w[1].form, w[1].in_macro)
            });
            let placed = hits.iter().all(|hit| {
                hit.count > 0
                    && census
                        .sources
                        .get(hit.source)
                        .is_some_and(|s| counted(hit.context, s))
            });
            let count = census.count(name);
            let short = census.fully_read() && count.total() < count.declared;
            if name.is_empty() || !ordered || !placed || short {
                return Err(format!(
                    "the hits of {name} are malformed or miss a declaration"
                ));
            }
        }
        for (name, &declared) in &census.declarations {
            let unseen = census.fully_read() && !census.tokens.contains_key(name);
            if name.is_empty() || declared == 0 || census.declared(name) != declared || unseen {
                return Err(format!("the census misses declarations of {name}"));
            }
        }
        Ok(())
    }

    /// Checks that the qualified paths are sorted, each in its source and
    /// module, with its first segment hit where the path starts, and that
    /// both indexes hold every path once under the right segment.
    fn check_paths(&self) -> Result<(), String> {
        let census = &self.census;
        if !census.paths.windows(2).all(|w| w[0] <= w[1]) {
            return Err("the qualified paths are not sorted".to_owned());
        }
        for path in &census.paths {
            let Some(source) = census.sources.get(path.source) else {
                return Err("a qualified path has no source".to_owned());
            };
            let first = path.segments.first().map_or("", String::as_str);
            let hits = census.token_hits(first);
            let from = hits.partition_point(|hit| hit.source < path.source);
            let named = is_keyword(first)
                || hits[from..]
                    .iter()
                    .take_while(|hit| hit.source == path.source)
                    .any(|hit| hit.context == path.context && hit.in_macro == path.in_macro);
            let module = path.module.is_none_or(|m| {
                let held = |t: usize, f: usize| {
                    self.krates[t]
                        .modules
                        .get(m)
                        .is_some_and(|module| module.file == f)
                };
                source.context == Context::Production
                    && source
                        .target
                        .zip(source.file)
                        .is_some_and(|(t, f)| held(t, f))
            });
            let segments = path.segments.len() > 1
                && path
                    .segments
                    .iter()
                    .all(|s| !s.is_empty() && !s.starts_with("r#"));
            if !(named
                && module
                && segments
                && counted(path.context, source)
                && path.line >= source.line)
            {
                return Err(format!(
                    "the qualified path {} is malformed",
                    path.segments.join("::")
                ));
            }
        }
        for (index, last) in [(&census.by_last, true), (&census.by_first, false)] {
            let mut total = 0;
            for (name, list) in index {
                let segment = |p: usize| {
                    let segments = &census.paths[p].segments;
                    if last {
                        segments.last()
                    } else {
                        segments.first()
                    }
                };
                let found = if last {
                    census.paths_ending(name).count()
                } else {
                    census.paths_starting(name).count()
                };
                let fits = list.windows(2).all(|w| w[0] < w[1])
                    && list
                        .iter()
                        .all(|&p| p < census.paths.len() && segment(p) == Some(name))
                    && found == list.len();
                if !fits {
                    return Err(format!("the path index of {name} is malformed"));
                }
                total += list.len();
            }
            if total != census.paths.len() {
                return Err("a qualified path is not indexed".to_owned());
            }
        }
        Ok(())
    }
}
