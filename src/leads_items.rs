//! What a detector group reads from a run, gathered once: every field of
//! every struct, union, and enum variant, and every function, each with
//! its compared types, and the impl facts that several detectors share.

use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::analysis_paths::PathKind;
use crate::leads::Lead;
use crate::leads_types::{Tok, Ty, identity};
use crate::leads_words::{bound_name, split_param};
use crate::model::{
    Crate, DeclId, DeclKind, ItemKind, Loc, MemberKind, Meta, ModId, Receiver, Shape, Sig,
    ValueRef, VariantFields, Vis,
};
use crate::model_body::Role;
use crate::typerefs::strip_raw;

/// One field: of a struct or union, or of an enum's variant.
pub(crate) struct Field<'a> {
    /// The declaration that holds it.
    pub(crate) decl: ItemId,
    /// The index of the variant that holds it, for an enum.
    pub(crate) variant: Option<usize>,
    /// Its name without `r#`, or a tuple field's position.
    pub(crate) name: String,
    /// Whether it has a name rather than a position.
    pub(crate) named: bool,
    pub(crate) ty: Ty,
    pub(crate) vis: Vis,
    pub(crate) loc: Loc,
    pub(crate) meta: &'a Meta,
}

impl Field<'_> {
    /// Returns what the field's subject path adds to its declaration's:
    /// its variant's name, then its own.
    pub(crate) fn part(&self, run: &AnalysisRun<'_>) -> Vec<String> {
        let mut part = Vec::new();
        if let (Some(v), ItemRef::Decl(d)) = (self.variant, self.decl.item) {
            if let Shape::Enum(variants) = &run.krates[self.decl.target].decls[d].shape {
                part.push(variants[v].name.clone());
            }
        }
        part.push(self.name.clone());
        part
    }

    /// Adds the field to a lead as a subject, located where it is written.
    pub(crate) fn add(&self, run: &AnalysisRun<'_>, lead: Lead) -> Lead {
        lead.part(run, self.decl, &self.part(run))
            .at(self.decl.target, self.loc)
    }
}

/// Where a function is, which decides how free its signature is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FnKind {
    /// A module's function.
    Free,
    /// A member of an inherent impl.
    Inherent,
    /// A member of a trait impl, whose signature its trait sets.
    TraitImpl,
    /// A trait's own function.
    Trait,
}

/// One parameter other than the receiver.
pub(crate) struct Arg {
    /// The one name its pattern binds, without `r#`.
    pub(crate) name: Option<String>,
    pub(crate) ty: Ty,
}

/// One function of the run.
pub(crate) struct Func<'a> {
    pub(crate) item: ItemId,
    pub(crate) kind: FnKind,
    /// The name without `r#`.
    pub(crate) name: String,
    pub(crate) sig: &'a Sig,
    /// The module that declares it; a member's is its declaration's.
    pub(crate) module: ModId,
    /// The declaration it is a member of: its impl's type, or its trait.
    pub(crate) decl: Option<DeclId>,
    pub(crate) args: Vec<Arg>,
    /// The return type, with its declaration's own name as `Self`.
    pub(crate) ret: Option<Ty>,
    /// The fields its body reaches through `self`, sorted, each once.
    pub(crate) self_fields: &'a [String],
    pub(crate) vis: Vis,
}

