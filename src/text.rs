//! Text assembled from source slices, with type names split out.

use crate::model::{DeclId, ModId, RefId};

/// One run of text: plain characters or a type name to resolve.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Seg {
    Plain(String),
    Ref(RefId),
}

/// Text assembled from source slices, with type names split out.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Text(pub(crate) Vec<Seg>);

impl Text {
    /// Builds text with no type names.
    pub(crate) fn plain(text: impl Into<String>) -> Text {
        let text = text.into();
        if text.is_empty() {
            Text(Vec::new())
        } else {
            Text(vec![Seg::Plain(text)])
        }
    }

    /// Appends plain characters, merging with a plain run at the end.
    pub(crate) fn push_str(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        if let Some(Seg::Plain(last)) = self.0.last_mut() {
            last.push_str(text);
        } else {
            self.0.push(Seg::Plain(text.to_owned()));
        }
    }

    pub(crate) fn push_ref(&mut self, id: RefId) {
        self.0.push(Seg::Ref(id));
    }

    /// Appends another text.
    pub(crate) fn append(&mut self, other: &Text) {
        for seg in &other.0 {
            match seg {
                Seg::Plain(text) => self.push_str(text),
                Seg::Ref(id) => self.push_ref(*id),
            }
        }
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
    /// True for a name from a comment, a re-export, or a derive, which never
    /// counts for naming or toward the unresolved tally.
    pub(crate) comment: bool,
    /// The declaration the name resolves to, filled in by resolve.
    pub(crate) target: Option<DeclId>,
    /// The full path of a name outside the crate, filled in by resolve:
    /// through `use` items, or the bare name of a prelude item.
    pub(crate) canonical: Option<Vec<String>>,
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
            target: None,
            canonical: None,
        }];
        assert_eq!(text.visible(&refs), "Vec<Block>,");
    }
}
