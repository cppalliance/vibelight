//! Records each body collection reads for analysis: what kind of code it
//! is, what it belongs to, and the facts its syntax, tokens, and comments
//! hold. Each path a body calls, reads, constructs, or matches gets comment
//! references that resolve fills in and no outline counts.

use syn::{FnArg, Pat, Signature};

use crate::collect::{Collector, Here};
use crate::collect_body_code::{Called, Code};
use crate::collect_body_comments::{comment_facts, last_line};
use crate::collect_body_read::read;
use crate::collect_body_tokens::scan;
use crate::model::{Loc, RefId, TypeRef};
use crate::model_body::{Body, BodyFacts, BodyKind, CallFact, Owner, ParamUse, PathFact};
use crate::typerefs::strip_raw;

impl Collector<'_> {
    /// Records one body and its facts. Every comment still unused inside
    /// the body goes to its facts, so none of them becomes a note. `sig` is
    /// a function body's signature.
    pub(crate) fn body(
        &mut self,
        here: &Here,
        kind: BodyKind,
        owner: Owner,
        code: Code<'_>,
        sig: Option<&Signature>,
    ) {
        let files = self.files;
        let file = &files[here.file];
        let range = code.range(file);
        let comments = self.pools[here.file].take(range.clone());
        let names = sig.map(param_names).unwrap_or_default();
        let read = read(file, code, &names);
        let scan = scan(&file.text[range.clone()], &names);
        let calls = read
            .called
            .into_iter()
            .map(|called| self.call_fact(here, called))
            .collect();
        let constructs = read
            .constructs
            .into_iter()
            .map(|(global, path)| self.path_fact(here, global, path, false))
            .collect();
        let patterns = read
            .patterns
            .into_iter()
            .map(|(global, path)| self.path_fact(here, global, path, true))
            .collect();
        let params = names
            .into_iter()
            .zip(read.param_uses.iter().zip(&scan.param_uses))
            .map(|(name, (syntax, tokens))| ParamUse {
                name,
                uses: syntax + tokens,
            })
            .collect();
        let facts = BodyFacts {
            calls,
            methods: scan.methods,
            constructs,
            patterns,
            macros: read.macros.into_iter().collect(),
            macro_idents: scan.macro_idents,
            counts: read.counts,
            statements: code.statements(),
            stmt_lines: read
                .statements
                .iter()
                .map(|s| (file.line_of(s.range.start), last_line(file, s)))
                .collect(),
            single: code.single(),
            loops: read.loops,
            loop_depth: read.loop_depth,
            chains: read.chains.into_iter().collect(),
            params,
            strings: scan.strings,
            numbers: scan.numbers,
            fingerprint: scan.fingerprint,
            comments: comment_facts(file, &read.statements, &comments),
        };
        self.bodies.push(Body {
            start: range.start,
            loc: Loc {
                file: here.file,
                line: file.line_of(range.start),
            },
            kind,
            owner,
            facts,
        });
    }

    /// Pushes a comment reference for a path kept for analysis, which
    /// resolve fills in and no outline counts or shows.
    pub(crate) fn analysis_ref(
        &mut self,
        here: &Here,
        global: bool,
        path: Vec<String>,
        may_be_value: bool,
    ) -> RefId {
        self.refs.push(TypeRef {
            text: path.join("::"),
            path,
            global,
            module: here.module,
            generics: here.generics.clone(),
            comment: true,
            may_be_value,
            res: None,
            written: None,
            foreign: None,
        });
        self.refs.len() - 1
    }

    /// Pushes a reference for a path without its last segment, which names
    /// the type of an associated item or the enum of a variant.
    fn prefix_ref(&mut self, here: &Here, global: bool, path: &[String]) -> Option<RefId> {
        let (_, prefix) = path.split_last()?;
        (!prefix.is_empty()).then(|| self.analysis_ref(here, global, prefix.to_vec(), false))
    }

    fn call_fact(&mut self, here: &Here, called: Called) -> CallFact {
        let full_ref = self.analysis_ref(here, called.global, called.path.clone(), true);
        let prefix_ref = self.prefix_ref(here, called.global, &called.path);
        CallFact {
            path: called.path,
            full_ref,
            prefix_ref,
            args: called.args,
            literal_args: called.literal_args,
            passed_params: called.passed_params,
            role: called.role,
        }
    }

    fn path_fact(
        &mut self,
        here: &Here,
        global: bool,
        path: Vec<String>,
        may_be_value: bool,
    ) -> PathFact {
        let full_ref = self.analysis_ref(here, global, path.clone(), may_be_value);
        let prefix_ref = self.prefix_ref(here, global, &path);
        PathFact {
            path,
            full_ref,
            prefix_ref,
        }
    }
}

