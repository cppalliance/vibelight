//! Text assembled from source slices, with type names split out.

use crate::model::{DeclId, Loc, ModId, RefId};

/// One run of text: plain characters or a type name to resolve.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Seg {
    Plain(String),
    Ref(RefId),
}

/// Text assembled from source slices, with type names split out. The first
/// list is the C++ spelling, with each reference's and pointer's sigil
/// after its type, as in `T mut& 'a`; the second is the Rust spelling, as
/// in `&'a mut T`, kept only when the two differ.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Text(pub(crate) Vec<Seg>, pub(crate) Option<Vec<Seg>>);

impl Text {
    /// Builds text with no type names.
    pub(crate) fn plain(text: impl Into<String>) -> Text {
        let text = text.into();
        if text.is_empty() {
            Text(Vec::new(), None)
        } else {
            Text(vec![Seg::Plain(text)], None)
        }
    }

    /// Appends plain characters, merging with a plain run at the end.
    pub(crate) fn push_str(&mut self, text: &str) {
        push_plain(&mut self.0, text);
        if let Some(rust) = &mut self.1 {
            push_plain(rust, text);
        }
    }

    pub(crate) fn push_ref(&mut self, id: RefId) {
        self.0.push(Seg::Ref(id));
        if let Some(rust) = &mut self.1 {
            rust.push(Seg::Ref(id));
        }
    }

    /// Appends another text, in both spellings.
    pub(crate) fn append(&mut self, other: &Text) {
        if self.1.is_none() && other.1.is_some() {
            self.1 = Some(self.0.clone());
        }
        extend(&mut self.0, &other.0);
        if let Some(rust) = &mut self.1 {
            extend(rust, other.rust());
        }
    }

    /// Returns the segments as Rust spells them.
    pub(crate) fn rust(&self) -> &[Seg] {
        self.1.as_deref().unwrap_or(&self.0)
    }

    /// Gives the text a Rust spelling of its own, such as a re-export's
    /// `path as Alias` for the C++ `Alias = path`.
    pub(crate) fn spelled(mut self, rust: Vec<Seg>) -> Text {
        self.1 = (rust != self.0).then_some(rust);
        self
    }

    /// Returns the ids of the type names in the text, in order.
    pub(crate) fn refs(&self) -> impl Iterator<Item = RefId> + '_ {
        self.0.iter().filter_map(|seg| match seg {
            Seg::Ref(id) => Some(*id),
            Seg::Plain(_) => None,
        })
    }

    /// Returns the text as printed, with type names as written.
    pub(crate) fn visible(&self, refs: &[TypeRef]) -> String {
        let mut out = String::new();
        for seg in &self.0 {
            match seg {
                Seg::Plain(text) => out.push_str(text),
                Seg::Ref(id) => out.push_str(&refs[*id].text),
            }
        }
        out
    }
}

/// Appends plain characters to a list, merging with a plain run at its end.
fn push_plain(segs: &mut Vec<Seg>, text: &str) {
    if text.is_empty() {
        return;
    }
    if let Some(Seg::Plain(last)) = segs.last_mut() {
        last.push_str(text);
    } else {
        segs.push(Seg::Plain(text.to_owned()));
    }
}

fn extend(segs: &mut Vec<Seg>, other: &[Seg]) {
    for seg in other {
        match seg {
            Seg::Plain(text) => push_plain(segs, text),
            Seg::Ref(id) => segs.push(Seg::Ref(*id)),
        }
    }
}

/// A `where` clause: each predicate as written, in order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Where(pub(crate) Vec<Text>);

impl Where {
    /// Returns the clause on one line: `where` and the predicates joined
    /// by `, `.
    pub(crate) fn joined(&self) -> Text {
        let mut text = Text::plain("where ");
        for (i, predicate) in self.0.iter().enumerate() {
            if i > 0 {
                text.push_str(", ");
            }
            text.append(predicate);
        }
        text
    }
}

/// Where a name is written: its line, and its byte offset, which tells two
/// slices of one written name apart from two names on one line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Written {
    pub(crate) loc: Loc,
    pub(crate) offset: usize,
}

/// A module-level function, constant, or static: the module that holds it
/// and its index in that module's items.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ValueRef {
    pub(crate) module: ModId,
    pub(crate) item: usize,
}

/// What a name reaches in another crate of the run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reach {
    Decl(DeclId),
    Value(ValueRef),
}

/// An item in another crate of the run: the crate's index in the run and
/// what the name reaches there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CrossRef {
    pub(crate) krate: usize,
    pub(crate) reach: Reach,
}

/// A type name found in a declaration, a comment, or a re-export.
#[derive(Clone, Debug)]
pub(crate) struct TypeRef {
    /// The name as written, such as `std::result::Result` or `Block`.
    pub(crate) text: String,
    /// The path segments, such as `["crate", "prompt", "Prompt"]`.
    pub(crate) path: Vec<String>,
    /// True when the path starts with `::`.
    pub(crate) global: bool,
    /// The module the name is written in.
    pub(crate) module: ModId,
    /// Generic parameter names in scope where the name is written.
    pub(crate) generics: Vec<String>,
    /// True for a name from a comment, a re-export, a derive, or a function
    /// body, which never counts for naming, toward the unresolved tally, or
    /// for spellings.
    pub(crate) comment: bool,
    /// True for a name that may also be a function, constant, or static:
    /// one in a comment or a re-export, never a derive.
    pub(crate) may_be_value: bool,
    /// The declaration the name resolves to, filled in by resolve.
    pub(crate) target: Option<DeclId>,
    /// The function, constant, or static of this crate the name resolves
    /// to, filled in by resolve when no declaration of that name is found.
    pub(crate) value: Option<ValueRef>,
    /// The full path of a name outside the crate, filled in by resolve:
    /// through `use` items, or the bare name of a prelude item.
    pub(crate) canonical: Option<Vec<String>>,
    /// Where the name is written, for a name sliced from the source.
    pub(crate) written: Option<Written>,
    /// The item in another crate of the run that the name reaches, filled
    /// in after every crate of the run is resolved.
    pub(crate) foreign: Option<CrossRef>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_merges_plain_runs() {
        let mut text = Text::plain("Vec<");
        text.push_ref(0);
        text.push_str(">");
        text.push_str(",");
        assert_eq!(text.0.len(), 3);
        let refs = vec![TypeRef {
            text: "Block".into(),
            path: vec!["Block".into()],
            global: false,
            module: 0,
            generics: Vec::new(),
            comment: false,
            may_be_value: false,
            target: None,
            value: None,
            canonical: None,
            written: None,
            foreign: None,
        }];
        assert_eq!(text.visible(&refs), "Vec<Block>,");
    }
}