impl Func<'_> {
    /// Tells whether the function takes `self` in any form.
    pub(crate) fn has_receiver(&self) -> bool {
        self.sig.receiver != Receiver::None
    }

    /// Tells whether the function is a type's own: free, or a member of
    /// an inherent impl, whose signature no trait sets.
    pub(crate) fn is_own(&self) -> bool {
        matches!(self.kind, FnKind::Free | FnKind::Inherent)
    }

    /// Returns the signature as one compared type, without names: the
    /// receiver, the parameters' types, the return type, the generics, and
    /// the `where` clause.
    pub(crate) fn signature(&self, run: &AnalysisRun<'_>) -> Ty {
        let t = self.item.target;
        let impl_member = matches!(self.kind, FnKind::Inherent | FnKind::TraitImpl);
        let own = (self.decl.filter(|_| impl_member)).map(|d| ItemId {
            target: t,
            item: ItemRef::Decl(d),
        });
        let receiver = match self.sig.receiver {
            Receiver::None => "fn",
            Receiver::Shared => "&self",
            Receiver::Mutable => "&mut self",
            Receiver::Value => "self",
        };
        let code = |text: &str| Tok::Code(text.to_owned());
        let mut toks = vec![code(receiver), code("(")];
        for (i, arg) in self.args.iter().enumerate() {
            if i > 0 {
                toks.push(code(","));
            }
            toks.extend(arg.ty.0.iter().cloned());
        }
        toks.extend([code(")"), code("->")]);
        toks.extend(self.ret.iter().flat_map(|ret| ret.0.iter().cloned()));
        toks.push(code("<"));
        toks.extend(Ty::of(run, t, &self.sig.generics, own).0);
        toks.extend([code(">"), code("where")]);
        if let Some(clause) = &self.sig.where_clause {
            toks.extend(Ty::of(run, t, &clause.joined(), own).0);
        }
        Ty(toks)
    }

    /// Returns the parameters' and the return type as a reader sees them,
    /// such as `(&str, u32) -> bool`.
    pub(crate) fn shown(&self) -> String {
        let args: Vec<String> = self.args.iter().map(|arg| arg.ty.shown()).collect();
        let ret = self.ret.as_ref().map(|ret| format!(" -> {}", ret.shown()));
        format!("({}){}", args.join(", "), ret.unwrap_or_default())
    }
}

/// Every field and function of a run.
pub(crate) struct Catalog<'a> {
    pub(crate) fields: Vec<Field<'a>>,
    pub(crate) funcs: Vec<Func<'a>>,
}

impl<'a> Catalog<'a> {
    pub(crate) fn new(run: &AnalysisRun<'a>) -> Catalog<'a> {
        let mut catalog = Catalog {
            fields: Vec::new(),
            funcs: Vec::new(),
        };
        let krates: &'a [Crate] = run.krates;
        for (t, krate) in krates.iter().enumerate() {
            catalog.add_fields(run, t, krate);
            catalog.add_funcs(run, t, krate);
        }
        catalog
    }

    /// Returns the fields of declaration `decl`, of its struct or union
    /// shape or of its variant `variant`.
    pub(crate) fn fields_of(
        &self,
        decl: ItemId,
        variant: Option<usize>,
    ) -> impl Iterator<Item = &Field<'a>> {
        let start = self
            .fields
            .partition_point(|f| (f.decl, f.variant) < (decl, variant));
        self.fields[start..]
            .iter()
            .take_while(move |f| f.decl == decl && f.variant == variant)
    }

