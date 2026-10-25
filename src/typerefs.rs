//! Finds type paths inside syn nodes and builds text with type references.

use std::ops::Range;

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Path, PointerMutability, TraitBound, Type, TypePath, TypePtr, TypeReference};

use crate::model::{ModId, RefId, Text, TypeRef};
use crate::source::{SourceFile, normalize, scan};
use crate::typerefs_edits::{Piece, marked, needs_parens, pieces};

/// Returns the byte range a syn node covers in its file.
pub(crate) fn range_of<T: Spanned + ?Sized>(node: &T, file: &SourceFile) -> Range<usize> {
    let span = node.span();
    let start = file.start(span);
    let end = file.end(span).max(start);
    start..end
}

/// A type path found inside a node.
#[derive(Debug, Clone)]
pub(crate) struct Found {
    /// From the path's start to the end of its last segment's name.
    pub(crate) range: Range<usize>,
    /// The segment names, with any `r#` prefix removed.
    pub(crate) segments: Vec<String>,
    pub(crate) global: bool,
}

/// Collects the type paths and trait-bound paths a node holds, and the edits
/// that move each reference's and raw pointer's sigil after its inner type.
pub(crate) struct Finder<'a> {
    file: &'a SourceFile,
    pub(crate) found: Vec<Found>,
    /// Source to drop, such as the `&'a mut ` before an inner type.
    cuts: Vec<Range<usize>>,
    /// Text to write where an inner type ends, recorded innermost first so
    /// that suffixes at one position nest.
    inserts: Vec<(usize, String)>,
}

impl<'a> Finder<'a> {
    pub(crate) fn new(file: &'a SourceFile) -> Finder<'a> {
        Finder {
            file,
            found: Vec::new(),
            cuts: Vec::new(),
            inserts: Vec::new(),
        }
    }

    /// Visits `elem`, then records the edits that drop the sigil starting at
    /// `sigil` and write `suffix` after `elem`, with `elem` in parentheses
    /// when `needs_parens` says so.
    fn move_sigil(&mut self, sigil: Span, elem: &Type, suffix: String) {
        let start = self.file.start(sigil);
        let inner = range_of(elem, self.file);
        self.visit_type(elem);
        if start < inner.start {
            self.cuts.push(start..inner.start);
            if needs_parens(elem) {
                self.inserts.push((inner.start, "(".to_owned()));
                self.inserts.push((inner.end, format!("){suffix}")));
            } else {
                self.inserts.push((inner.end, suffix));
            }
        }
    }

    /// Records a path that is not inside a type, such as an impl's trait,
    /// along with the types in its generic arguments.
    pub(crate) fn bare_path(&mut self, path: &Path) {
        self.record(path);
        for segment in &path.segments {
            self.visit_path_arguments(&segment.arguments);
        }
    }

    fn record(&mut self, path: &Path) {
        let Some(last) = path.segments.last() else {
            return;
        };
        let start = match &path.leading_colon {
            Some(colons) => self.file.start(colons.spans[0]),
            None => self.file.start(path.segments[0].ident.span()),
        };
        let end = self.file.end(last.ident.span());
        if start >= end {
            return;
        }
        self.found.push(Found {
            range: start..end,
            segments: path
                .segments
                .iter()
                .map(|s| strip_raw(&s.ident.to_string()))
                .collect(),
            global: path.leading_colon.is_some(),
        });
    }
}

impl<'ast> Visit<'ast> for Finder<'_> {
    fn visit_type_path(&mut self, node: &'ast TypePath) {
        match &node.qself {
            None => self.record(&node.path),
            Some(qself) => self.visit_type(&qself.ty),
        }
        for segment in &node.path.segments {
            self.visit_path_arguments(&segment.arguments);
        }
    }

    fn visit_trait_bound(&mut self, node: &'ast TraitBound) {
        self.record(&node.path);
        if let Some(lifetimes) = &node.lifetimes {
            self.visit_bound_lifetimes(lifetimes);
        }
        for segment in &node.path.segments {
            self.visit_path_arguments(&segment.arguments);
        }
    }

    fn visit_type_reference(&mut self, node: &'ast TypeReference) {
        let sigil = marked('&', node.mutability.is_some());
        let suffix = match &node.lifetime {
            Some(lifetime) => format!("{sigil} {lifetime}"),
            None => sigil,
        };
        self.move_sigil(node.and_token.spans[0], &node.elem, suffix);
    }

    fn visit_type_ptr(&mut self, node: &'ast TypePtr) {
        let mutable = matches!(node.mutability, PointerMutability::Mut(_));
        self.move_sigil(node.star_token.spans[0], &node.elem, marked('*', mutable));
    }

    fn visit_macro(&mut self, _: &'ast syn::Macro) {}

    fn visit_expr(&mut self, node: &'ast syn::Expr) {
        if let syn::Expr::Block(_) = node {
            return;
        }
        visit::visit_expr(self, node);
    }
}

/// Removes a raw identifier's `r#` prefix.
pub(crate) fn strip_raw(name: &str) -> String {
    name.strip_prefix("r#").unwrap_or(name).to_owned()
}

/// The first private-use character, which marks a type reference while a
/// slice is normalized.
const MARK: u32 = 0xE000;
const MARKS: u32 = 0x1900;

/// Builds text from source slices, recording type references as it goes.
pub(crate) struct Slicer<'a> {
    pub(crate) file: &'a SourceFile,
    pub(crate) refs: &'a mut Vec<TypeRef>,
    pub(crate) module: ModId,
    pub(crate) generics: &'a [String],
}

