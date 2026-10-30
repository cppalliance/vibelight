//! What a compared type is made of: its head and generic arguments, the
//! `Option`, `Result`, tuple, slice, or primitive it is, and the type a
//! role holds at heart once references, options, and wrappers are peeled.

use crate::leads_types::{Tok, Ty, leaf};

/// The standard collections and the common map and set crates' ones, by
/// the last segment of their path.
const COLLECTIONS: [&str; 10] = [
    "BTreeMap",
    "BTreeSet",
    "BinaryHeap",
    "HashMap",
    "HashSet",
    "IndexMap",
    "IndexSet",
    "LinkedList",
    "Vec",
    "VecDeque",
];

/// The pointers, cells, locks, and wrappers that [`Ty::core`] peels, by
/// the last segment of their path; a `Result` gives its value.
const PEELED: [&str; 15] = [
    "Arc",
    "Box",
    "Cell",
    "Cow",
    "LazyLock",
    "ManuallyDrop",
    "Mutex",
    "OnceCell",
    "OnceLock",
    "Pin",
    "Rc",
    "RefCell",
    "Result",
    "RwLock",
    "Weak",
];

impl Ty {
    /// Returns the type without the `&`, `'static`, and `mut` before it.
    pub(crate) fn deref(&self) -> Ty {
        let mut toks = self.0.as_slice();
        while let Some((first, rest)) = toks.split_first() {
            if !matches!(first.code(), Some("&" | "mut" | "'static")) {
                break;
            }
            toks = rest;
        }
        Ty(toks.to_vec())
    }

    /// Returns the head name and the generic arguments of a type written as
    /// one name and one argument list, such as `Option<u8>`.
    pub(crate) fn generic(&self) -> Option<(&str, Vec<Ty>)> {
        let [
            Tok::Name(head),
            Tok::Code(open),
            inner @ ..,
            Tok::Code(close),
        ] = self.0.as_slice()
        else {
            return None;
        };
        (open == "<" && close == ">" && balanced(inner)).then(|| (head.as_str(), split(inner)))
    }

    /// Returns what an `Option` holds.
    pub(crate) fn option(&self) -> Option<Ty> {
        match self.generic()? {
            ("std::option::Option", mut args) if args.len() == 1 => args.pop(),
            _ => None,
        }
    }

    /// Tells whether the type is a `Result`: the standard one, or an alias
    /// named `Result`.
    pub(crate) fn is_result(&self) -> bool {
        self.generic()
            .is_some_and(|(head, _)| leaf(head) == "Result")
    }

    /// Returns the parts of a tuple of two or more.
    pub(crate) fn tuple(&self) -> Option<Vec<Ty>> {
        let [Tok::Code(open), inner @ .., Tok::Code(close)] = self.0.as_slice() else {
            return None;
        };
        if open != "(" || close != ")" || !balanced(inner) {
            return None;
        }
        let parts = split(inner);
        (parts.len() > 1).then_some(parts)
    }

    pub(crate) fn is_unit(&self) -> bool {
        matches!(self.0.as_slice(), [Tok::Code(o), Tok::Code(c)] if o == "(" && c == ")")
    }

    /// Returns the primitive the type is, such as `u32` or `str`.
    pub(crate) fn primitive(&self) -> Option<&str> {
        match self.0.as_slice() {
            [Tok::Name(name)] => name.strip_prefix("std::primitive::"),
            _ => None,
        }
    }

    pub(crate) fn is_bool(&self) -> bool {
        self.primitive() == Some("bool")
    }

    /// Tells whether the type is `String` or `str`.
    pub(crate) fn is_string(&self) -> bool {
        matches!(self.0.as_slice(), [Tok::Name(n)]
            if n == "std::string::String" || n == "std::primitive::str")
    }

    /// Tells whether the type is a collection or a slice.
    pub(crate) fn is_collection(&self) -> bool {
        if let Some((head, _)) = self.generic() {
            return COLLECTIONS.contains(&leaf(head).as_str());
        }
        self.slice().is_some()
    }