    fn add_fields(&mut self, run: &AnalysisRun<'a>, t: usize, krate: &'a Crate) {
        for (d, decl) in krate.decls.iter().enumerate() {
            let at = ItemId {
                target: t,
                item: ItemRef::Decl(d),
            };
            let mut field = |variant, name: String, named, ty, vis, loc, meta| {
                self.fields.push(Field {
                    decl: at,
                    variant,
                    name,
                    named,
                    ty,
                    vis,
                    loc,
                    meta,
                });
            };
            match &decl.shape {
                Shape::Named => {
                    for member in &decl.members {
                        if let MemberKind::Field(ty) = &member.kind {
                            let ty = Ty::of(run, t, ty, Some(at));
                            let name = strip_raw(&member.name);
                            field(None, name, true, ty, member.vis, member.loc, &member.meta);
                        }
                    }
                }
                Shape::Tuple(params) => {
                    for (k, param) in params.iter().enumerate() {
                        let ty = Ty::of(run, t, &param.text, Some(at));
                        let vis = vis_of(&param.meta.vis);
                        field(None, k.to_string(), false, ty, vis, decl.loc, &param.meta);
                    }
                }
                Shape::Enum(variants) => {
                    for (v, variant) in variants.iter().enumerate() {
                        let (named, params) = match &variant.fields {
                            VariantFields::Named(params) => (true, params),
                            VariantFields::Tuple(params) => (false, params),
                            VariantFields::Unit => continue,
                        };
                        for (k, param) in params.iter().enumerate() {
                            let (name, ty) = match split_param(&param.text).filter(|_| named) {
                                Some((name, segs)) => {
                                    (strip_raw(&name), Ty::segs(run, t, &segs, Some(at)))
                                }
                                None => (k.to_string(), Ty::of(run, t, &param.text, Some(at))),
                            };
                            let (loc, meta) = (variant.loc, &param.meta);
                            field(Some(v), name, named, ty, Vis::Public, loc, meta);
                        }
                    }
                }
                Shape::Trait { .. } | Shape::Unit | Shape::Alias(_) => {}
            }
        }
    }

    fn add_funcs(&mut self, run: &AnalysisRun<'a>, t: usize, krate: &'a Crate) {
        let at = |item| ItemId { target: t, item };
        for (m, module) in krate.modules.iter().enumerate() {
            for (k, item) in module.items.iter().enumerate() {
                if let ItemKind::Fn(sig) = &item.kind {
                    let place = Place {
                        item: at(ItemRef::Value(ValueRef { module: m, item: k })),
                        module: m,
                        decl: None,
                        kind: FnKind::Free,
                        own: None,
                        vis: item.vis,
                    };
                    self.add_func(run, place, sig, &[]);
                }
            }
        }
        for (i, imp) in krate.impls.iter().enumerate() {
            let decl = run.attached[t][i];
            let module = decl.map_or_else(|| impl_module(krate, i), |d| krate.decls[d].module);
            let members = imp.members.iter().map(|m| (m, FnKind::Inherent));
            let traits = imp.trait_members.iter().map(|m| (m, FnKind::TraitImpl));
            let inherent = (0..imp.members.len()).map(|k| ItemRef::Inherent(i, k));
            let traited = (0..imp.trait_members.len()).map(|k| ItemRef::TraitImpl(i, k));
            for ((member, kind), item) in members.chain(traits).zip(inherent.chain(traited)) {
                if let MemberKind::Fn(sig) = &member.kind {
                    let place = Place {
                        item: at(item),
                        module,
                        decl,
                        kind,
                        own: decl.map(|d| at(ItemRef::Decl(d))),
                        vis: member.vis,
                    };
                    self.add_func(run, place, sig, &member.self_fields);
                }
            }
        }
        for (d, decl) in krate.decls.iter().enumerate() {
            if decl.kind != DeclKind::Trait {
                continue;
            }
            for (k, member) in decl.members.iter().enumerate() {
                if let MemberKind::Fn(sig) = &member.kind {
                    let place = Place {
                        item: at(ItemRef::Own(d, k)),
                        module: decl.module,
                        decl: Some(d),
                        kind: FnKind::Trait,
                        own: None,
                        vis: member.vis,
                    };
                    self.add_func(run, place, sig, &member.self_fields);
                }
            }
        }
    }

    fn add_func(
        &mut self,
        run: &AnalysisRun<'a>,
        place: Place,
        sig: &'a Sig,
        self_fields: &'a [String],
    ) {
        let t = place.item.target;
        self.funcs.push(Func {
            item: place.item,
            kind: place.kind,
            name: strip_raw(&sig.name),
            sig,
            module: place.module,
            decl: place.decl,
            args: args(run, t, sig, place.own),
            ret: sig.ret.as_ref().map(|ret| Ty::of(run, t, ret, place.own)),
            self_fields,
            vis: place.vis,
        });
    }
}

