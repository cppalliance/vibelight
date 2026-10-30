//! The model of function bodies and the other facts collection keeps for
//! analysis: what each body belongs to, and what its code and comments hold.
//! The markdown prints them as counts, names, and comments, never as code,
//! and the HTML prints none of them.

use crate::model::{DeclId, Loc, ModId, RefId, ValueRef};

/// A type name a function body writes. It keeps the type out of the HTML's
/// Unconnected types, the declarations that nothing names, and names nothing
/// else.
#[derive(Clone, Copy, Debug)]
pub(crate) struct BodyName {
    pub(crate) name: RefId,
    pub(crate) owner: BodyOwner,
}

/// What a function body belongs to. A type that owns a body does not count
/// as named by it.
#[derive(Clone, Copy, Debug)]
pub(crate) enum BodyOwner {
    /// A free function.
    Module,
    /// A method of this impl, which every type its header names owns, such
    /// as `Engine` in `impl Clone for Box<Engine>`.
    Impl(usize),
    /// A default method of this trait.
    Trait(DeclId),
}

/// Index into [`Crate::bodies`](crate::model::Crate::bodies).
pub(crate) type BodyId = usize;

/// What kind of code a body is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum BodyKind {
    /// A free function's block.
    Fn,
    /// The block of an inherent impl's method.
    Method,
    /// The block of a trait impl's method.
    TraitImplMethod,
    /// The block of a trait's default method.
    TraitDefault,
    /// A constant's initializer.
    Const,
    /// A static's initializer.
    Static,
    /// An associated constant's value: an impl's, or a trait's default.
    AssocConst,
    /// An enum variant's discriminant.
    Discriminant,
    /// The initializer of an anonymous `const _`, which is no item.
    AnonConst,
}

/// The item a body belongs to. It is finer than [`BodyOwner`]: it names the
/// module item or the member itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Owner {
    /// A function, constant, or static of a module.
    Item(ValueRef),
    /// An anonymous `const _` of this module.
    Anon(ModId),
    /// A member of this impl, by name: a method or an associated constant.
    Impl(usize, String),
    /// A member of this trait, by name: a default method or an associated
    /// constant's default.
    Trait(DeclId, String),
    /// A variant of this enum, by name.
    Variant(DeclId, String),
}

/// One body: where it starts, what it is, what it belongs to, and its facts.
/// Closures, async blocks, and nested items are part of the body that holds
/// them.
#[derive(Clone, Debug)]
pub(crate) struct Body {
    /// The byte offset where the body starts in the file's LF-normalized
    /// text: a block's `{`, or an initializer's first token.
    pub(crate) start: usize,
    /// The line `start` is on.
    pub(crate) loc: Loc,
    pub(crate) kind: BodyKind,
    pub(crate) owner: Owner,
    pub(crate) facts: BodyFacts,
}

/// What a body's code and comments hold. Macro arguments are tokens: the
/// syntax facts leave them out, and the lexical facts read them.
#[derive(Clone, Debug, Default)]
pub(crate) struct BodyFacts {
    /// Each path in expression position that may name a function, in
    /// source order. A local binding or a parameter is not one.
    pub(crate) calls: Vec<CallFact>,
    /// The name of each method call, read from the tokens, sorted, repeats
    /// kept.
    pub(crate) methods: Vec<String>,
    /// The path of each struct literal, such as `Shape::Circle { r }`.
    pub(crate) constructs: Vec<PathFact>,
    /// Each path a pattern names, such as `Shape::Circle(r)`, or a name
    /// that starts with a capital, such as `None`.
    pub(crate) patterns: Vec<PathFact>,
    /// The macros the body invokes, as written, sorted, each once.
    pub(crate) macros: Vec<String>,
    /// The identifiers inside macro arguments, sorted, each once.
    pub(crate) macro_idents: Vec<String>,
    pub(crate) counts: Counts,
    /// The statements directly in the body. An initializer that is not a
    /// block counts as one.
    pub(crate) statements: usize,
    /// The first and last line of each statement at every depth, in source
    /// order, which `BodyCommentFact::stmt` indexes.
    pub(crate) stmt_lines: Vec<(u32, u32)>,
    /// What the body does when it is one statement that calls or reads.
    pub(crate) single: Option<Single>,
    /// The `for`, `while`, and `loop` expressions.
    pub(crate) loops: usize,
    /// The deepest nesting of loops.
    pub(crate) loop_depth: usize,
    /// The direct uses of `self` fields and parameters, sorted, each once.
    pub(crate) chains: Vec<OpChain>,
    /// How often the body names each parameter, `self` first when there is
    /// a receiver, in signature order.
    pub(crate) params: Vec<ParamUse>,
    /// The value of each string literal, sorted, each once.
    pub(crate) strings: Vec<String>,
    /// Each integer or float literal as base 10 digits without its suffix
    /// or sign, sorted, each once.
    pub(crate) numbers: Vec<String>,
    pub(crate) fingerprint: Fingerprint,
    /// The ordinary comments inside the body, in source order.
    pub(crate) comments: Vec<BodyCommentFact>,
}

/// How a call fact's path is used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Role {
    /// The callee of a call, such as `run` in `run(x)`.
    Call,
    /// Read as a value that may leave the body, such as `run` in
    /// `map(run)`, `let f = run;`, or `return run`.
    ValueEscape,
    /// Read in place: a method's receiver, the base of a field or index, an
    /// operand, a condition, or an assigned place, such as `LIMIT` in
    /// `n < LIMIT`.
    OtherReference,
}