    /// Returns what a slice `[T]` holds.
    fn slice(&self) -> Option<Ty> {
        let [Tok::Code(open), inner @ .., Tok::Code(close)] = self.0.as_slice() else {
            return None;
        };
        let array = inner.iter().any(|t| t.code() == Some(";"));
        (open == "[" && close == "]" && !array && balanced(inner)).then(|| Ty(inner.to_vec()))
    }

    /// Tells whether a name in the type is a generic parameter or `Self`.
    pub(crate) fn is_generic(&self) -> bool {
        self.names().any(|n| n.starts_with('$') || n == "Self")
    }

    /// Returns the names in the type, in order.
    pub(crate) fn names(&self) -> impl Iterator<Item = &str> {
        self.0.iter().filter_map(|tok| match tok {
            Tok::Name(name) => Some(name.as_str()),
            Tok::Code(_) => None,
        })
    }

    /// Returns the type a name or a role stands for: without references or
    /// an `Option` around it, `str` as `String`, `Path` as `PathBuf`,
    /// `OsStr` as `OsString`, and a slice as a `Vec`.
    pub(crate) fn base(&self) -> Ty {
        let mut ty = self.deref();
        if let Some(inner) = ty.option() {
            ty = inner.deref();
        }
        if let Some(inner) = ty.slice() {
            let mut toks = vec![Tok::Name("std::vec::Vec".to_owned()), punct("<")];
            toks.extend(inner.0);
            toks.push(punct(">"));
            return Ty(toks);
        }
        let owned = match ty.0.as_slice() {
            [Tok::Name(name)] => match name.as_str() {
                "std::primitive::str" => "std::string::String",
                "std::path::Path" => "std::path::PathBuf",
                "std::ffi::OsStr" => "std::ffi::OsString",
                _ => return ty,
            },
            _ => return ty,
        };
        Ty::name(owned)
    }

    /// Returns the type a role holds at heart: [`Ty::base`] after every
    /// [`PEELED`] wrapper and every `impl Into`, `impl AsRef`, or `impl
    /// Borrow` around it is peeled, with an atomic as its primitive.
    pub(crate) fn core(&self) -> Ty {
        let mut ty = self.base();
        loop {
            let peeled = match ty.generic() {
                Some((head, args)) if PEELED.contains(&leaf(head).as_str()) => {
                    let one = args.len() == 1 || leaf(head) == "Result";
                    args.into_iter().next().filter(|_| one)
                }
                _ => None,
            };
            match peeled.or_else(|| ty.conversion()).or_else(|| ty.atomic()) {
                Some(inner) => ty = inner.base(),
                None => return ty,
            }
        }
    }

    /// Returns what `impl Into<T>`, `impl AsRef<T>`, or `impl Borrow<T>`
    /// converts to.
    fn conversion(&self) -> Option<Ty> {
        let [
            Tok::Code(word),
            Tok::Name(head),
            Tok::Code(open),
            inner @ ..,
            Tok::Code(close),
        ] = self.0.as_slice()
        else {
            return None;
        };
        let converts = matches!(leaf(head).as_str(), "AsRef" | "Borrow" | "Into");
        let one = word == "impl" && converts && open == "<" && close == ">";
        (one && balanced(inner)).then(|| Ty(inner.to_vec()))
    }

    /// Returns the primitive a standard atomic holds, such as `u64` for
    /// `AtomicU64`.
    fn atomic(&self) -> Option<Ty> {
        let [Tok::Name(name)] = self.0.as_slice() else {
            return None;
        };
        let rest = name.strip_prefix("std::sync::atomic::Atomic")?;
        Some(Ty::name(&format!(
            "std::primitive::{}",
            rest.to_lowercase()
        )))
    }

