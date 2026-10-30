//! Reads what analysis keeps from attributes beside the notes: the bytes of
//! each test-only node, raw doc text, lint attributes, and the paths that
//! `serde` attributes name.

use std::path::Path;

use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Attribute, Item, Meta, Token};

use crate::attrs::{self, string_value};
use crate::collect::{Collector, Here};
use crate::comments::{DocText, doc_text};
use crate::model::Loc;
use crate::model_body::AttrPath;
use crate::modtree::ModSource;
use crate::source::SourceFile;
use crate::typerefs::{range_of, strip_raw};

/// The `serde` attribute keys whose string value is a path the derived
/// code calls or reads.
pub(crate) const SERDE_PATH_KEYS: [&str; 6] = [
    "default",
    "deserialize_with",
    "getter",
    "serialize_with",
    "skip_serializing_if",
    "with",
];

impl Collector<'_> {
    /// Tells whether a node is test-only. When it is, records its bytes,
    /// which the census reads as test code.
    pub(crate) fn test_only<T: Spanned + ?Sized>(
        &mut self,
        here: &Here,
        attrs: &[Attribute],
        node: &T,
    ) -> bool {
        let test = attrs::is_test_only(attrs);
        if test {
            let range = range_of(node, &self.files[here.file]);
            self.test_ranges.push((here.file, range));
        }
        test
    }

    /// Returns a node's raw doc text and lint attributes, and records the
    /// paths its `serde` attributes name.
    pub(crate) fn attr_facts(
        &mut self,
        here: &Here,
        attrs: &[Attribute],
    ) -> (Option<String>, Vec<String>) {
        let files = self.files;
        let file = &files[here.file];
        for (key, global, path, line) in serde_paths(attrs, file) {
            let path_ref = self.analysis_ref(here, global, path.clone(), true);
            self.attr_paths.push(AttrPath {
                key,
                path,
                path_ref,
                loc: Loc {
                    file: here.file,
                    line,
                },
            });
        }
        (
            raw_doc(attrs, file, self.crate_dir),
            attrs::lints(attrs, file),
        )
    }

    /// Gives each module its raw doc text and lint attributes: those of its
    /// `mod` line, then its inner ones.
    pub(crate) fn module_facts(&mut self, sources: &[ModSource]) {
        let files = self.files;
        for (id, source) in sources.iter().enumerate() {
            let mut docs = Vec::new();
            let mut lints = Vec::new();
            let declared = source.decl.and_then(|(parent, index)| {
                let parent = &sources[parent];
                match parent.items.get(index) {
                    Some(Item::Mod(m)) => Some((&files[parent.file], m)),
                    _ => None,
                }
            });
            if let Some((file, m)) = declared {
                let (outer, _) = attrs::split_inner(&m.attrs);
                docs.extend(raw_doc(&outer, file, self.crate_dir));
                lints.extend(attrs::lints(&outer, file));
            }
            let file = &files[source.file];
            docs.extend(raw_doc(&source.inner, file, self.crate_dir));
            lints.extend(attrs::lints(&source.inner, file));
            attrs::sort_marks(&mut lints);
            let module = &mut self.modules[id];
            module.raw_doc = (!docs.is_empty()).then(|| docs.join("\n"));
            module.lints = lints;
        }
    }
}

/// Returns a node's doc text before flattening, for analysis: the lines of
/// each doc attribute in order, a written line without the one space after
/// `///`, a block doc beautified, and a file `include_str!` reads as it is.
/// The lines are joined by newlines, without blank lines at either end;
/// `None` when there is no doc text.
fn raw_doc(attrs: &[Attribute], file: &SourceFile, crate_dir: &Path) -> Option<String> {
    let mut lines: Vec<String> = Vec::new();
    for attr in attrs {
        match doc_text(attr, file, crate_dir) {
            Some(DocText::Written(written)) => lines.extend(written.iter().map(|line| {
                let line = line.trim_end();
                line.strip_prefix(' ').unwrap_or(line).to_owned()
            })),
            Some(DocText::Included(included)) => {
                lines.extend(included.iter().map(|line| line.trim_end().to_owned()));
            }
            Some(DocText::Unreadable(_)) | None => {}
        }
    }
    let first = lines.iter().position(|line| !line.is_empty())?;
    let last = lines.iter().rposition(|line| !line.is_empty())?;
    Some(lines[first..=last].join("\n"))
}

