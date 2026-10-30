//! Collects functions: qualifiers, receivers, parameters, and returns.

use std::ops::Range;

use syn::visit::Visit;
use syn::{Attribute, FnArg, ItemFn, ReceiverKind, ReturnType, Safety, Signature, Type};

use crate::attrs;
use crate::collect::{Collector, Here, loc};
use crate::collect_body_code::Code;
use crate::comments::{Placed, place_comments};
use crate::model::{BodyOwner, Holder, Item, ItemKind, Meta, Param, Receiver, Sig, Text};
use crate::model_body::BodyKind;
use crate::typerefs::range_of;

impl Collector<'_> {
    pub(crate) fn item_fn(&mut self, here: &Here, f: &ItemFn, placed: Placed) {
        if self.test_only(here, &f.attrs, f) {
            return;
        }
        let files = self.files;
        let file = &files[here.file];
        let here = here.with(&f.sig.generics);
        let owner = self.next_item(&here);
        let code = Code::Block(&f.block);
        self.body(&here, BodyKind::Fn, owner, code, Some(&f.sig));
        self.body_names(&here, &f.block, BodyOwner::Module);
        let sig = self.sig(&here, &f.sig);
        let loc = loc(&here, f.sig.ident.span());
        let notes = self.node_notes(&here, &f.attrs, placed.leading, range_of(f, file));
        let mark = self.attribute_mark(&f.attrs, false, Holder::Module(here.module), loc);
        let (raw_doc, lints) = self.attr_facts(&here, &f.attrs);
        let meta = Meta {
            notes,
            trailing: self.trailing(&here, &placed.trailing),
            cfgs: attrs::cfgs(&f.attrs, file),
            attrs: attrs::api(&f.attrs, file),
            marks: mark.into_iter().collect(),
            vis: attrs::spelled(&f.vis, file),
            raw_doc,
            lints,
        };
        self.modules[here.module].items.push(Item {
            name: f.sig.ident.to_string(),
            vis: attrs::vis(&f.vis),
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
        let (per_param, _) = place_comments(&mut self.pools[here.file], file, scope, &children);
        let mut comments = per_param.into_iter();
        let mut receiver = Receiver::None;
        let mut params = Vec::new();
        for arg in &sig.inputs {
            let placed = comments.next().unwrap_or_default();
            let range = range_of(arg, file);
            match arg {
                FnArg::Receiver(r) => {
                    let text = file.clean(range.clone());
                    receiver = match &r.kind {
                        ReceiverKind::Reference(_, _, None) => Receiver::Shared(text),
                        ReceiverKind::Reference(_, _, Some(_)) => Receiver::Mutable(text),
                        ReceiverKind::Typed(_, ty) => typed_receiver(ty, text),
                        _ => Receiver::Value,
                    };
                    if matches!(receiver, Receiver::Value) {
                        let text = self.slicer(here).node(r, |f| f.visit_receiver(r));
                        params.push(self.param(here, &r.attrs, text, placed, range));
                    } else {
                        self.discard(here, range);
                    }
                }
                FnArg::Typed(pt) => {
                    let text = self.slicer(here).node(pt, |f| f.visit_type(&pt.ty));
                    params.push(self.param(here, &pt.attrs, text, placed, range));
                }
            }
        }
        if let Some(variadic) = &sig.variadic {
            let placed = comments.next().unwrap_or_default();
            let range = range_of(variadic, file);
            let text = Text::plain(file.clean(range.clone()));
            params.push(self.param(here, &variadic.attrs, text, placed, range));
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
        placed: Placed,
        range: Range<usize>,
    ) -> Param {
        let mut lead = placed.leading;
        lead.extend(placed.trailing);
        let notes = self.node_notes(here, attrs, lead, range);
        let (raw_doc, lints) = self.attr_facts(here, attrs);
        Param {
            text,
            meta: Meta {
                notes,
                raw_doc,
                lints,
                ..Meta::default()
            },
        }
    }
}

/// Classifies `self: Type`: `&Self` and `&mut Self` count as `&self` and
/// `&mut self`, holding `text` as the receiver as written; anything else is
/// taken by value.
fn typed_receiver(ty: &Type, text: String) -> Receiver {
    if let Type::Reference(reference) = ty {
        let is_self = matches!(&*reference.elem, Type::Path(p) if p.qself.is_none() && p.path.is_ident("Self"));
        if is_self {
            return if reference.mutability.is_some() {
                Receiver::Mutable(text)
            } else {
                Receiver::Shared(text)
            };
        }
    }
    Receiver::Value
}
