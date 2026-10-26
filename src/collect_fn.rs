//! Collects functions: qualifiers, receivers, parameters, and returns.

use std::ops::Range;

use syn::visit::Visit;
use syn::{Attribute, FnArg, ItemFn, ReceiverKind, ReturnType, Safety, Signature, Type};

use crate::attrs;
use crate::collect::{Collector, Here, loc};
use crate::comments::{Attached, attach};
use crate::model::{Holder, Item, ItemKind, Meta, Param, Receiver, Sig, Text};
use crate::source::Comment;
use crate::typerefs::range_of;

impl Collector<'_> {
    pub(crate) fn item_fn(
        &mut self,
        here: &Here,
        f: &ItemFn,
        lead: Vec<Comment>,
        trail: &[Comment],
    ) {
        if attrs::is_test_only(&f.attrs) {
            return;
        }
        let files = self.files;
        let file = &files[here.file];
        self.discard(here, range_of(&*f.block, file));
        let here = here.with(&f.sig.generics);
        let sig = self.sig(&here, &f.sig);
        let loc = loc(&here, f.sig.ident.span());
        let notes = self.node_notes(&here, &f.attrs, lead, range_of(f, file));
        let mark = self.attribute_mark(&f.attrs, false, Holder::Module(here.module), loc);
        let meta = Meta {
            notes,
            trailing: self.trailing(&here, trail),
            cfgs: attrs::cfgs(&f.attrs, file),
            marks: mark.into_iter().collect(),
        };
        self.modules[here.module].items.push(Item {
            name: f.sig.ident.to_string(),
            public: attrs::is_public(&f.vis),
            kind: ItemKind::Fn(sig),
            meta,
            loc,
        });
    }

    /// Splits a signature into its parts, collecting comments on each
    /// parameter. `here` must already hold the function's own generics.
    pub(crate) fn sig(&mut self, here: &Here, sig: &Signature) -> Sig {
        let files = self.files;
        let file = &files[here.file];
        let mut quals = Vec::new();
        if sig.constness.is_some() {
            quals.push("constexpr".to_owned());
        }
        if sig.asyncness.is_some() {
            quals.push("async".to_owned());
        }
        match &sig.safety {
            Safety::Unsafe(_) => quals.push("unsafe".to_owned()),
            Safety::Safe(_) => quals.push("safe".to_owned()),
            Safety::Default => {}
        }
        if let Some(abi) = &sig.abi {
            quals.push(match &abi.name {
                Some(name) => format!("extern \"{}\"", name.value()),
                None => "extern".to_owned(),
            });
        }
        let generics = self.generics_text(here, &sig.generics);

        let scope = file.end(sig.paren_token.span.open())..file.start(sig.paren_token.span.close());
        let mut children: Vec<Range<usize>> =
            sig.inputs.iter().map(|a| range_of(a, file)).collect();
        if let Some(variadic) = &sig.variadic {
            children.push(range_of(variadic, file));
        }
        let Attached {
            leading, trailing, ..
        } = attach(&mut self.pools[here.file], file, scope, &children);
        let mut comments = leading.into_iter().zip(trailing);
        let mut receiver = Receiver::None;
        let mut params = Vec::new();
        for arg in &sig.inputs {
            let (lead, trail) = comments.next().unwrap_or_default();
            let range = range_of(arg, file);
            match arg {
                FnArg::Receiver(r) => {
                    receiver = match &r.kind {
                        ReceiverKind::Reference(_, _, None) => Receiver::Shared,
                        ReceiverKind::Reference(_, _, Some(_)) => Receiver::Mutable,
                        ReceiverKind::Typed(_, ty) => typed_receiver(ty),
                        _ => Receiver::Value,
                    };
                    if receiver == Receiver::Value {
                        let text = self.slicer(here).node(r, |f| f.visit_receiver(r));
                        params.push(self.param(here, &r.attrs, text, lead, trail, range));
                    } else {
                        self.discard(here, range);
                    }
                }
                FnArg::Typed(pt) => {
                    let text = self.slicer(here).node(pt, |f| f.visit_type(&pt.ty));
                    params.push(self.param(here, &pt.attrs, text, lead, trail, range));
                }
            }
        }
        if let Some(variadic) = &sig.variadic {
            let (lead, trail) = comments.next().unwrap_or_default();
            let range = range_of(variadic, file);
            let text = Text::plain(file.clean(range.clone()));
            params.push(self.param(here, &variadic.attrs, text, lead, trail, range));
        }
        let ret = match &sig.output {
            ReturnType::Type(_, ty) => Some(self.slicer(here).node(&**ty, |f| f.visit_type(ty))),
            ReturnType::Default => None,
        };
        let where_clause = self.where_text(here, sig.generics.where_clause.as_ref());
        Sig {
            quals,
            name: sig.ident.to_string(),
            generics,
            receiver,
            params,
            ret,
            where_clause,
        }
    }

    /// Builds a parameter whose comments, leading and trailing alike, print
    /// above it.
    fn param(
        &mut self,
        here: &Here,
        attrs: &[Attribute],
        text: Text,
        mut lead: Vec<Comment>,
        trail: Vec<Comment>,
        range: Range<usize>,
    ) -> Param {
        lead.extend(trail);
        let notes = self.node_notes(here, attrs, lead, range);
        Param {
            text,
            meta: Meta {
                notes,
                ..Meta::default()
            },
        }
    }
}

/// Classifies `self: Type`: `&Self` and `&mut Self` count as `&self` and
/// `&mut self`; anything else is taken by value.
fn typed_receiver(ty: &Type) -> Receiver {
    if let Type::Reference(reference) = ty {
        let is_self = matches!(&*reference.elem, Type::Path(p) if p.qself.is_none() && p.path.is_ident("Self"));
        if is_self {
            return if reference.mutability.is_some() {
                Receiver::Mutable
            } else {
                Receiver::Shared
            };
        }
    }
    Receiver::Value
}