/// Returns the names a signature binds, in order, each once: `self` for a
/// receiver, then each name its parameter patterns bind.
fn param_names(sig: &Signature) -> Vec<String> {
    let mut names = Vec::new();
    for arg in &sig.inputs {
        match arg {
            FnArg::Receiver(_) => bind("self".to_owned(), &mut names),
            FnArg::Typed(typed) => bound(&typed.pat, &mut names),
        }
    }
    names
}

fn bind(name: String, names: &mut Vec<String>) {
    if !names.contains(&name) {
        names.push(name);
    }
}

/// Adds the names a parameter pattern binds.
fn bound(pat: &Pat, names: &mut Vec<String>) {
    match pat {
        Pat::Ident(ident) => {
            bind(strip_raw(&ident.ident.to_string()), names);
            if let Some((_, sub)) = &ident.subpat {
                bound(sub, names);
            }
        }
        Pat::Tuple(tuple) => tuple.elems.iter().for_each(|p| bound(p, names)),
        Pat::TupleStruct(tuple) => tuple.elems.iter().for_each(|p| bound(p, names)),
        Pat::Slice(slice) => slice.elems.iter().for_each(|p| bound(p, names)),
        Pat::Or(or) => or.cases.iter().for_each(|p| bound(p, names)),
        Pat::Struct(fields) => fields.fields.iter().for_each(|f| bound(&f.pat, names)),
        Pat::Reference(reference) => bound(&reference.pat, names),
        Pat::Paren(paren) => bound(&paren.pat, names),
        Pat::Type(typed) => bound(&typed.pat, names),
        _ => {}
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::fs;

    use crate::model::{Crate, MemberKind, ValueRef};
    use crate::model_body::{BodyKind, Counts, OpChain, Owner, Role, Root, Single};

    /// Collects and resolves a crate made of one `lib.rs`.
    pub(crate) fn collected(lib: &str) -> (tempfile::TempDir, Crate) {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("lib.rs"), lib).unwrap();
        let tree = crate::modtree::walk(tmp.path(), "src/lib.rs").unwrap();
        let mut krate = crate::collect::collect(tree, tmp.path(), "t", "t");
        crate::resolve::resolve(&mut krate);
        (tmp, krate)
    }

    /// The facts of the body that belongs to `owner`.
    fn facts<'k>(krate: &'k Crate, owner: &Owner) -> &'k crate::model_body::BodyFacts {
        &krate
            .bodies
            .iter()
            .find(|b| b.owner == *owner)
            .expect("the body exists")
            .facts
    }

    fn item(module: usize, item: usize) -> Owner {
        Owner::Item(ValueRef { module, item })
    }

    #[test]
    fn every_body_kind_gets_one_body_with_its_owner() {
        let (_tmp, krate) = collected(
            "pub fn free() -> u8 { 1 }\n\
             pub const LIMIT: u8 = 2 + 1;\n\
             pub static TOTAL: u8 = 3;\n\
             const _: () = { let _ = 4; };\n\
             pub enum Level { Low = 1, High = 2 }\n\
             pub struct Gauge;\n\
             impl Gauge { pub const MAX: u8 = 9; pub fn read(&self) -> u8 { Self::MAX } }\n\
             impl Default for Gauge { fn default() -> Self { Gauge } }\n\
             pub trait Probe { const DEPTH: u8 = 5; fn probe(&self) -> u8 { Self::DEPTH } fn bare(&self); }\n\
             impl Probe for Gauge { const DEPTH: u8 = 6; fn bare(&self) { let _ = || self.probe(); } }\n",
        );
        let s = str::to_owned;
        let found: Vec<(BodyKind, Owner)> = krate
            .bodies
            .iter()
            .map(|b| (b.kind, b.owner.clone()))
            .collect();
        assert_eq!(
            found,
            [
                (BodyKind::Fn, item(0, 0)),
                (BodyKind::Const, item(0, 1)),
                (BodyKind::Static, item(0, 2)),
                (BodyKind::AnonConst, Owner::Anon(0)),
                (BodyKind::Discriminant, Owner::Variant(0, s("Low"))),
                (BodyKind::Discriminant, Owner::Variant(0, s("High"))),
                (BodyKind::AssocConst, Owner::Impl(0, s("MAX"))),
                (BodyKind::Method, Owner::Impl(0, s("read"))),
                (BodyKind::TraitImplMethod, Owner::Impl(1, s("default"))),
                (BodyKind::AssocConst, Owner::Trait(2, s("DEPTH"))),
                (BodyKind::TraitDefault, Owner::Trait(2, s("probe"))),
                (BodyKind::AssocConst, Owner::Impl(2, s("DEPTH"))),
                (BodyKind::TraitImplMethod, Owner::Impl(2, s("bare"))),
            ]
        );
        let mut keys: Vec<_> = krate
            .bodies
            .iter()
            .map(|b| (b.loc.file, b.start, b.kind))
            .collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), krate.bodies.len(), "body keys are unique");
        let bare = facts(&krate, &Owner::Impl(2, s("bare")));
        assert_eq!(bare.methods, ["probe"], "the closure is part of its body");
        assert_eq!(
            krate.bodies[3].facts.numbers,
            ["4"],
            "const _ has facts but no item"
        );
    }

    #[test]
    fn trait_impl_members_are_kept_apart_with_comment_references() {
        let (_tmp, krate) = collected(
            "pub struct Gauge;\n\
             impl Gauge { pub fn new() -> Gauge { Gauge } }\n\
             impl std::fmt::Display for Gauge {\n\
             /// Shows it.\n\
             fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, \"g\") }\n\
             }\n\
             impl Iterator for Gauge {\n\
             type Item = u8;\n\
             // the next one\n\
             fn next(&mut self) -> Option<u8> { None }\n\
             }\n",
        );
        let names = |members: &[crate::model::Member]| -> Vec<String> {
            members.iter().map(|m| m.name.clone()).collect()
        };
        assert_eq!(names(&krate.impls[0].members), ["new"]);
        assert!(krate.impls[0].trait_members.is_empty());
        assert!(krate.impls[1].members.is_empty());
        assert_eq!(names(&krate.impls[1].trait_members), ["fmt"]);
        assert_eq!(names(&krate.impls[2].trait_members), ["Item", "next"]);
        let fmt = &krate.impls[1].trait_members[0];
        assert_eq!(fmt.meta.raw_doc.as_deref(), Some("Shows it."));
        assert!(
            fmt.meta.notes.is_empty(),
            "no doc note, so no doc reference"
        );
        let next = &krate.impls[2].trait_members[1];
        assert_eq!(next.meta.notes.len(), 1, "the plain comment is kept");
        let MemberKind::Fn(sig) = &fmt.kind else {
            panic!("fmt is a function");
        };
        let refs: Vec<usize> = sig
            .params
            .iter()
            .flat_map(|p| p.text.refs())
            .chain(sig.ret.iter().flat_map(crate::model::Text::refs))
            .collect();
        assert!(!refs.is_empty(), "the signature names types");
        assert!(refs.iter().all(|&r| krate.refs[r].comment));
    }

    #[test]
    fn calls_keep_paths_roles_arguments_and_resolve() {
        let (_tmp, krate) = collected(
            "pub mod tools { pub fn run(_n: u8, _s: &str) -> u8 { 1 } pub fn helper() -> u8 { 0 } }\n\
             pub struct Table;\n\
             impl Table { pub fn new() -> Table { Table } pub fn make() -> Self { Self::new() } }\n\
             pub fn caller(limit: u8, name: &str) -> u8 {\n\
             let pick = tools::helper;\n\
             let _table = Table::new();\n\
             let local = 3;\n\
             tools::run(7, &name) + limit + local + pick()\n\
             }\n",
        );
        let caller = facts(&krate, &item(0, 0));
        let found: Vec<(String, Role, usize)> = caller
            .calls
            .iter()
            .map(|c| (c.path.join("::"), c.role, c.args))
            .collect();
        let s = str::to_owned;
        assert_eq!(
            found,
            [
                (s("tools::helper"), Role::ValueEscape, 0),
                (s("Table::new"), Role::Call, 0),
                (s("tools::run"), Role::Call, 2),
            ]
        );
        let run = &caller.calls[2];
        assert_eq!(run.literal_args, [(0, s("7"))]);
        assert_eq!(run.passed_params, [(1, s("name"))]);
        let reached = krate.refs[run.full_ref].value().expect("run resolves");
        assert_eq!(
            krate.modules[reached.module].items[reached.item].name,
            "run"
        );
        let table = caller.calls[1].prefix_ref.expect("a prefix");
        assert_eq!(
            krate.refs[table]
                .decl()
                .map(|d| krate.decls[d].name.as_str()),
            Some("Table")
        );
        let uses: Vec<(String, usize)> = caller
            .params
            .iter()
            .map(|p| (p.name.clone(), p.uses))
            .collect();
        assert_eq!(uses, [(s("limit"), 1), (s("name"), 1)]);
        let make = facts(&krate, &Owner::Impl(0, s("make")));
        let via_self = &make.calls[0];
        assert_eq!(via_self.path, ["Self", "new"]);
        assert_eq!(
            krate.refs[via_self.full_ref].decl(),
            None,
            "Self waits for analysis"
        );
        for call in caller.calls.iter().chain(&make.calls) {
            assert!(krate.refs[call.full_ref].comment);
            assert!(krate.body_names.iter().all(|b| b.name != call.full_ref));
        }
    }

    #[test]
    fn chains_loops_counts_and_single_statements() {
        let (_tmp, krate) = collected(
            "pub struct Queue { items: Vec<u8>, seen: u8 }\n\
             impl Queue {\n\
             pub fn drain(&mut self, extra: &mut Vec<u8>) -> Option<u8> {\n\
             for x in self.items.iter() { if extra.contains(x) { extra.remove(0); } }\n\
             self.items.push(1);\n\
             self.seen += 1;\n\
             let first = self.items.first().copied()?;\n\
             if first == 0 { return None; }\n\
             Err::<u8, u8>(2).ok();\n\
             todo!()\n\
             }\n\
             pub fn len(&self) -> usize { self.items.len() }\n\
             pub fn seen(&self) -> &u8 { &self.seen }\n\
             }\n\
             pub async fn wait(fut: std::future::Ready<u8>) -> u8 { fut.await }\n",
        );
        let s = str::to_owned;
        let drain = facts(&krate, &Owner::Impl(0, s("drain")));
        let chain = |root: Root, methods: &[&str], first: Option<&str>, in_loop, write| OpChain {
            root,
            methods: methods.iter().map(|m| (*m).to_owned()).collect(),
            first_literal: first.map(str::to_owned),
            in_loop,
            write,
        };
        let field = |name: &str| Root::Field(name.to_owned());
        let param = |name: &str| Root::Param(name.to_owned());
        assert_eq!(
            drain.chains,
            [
                chain(field("items"), &["first", "copied"], None, false, false),
                chain(field("items"), &["iter"], None, false, false),
                chain(field("items"), &["push"], Some("1"), false, false),
                chain(field("seen"), &[], None, false, true),
                chain(param("extra"), &["contains"], None, true, false),
                chain(param("extra"), &["remove"], Some("0"), true, false),
            ]
        );
        let counts = Counts {
            tries: 1,
            errs: 1,
            nones: 1,
            awaits: 0,
            panics: 1,
        };
        assert_eq!(drain.counts, counts);
        assert_eq!((drain.loops, drain.loop_depth, drain.statements), (1, 1, 7));
        assert_eq!(drain.macros, ["todo"]);
        assert_eq!(drain.single, None);
        let uses: Vec<usize> = drain.params.iter().map(|p| p.uses).collect();
        assert_eq!(uses, [4, 2], "self, then extra");
        let len = facts(&krate, &Owner::Impl(0, s("len")));
        assert_eq!(len.single, Some(Single::Method(s("len"))));
        let seen = facts(&krate, &Owner::Impl(0, s("seen")));
        assert_eq!(seen.single, Some(Single::Field(s("seen"))));
        let wait = facts(&krate, &item(0, 0));
        assert_eq!((wait.counts.awaits, wait.single.clone()), (1, None));
    }

    #[test]
    fn parameter_patterns_bind_each_name_once() {
        let (_tmp, krate) = collected(
            "pub struct P { pub x: u8, pub y: u8 }\n\
             pub fn f((a, b): (u8, u8), P { x, .. }: P, _: u8, mut c: u8, r#type: u8) -> u8 {\n\
             a + b + x + c + r#type\n\
             }\n",
        );
        let names: Vec<&str> = krate.bodies[0]
            .facts
            .params
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        assert_eq!(names, ["a", "b", "x", "c", "type"]);
        assert!(krate.bodies[0].facts.params.iter().all(|p| p.uses == 1));
        assert!(krate.bodies[0].facts.calls.is_empty());
    }
}