/// Returns each path a `serde` attribute names in a string, with its key,
/// whether it starts with `::`, and its line. A `serde` attribute inside a
/// `cfg_attr` that is not test-only counts.
fn serde_paths(attrs: &[Attribute], file: &SourceFile) -> Vec<(String, bool, Vec<String>, u32)> {
    let mut out = Vec::new();
    for meta in crate::macros::effective(attrs) {
        let Meta::List(list) = &meta else {
            continue;
        };
        if !list.path.is_ident("serde") {
            continue;
        }
        let Ok(items) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        else {
            continue;
        };
        for item in &items {
            let Meta::NameValue(pair) = item else {
                continue;
            };
            let key = SERDE_PATH_KEYS.iter().find(|key| pair.path.is_ident(key));
            let value = string_value(&pair.value);
            let (Some(key), Some(value)) = (key, value) else {
                continue;
            };
            let Ok(path) = syn::parse_str::<syn::Path>(&value) else {
                continue;
            };
            let segments = path
                .segments
                .iter()
                .map(|s| strip_raw(&s.ident.to_string()))
                .collect();
            let line = file.line_of(range_of(pair, file).start);
            out.push((
                (*key).to_owned(),
                path.leading_colon.is_some(),
                segments,
                line,
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use crate::collect_body_facts::tests::collected;

    #[test]
    fn raw_docs_keep_text_before_flattening() {
        let (_tmp, krate) = collected(
            "/// First, with [`Link`] kept.\n\
             ///\n\
             ///     indented code\n\
             /** A block\n * doc. */\n\
             #[doc = \"plain\"]\n\
             pub struct Link;\n\
             pub struct Bare;\n",
        );
        assert_eq!(
            krate.decls[0].meta.raw_doc.as_deref(),
            Some("First, with [`Link`] kept.\n\n    indented code\nA block\ndoc.\nplain")
        );
        assert_eq!(krate.decls[1].meta.raw_doc, None);
    }

    #[test]
    fn modules_get_docs_and_lints_of_the_mod_line_and_inner_ones() {
        let (_tmp, krate) = collected(
            "//! The root.\n\
             /// Outer.\n\
             #[allow(dead_code)]\n\
             pub mod inner { #![doc = \"Inside.\"] #![warn(unused)] }\n",
        );
        assert_eq!(krate.modules[0].raw_doc.as_deref(), Some("The root."));
        assert_eq!(krate.modules[1].raw_doc.as_deref(), Some("Outer.\nInside."));
        assert_eq!(krate.modules[1].lints, ["allow(dead_code)", "warn(unused)"]);
    }

    #[test]
    fn test_only_nodes_record_their_bytes_and_stay_out() {
        let lib = "pub struct Kept { pub a: u8, #[cfg(test)] pub b: u8 }\n\
                   #[cfg(test)] fn helper() {}\n\
                   #[cfg(test)]\nmod tests { fn t() {} }\n";
        let (_tmp, krate) = collected(lib);
        let text = &krate.files[0].text;
        let found: Vec<&str> = krate
            .test_ranges
            .iter()
            .map(|(file, range)| {
                assert_eq!(*file, 0);
                &text[range.clone()]
            })
            .collect();
        assert_eq!(
            found,
            [
                "#[cfg(test)] pub b: u8",
                "#[cfg(test)] fn helper() {}",
                "#[cfg(test)]\nmod tests { fn t() {} }"
            ]
        );
        assert_eq!(krate.modules.len(), 1);
        assert_eq!(krate.decls[0].members.len(), 1);
        assert!(krate.bodies.is_empty());
    }

    #[test]
    fn serde_paths_get_value_references() {
        let (_tmp, krate) = collected(
            "pub struct Config {\n\
             #[serde(default = \"port\", rename = \"p\")]\n\
             #[cfg_attr(feature = \"x\", serde(with = \"self::codec\"))]\n\
             pub port: u16,\n\
             }\n\
             fn port() -> u16 { 1 }\n\
             mod codec {}\n",
        );
        let found: Vec<(&str, String, u32)> = krate
            .attr_paths
            .iter()
            .map(|p| (p.key.as_str(), p.path.join("::"), p.loc.line))
            .collect();
        assert_eq!(
            found,
            [
                ("default", "port".to_owned(), 2),
                ("with", "self::codec".to_owned(), 3)
            ]
        );
        let port = &krate.refs[krate.attr_paths[0].path_ref];
        assert!(port.comment);
        assert_eq!(port.value().map(|v| v.item), Some(0));
    }
}