/// One path in expression position that may name a function.
#[derive(Clone, Debug)]
pub(crate) struct CallFact {
    /// The segments, without `r#`.
    pub(crate) path: Vec<String>,
    /// The reference for the whole path, which may be a value.
    pub(crate) full_ref: RefId,
    /// The reference for the path without its last segment, which names
    /// the type of an associated function such as `Listed::new`; `None` for
    /// a path of one segment.
    pub(crate) prefix_ref: Option<RefId>,
    /// How many arguments a call passes; 0 when the path is not called.
    pub(crate) args: usize,
    /// Each argument that is a literal, by position, as written.
    pub(crate) literal_args: Vec<(usize, String)>,
    /// Each argument that passes a parameter on unchanged, bare or behind
    /// `&` or `&mut`, by position, with the parameter's name.
    pub(crate) passed_params: Vec<(usize, String)>,
    pub(crate) role: Role,
}

/// A path in a struct literal or a pattern.
#[derive(Clone, Debug)]
pub(crate) struct PathFact {
    /// The segments, without `r#`.
    pub(crate) path: Vec<String>,
    /// The reference for the whole path: a type, or in a pattern also a
    /// constant.
    pub(crate) full_ref: RefId,
    /// The reference for the path without its last segment, which names
    /// the enum of a variant; `None` for a path of one segment.
    pub(crate) prefix_ref: Option<RefId>,
}

/// How often a body writes each error, absence, panic, and `.await` marker
/// outside macro arguments.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Counts {
    /// `?` operators.
    pub(crate) tries: usize,
    /// Paths that end in `Err`, in expression position.
    pub(crate) errs: usize,
    /// Paths that end in `None`, in expression position.
    pub(crate) nones: usize,
    /// `.await` expressions.
    pub(crate) awaits: usize,
    /// Invocations of `panic!`, `todo!`, `unimplemented!`, and
    /// `unreachable!`.
    pub(crate) panics: usize,
}

/// What a body of one statement does, past `return`, `?`, `.await`, `&`,
/// `*`, and parentheses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Single {
    /// Calls a path, such as `inner::run`.
    Call(Vec<String>),
    /// Calls a method, such as `len` in `self.items.len()`.
    Method(String),
    /// Reads a field, such as `name` in `&self.name`.
    Field(String),
}

/// Where an operation chain starts.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Root {
    /// A field reached directly through `self`.
    Field(String),
    /// A parameter of the body's function, other than `self`.
    Param(String),
}

/// One direct use of a `self` field or a parameter, with the methods
/// called on it in one expression chain.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct OpChain {
    pub(crate) root: Root,
    /// The methods the chain calls on the root, in order, until a field
    /// access or an index ends it, such as `iter`, `map`, and `collect`.
    pub(crate) methods: Vec<String>,
    /// The first argument of the first method when it is a literal, such
    /// as `0` in `remove(0)`.
    pub(crate) first_literal: Option<String>,
    /// Whether the chain sits inside a loop's body or condition.
    pub(crate) in_loop: bool,
    /// Whether the chain is assigned to or borrowed as `&mut`.
    pub(crate) write: bool,
}

/// How often a body names one parameter: in its syntax, and as an
/// identifier inside macro arguments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ParamUse {
    pub(crate) name: String,
    pub(crate) uses: usize,
}

/// A body's clone fingerprint. Its canonical tokens drop comments and
/// whitespace, keep keywords and punctuation, and turn other identifiers
/// into `ID`, literals into `LIT`, and each macro invocation into `MACRO`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Fingerprint {
    /// How many canonical tokens the body has.
    pub(crate) tokens: usize,
    /// The hash of each run of five canonical tokens, sorted, each once;
    /// empty for a body of fewer than 20 tokens.
    pub(crate) shingles: Vec<u64>,
}

/// Where a body comment sits relative to the statement it goes with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Placement {
    /// Above the statement.
    Leading,
    /// After the statement: at the end of its last line, or below it with
    /// no statement after the comment.
    Trailing,
    /// Inside the statement, or inside a body with no statement.
    Inner,
}

/// One ordinary comment inside a body.
#[derive(Clone, Debug)]
pub(crate) struct BodyCommentFact {
    /// The comment as written, one entry per line, normalized as notes are.
    pub(crate) text: Vec<String>,
    /// The line the comment starts on.
    pub(crate) line: u32,
    pub(crate) placement: Placement,
    /// The index in `BodyFacts::stmt_lines` of the statement it goes with.
    pub(crate) stmt: Option<usize>,
    /// The lowercase identifiers and operators of that statement, and of
    /// the next two for a leading comment, sorted, each once.
    pub(crate) fingerprint: Vec<String>,
}

/// A path a `serde` attribute names in a string, such as `default =
/// "port"`, which the derived code calls.
#[derive(Clone, Debug)]
pub(crate) struct AttrPath {
    /// The attribute's key, such as `default` or `with`.
    pub(crate) key: String,
    /// The segments, without `r#`.
    pub(crate) path: Vec<String>,
    /// The reference for the path, which may be a value.
    pub(crate) path_ref: RefId,
    pub(crate) loc: Loc,
}