/// Where a function is declared, and how far it is visible.
#[derive(Clone, Copy)]
struct Place {
    item: ItemId,
    module: ModId,
    decl: Option<DeclId>,
    kind: FnKind,
    /// The declaration whose own name compares as `Self`: an impl's type.
    own: Option<ItemId>,
    vis: Vis,
}

/// Returns a signature's parameters after its receiver.
fn args(run: &AnalysisRun<'_>, t: usize, sig: &Sig, own: Option<ItemId>) -> Vec<Arg> {
    let skip = usize::from(sig.receiver == Receiver::Value);
    sig.params
        .iter()
        .skip(skip)
        .map(|param| match split_param(&param.text) {
            Some((pattern, segs)) => Arg {
                name: bound_name(&pattern),
                ty: Ty::segs(run, t, &segs, own),
            },
            None => Arg {
                name: None,
                ty: Ty::of(run, t, &param.text, own),
            },
        })
        .collect()
}

/// Returns a visibility from how it is written.
pub(crate) fn vis_of(written: &str) -> Vis {
    match written {
        "pub" => Vis::Public,
        "" | "pub(self)" => Vis::Private,
        _ => Vis::Internal,
    }
}

/// Returns the module whose path is a module's without its last segment.
pub(crate) fn parent_module(krate: &Crate, m: ModId) -> Option<ModId> {
    let (_, up) = krate.modules[m].path.split_last()?;
    krate.modules.iter().position(|module| module.path == up)
}

/// Returns the module an impl is written in: where its self type's first
/// name is written, or its trait's.
pub(crate) fn impl_module(krate: &Crate, i: usize) -> ModId {
    let imp = &krate.impls[i];
    let first = imp.self_text.refs().next();
    first
        .or_else(|| imp.trait_.as_ref().and_then(|base| base.path_ref))
        .map_or(0, |r| krate.refs[r].module)
}

/// Returns what impl `i` of target `t` implements, as a compared name such
/// as `std::fmt::Display`; `None` for an inherent impl.
pub(crate) fn trait_of(run: &AnalysisRun<'_>, t: usize, i: usize) -> Option<String> {
    let base = run.krates[t].impls[i].trait_.as_ref()?;
    match base.path_ref {
        Some(r) => Some(identity(run, t, r, None)),
        None => Some(base.name.clone()),
    }
}

/// Returns the impls of declaration `d` of target `t` that implement the
/// trait `name` names.
pub(crate) fn impls_of_trait<'r>(
    run: &'r AnalysisRun<'_>,
    t: usize,
    d: DeclId,
    name: &'r str,
) -> impl Iterator<Item = usize> + 'r {
    run.impls_of[t][d]
        .iter()
        .copied()
        .filter(move |&i| trait_of(run, t, i).as_deref() == Some(name))
}

/// Tells whether a trait's compared name is serde's `Deserialize`.
pub(crate) fn is_deserialize(name: &str) -> bool {
    name.starts_with("serde::") && name.ends_with("::Deserialize")
}

/// Tells whether a body of the run reads a function as a value that may
/// leave it, such as a handler passed to a router, so that a callback's
/// signature may bind the function's.
pub(crate) fn escapes(run: &AnalysisRun<'_>, item: ItemId) -> bool {
    run.edges_to(item)
        .any(|edge| edge.kind == PathKind::Call(Role::ValueEscape))
}

/// Tells whether a declaration is a wire type: it derives `Serialize` or
/// `Deserialize`, or carries a `serde` attribute.
pub(crate) fn is_wire(run: &AnalysisRun<'_>, decl: ItemId) -> bool {
    let ItemRef::Decl(d) = decl.item else {
        return false;
    };
    let decl = &run.krates[decl.target].decls[d];
    let derived = (decl.written_derives.iter())
        .any(|name| matches!(name.rsplit("::").next(), Some("Deserialize" | "Serialize")));
    derived || decl.meta.attrs.iter().any(|attr| attr.starts_with("serde"))
}