    /// Tells whether the type is `impl` or `dyn` some trait.
    pub(crate) fn is_opaque(&self) -> bool {
        (self.0.iter()).any(|tok| matches!(tok.code(), Some("dyn" | "impl")))
    }

    /// Tells whether every name in the type, and there is one, is `std`'s.
    pub(crate) fn is_std(&self) -> bool {
        self.names().next().is_some() && self.names().all(|name| name.starts_with("std::"))
    }
}

fn punct(text: &str) -> Tok {
    Tok::Code(text.to_owned())
}

/// Tells whether brackets of every kind open and close in order.
fn balanced(toks: &[Tok]) -> bool {
    let mut depth = 0usize;
    for tok in toks {
        match tok.code() {
            Some("<" | "(" | "[") => depth += 1,
            Some(">" | ")" | "]") => match depth.checked_sub(1) {
                Some(less) => depth = less,
                None => return false,
            },
            _ => {}
        }
    }
    depth == 0
}

/// Splits tokens at their top-level commas.
fn split(toks: &[Tok]) -> Vec<Ty> {
    let mut parts = vec![Vec::new()];
    let mut depth = 0usize;
    for tok in toks {
        match tok.code() {
            Some("<" | "(" | "[") => depth += 1,
            Some(">" | ")" | "]") => depth = depth.saturating_sub(1),
            Some(",") if depth == 0 => {
                parts.push(Vec::new());
                continue;
            }
            _ => {}
        }
        if let Some(part) = parts.last_mut() {
            part.push(tok.clone());
        }
    }
    parts
        .into_iter()
        .filter(|p| !p.is_empty())
        .map(Ty)
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::leads_types::tests::ty;

    #[test]
    fn a_type_splits_into_its_parts() {
        let option = ty(&["std::option::Option", "<&", "std::primitive::bool", ">"]);
        assert_eq!(option.option().map(|t| t.deref().is_bool()), Some(true));
        let tuple = ty(&[
            "(",
            "std::primitive::u8",
            ", ",
            "std::vec::Vec",
            "<(",
            "$T",
            ", ",
            "$U",
            ")>)",
        ]);
        let parts = tuple.tuple().unwrap();
        assert_eq!(parts.len(), 2);
        assert!(parts[1].is_collection() && parts[1].is_generic());
        assert!(ty(&["(", ")"]).is_unit());
        assert!(ty(&["[", "std::primitive::u8", "]"]).is_collection());
        assert!(!ty(&["[", "std::primitive::u8", "; 4]"]).is_collection());
        let slice = ty(&["&[", "std::primitive::str", "]"]);
        assert_eq!(slice.base().shown(), "Vec<str>");
        assert_eq!(ty(&["&", "std::path::Path"]).base().shown(), "PathBuf");
    }

    #[test]
    fn a_core_type_drops_its_wrappers() {
        let held = ty(&[
            "std::sync::Arc",
            "<",
            "std::sync::Mutex",
            "<",
            "#0::Inner",
            ">>",
        ]);
        assert_eq!(held.core().shown(), "Inner");
        let into = ty(&[
            "impl ",
            "std::convert::Into",
            "<",
            "std::string::String",
            ">",
        ]);
        assert_eq!(into.core(), ty(&["std::string::String"]));
        let shared = ty(&["std::sync::Arc", "<", "std::primitive::str", ">"]);
        assert_eq!(shared.core(), ty(&["std::string::String"]));
        assert_eq!(ty(&["std::sync::atomic::AtomicU64"]).core().shown(), "u64");
        let result = ty(&[
            "std::result::Result",
            "<",
            "#0::Form",
            ", ",
            "#0::Rejection",
            ">",
        ]);
        assert_eq!(result.core().shown(), "Form");
        assert!(ty(&["dyn ", "#0::Run"]).is_opaque());
        assert!(ty(&["(", "std::string::String", ", ", "std::path::PathBuf", ")"]).is_std());
        assert!(!ty(&["std::vec::Vec", "<", "#0::Form", ">"]).is_std());
    }
}
