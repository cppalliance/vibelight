//! The text that opens a comment line: the markdown prints it, and the HTML
//! drops it, so a comment reads as prose in the page.

/// The text that opens each line of a comment, in the two forms a line's
/// own text can call for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Marker {
    /// Before a line whose text starts a word.
    pub(crate) spaced: &'static str,
    /// Before a line that already starts with whitespace, and before a
    /// continuation line of a block comment.
    pub(crate) bare: &'static str,
}

impl Marker {
    /// A markdown doc comment, `///`.
    pub(crate) const DOC: Marker = Marker {
        spaced: "/// ",
        bare: "///",
    };

    /// A module's doc comment, `//!`.
    pub(crate) const MODULE: Marker = Marker {
        spaced: "//! ",
        bare: "//!",
    };

    /// An ordinary comment, `//`.
    pub(crate) const PLAIN: Marker = Marker {
        spaced: "// ",
        bare: "//",
    };

    /// The `// cfg:` line that opens a node's configuration.
    pub(crate) const CFG: Marker = Marker {
        spaced: "// cfg: ",
        bare: "// cfg:",
    };

    /// The `// impl in <file>` line that opens an inherent impl's members.
    pub(crate) const IMPL_IN: Marker = Marker {
        spaced: "// impl in ",
        bare: "// impl in",
    };

    /// The `// impl <trait>` line above an impl's comments.
    pub(crate) const IMPL: Marker = Marker {
        spaced: "// impl ",
        bare: "// impl",
    };

    /// The `// internal` or `// private` line above a type's header.
    pub(crate) const VISIBILITY: Marker = Marker {
        spaced: "// ",
        bare: "//",
    };

    /// Returns the text to print: `spaced` when the line's own text starts a
    /// word, `bare` when it already starts with whitespace.
    pub(crate) fn text(self, already_indented: bool) -> &'static str {
        if already_indented {
            self.bare
        } else {
            self.spaced
        }
    }
}