impl Slicer<'_> {
    /// Returns a node's text with the type paths `visit` finds as references.
    pub(crate) fn node<T: Spanned + ?Sized>(
        &mut self,
        node: &T,
        visit: impl FnOnce(&mut Finder<'_>),
    ) -> Text {
        let range = range_of(node, self.file);
        self.range(range, visit).0
    }

    /// Returns the text of a byte range with the type paths `visit` finds as
    /// references, and each reference's id with its range.
    pub(crate) fn range(
        &mut self,
        range: Range<usize>,
        visit: impl FnOnce(&mut Finder<'_>),
    ) -> (Text, Vec<(Range<usize>, RefId)>) {
        let mut finder = Finder::new(self.file);
        visit(&mut finder);
        let Finder {
            mut found,
            cuts,
            inserts,
            ..
        } = finder;
        found.sort_by(|a, b| {
            a.range
                .start
                .cmp(&b.range.start)
                .then(b.range.end.cmp(&a.range.end))
        });
        let mut kept: Vec<Found> = Vec::new();
        let mut last_end = range.start;
        for item in found {
            let inside = item.range.start >= last_end && item.range.end <= range.end;
            if inside && kept.len() < MARKS as usize {
                last_end = item.range.end;
                kept.push(item);
            }
        }
        let text = &self.file.text;
        let slice = &text[range.clone()];
        if slice
            .chars()
            .any(|c| (MARK..MARK + MARKS).contains(&(c as u32)))
        {
            kept.clear();
        }

        let mut raw = String::with_capacity(range.len());
        let mut at = range.start;
        let comments = scan(text, range.clone());
        let mut comment = comments.iter().peekable();
        for (span, piece) in pieces(&kept, &cuts, &inserts, &range) {
            push_without_comments(&mut raw, text, at..span.start, &mut comment);
            match piece {
                Piece::Mark(index) => {
                    let code = MARK + u32::try_from(index).unwrap_or(0);
                    raw.push(char::from_u32(code).unwrap_or(' '));
                }
                Piece::Cut => {}
                Piece::Insert(suffix) => raw.push_str(suffix),
            }
            at = span.end;
            while comment.peek().is_some_and(|c| c.range.start < at) {
                comment.next();
            }
        }
        push_without_comments(&mut raw, text, at..range.end, &mut comment);

        let mut out = Text::default();
        let mut ids = Vec::new();
        for c in normalize(&raw).chars() {
            let code = c as u32;
            if (MARK..MARK + MARKS).contains(&code) {
                let item = &kept[(code - MARK) as usize];
                let id = self.refs.len();
                self.refs.push(TypeRef {
                    text: self.file.clean(item.range.clone()),
                    path: item.segments.clone(),
                    global: item.global,
                    module: self.module,
                    generics: self.generics.to_vec(),
                    comment: false,
                    target: None,
                    canonical: None,
                });
                out.push_ref(id);
                ids.push((item.range.clone(), id));
            } else {
                let mut buffer = [0u8; 4];
                out.push_str(c.encode_utf8(&mut buffer));
            }
        }
        (out, ids)
    }
}

/// Appends `text[range]`, replacing each comment in it with one space.
fn push_without_comments(
    out: &mut String,
    text: &str,
    range: Range<usize>,
    comments: &mut std::iter::Peekable<std::slice::Iter<'_, crate::source::Comment>>,
) {
    let mut at = range.start;
    while let Some(comment) = comments.peek() {
        if comment.range.start >= range.end {
            break;
        }
        if comment.range.start >= at {
            out.push_str(&text[at..comment.range.start]);
            out.push(' ');
            at = comment.range.end.min(range.end);
        }
        comments.next();
    }
    if at < range.end {
        out.push_str(&text[at..range.end]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slice_type(source: &str) -> (String, Vec<String>) {
        let file = SourceFile::new("a.rs".into(), source);
        let item: syn::ItemType = syn::parse_str(&file.text).unwrap();
        let mut refs = Vec::new();
        let mut slicer = Slicer {
            file: &file,
            refs: &mut refs,
            module: 0,
            generics: &[],
        };
        let text = slicer.node(&*item.ty, |f| f.visit_type(&item.ty));
        let names = refs.iter().map(|r| r.path.join("::")).collect();
        (text.visible(&refs), names)
    }

    #[test]
    fn nested_paths_become_references() {
        let (text, refs) = slice_type("type A = Vec< Block >;");
        assert_eq!(text, "Vec<Block>");
        assert_eq!(refs, ["Vec", "Block"]);
    }

    #[test]
    fn spacing_survives_around_references() {
        let (text, refs) =
            slice_type("type A = &'a   std::result::Result<\n    Prompt,\n    ParseError,\n>;");
        assert_eq!(text, "std::result::Result<Prompt, ParseError>& 'a");
        assert_eq!(refs, ["std::result::Result", "Prompt", "ParseError"]);
    }

    #[test]
    fn bounds_and_comments_inside_types() {
        let (text, refs) = slice_type("type A = Box<dyn Fn(u32) -> Out /* note */ + Send>;");
        assert_eq!(text, "Box<dyn Fn(u32) -> Out + Send>");
        assert_eq!(refs, ["Box", "Fn", "u32", "Out", "Send"]);
    }

    #[test]
    fn qualified_paths_keep_their_inner_types() {
        let (text, refs) = slice_type("type A = <T as Trait>::Item;");
        assert_eq!(text, "<T as Trait>::Item");
        assert_eq!(refs, ["T"]);
    }
}
