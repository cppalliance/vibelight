//! Spells declarations for the markdown outline as one line of Rust each,
//! the way the source writes them without bodies or comments: a type's
//! whole shape with its fields or variants, a function's signature with its
//! `where` clause inline, and the head of every other item. No line ends in
//! `;`, and no initializer prints but a single literal.

use std::fmt::Write as _;

use syn::{Expr, UnOp};

use crate::model::{
    Decl, DeclKind, Foreign, MemberKind, Meta, Param, Receiver, Shape, Sig, Text, TypeRef, Variant,
    VariantFields, Where,
};

/// The longest initializer a constant or static shows.
const VALUE_WIDTH: usize = 100;

/// Returns text as Rust spells it, with each type name as written.
pub(crate) fn rust(text: &Text, refs: &[TypeRef]) -> String {
    text.rust_visible(refs)
}

/// Returns a visibility as written with the space after it, or nothing.
fn vis(written: &str) -> String {
    if written.is_empty() {
        String::new()
    } else {
        format!("{written} ")
    }
}

/// Returns a `where` clause on one line after a space, or nothing.
fn where_text(clause: Option<&Where>, refs: &[TypeRef]) -> String {
    match clause {
        Some(clause) if !clause.0.is_empty() => format!(" {}", rust(&clause.joined(), refs)),
        _ => String::new(),
    }
}

/// Returns the `cfg` and API attributes of a field or a variant as they
/// print inside a shape, each followed by a space.
fn inline_attrs(meta: &Meta) -> String {
    let mut out = String::new();
    for cfg in &meta.cfgs {
        let _ = write!(out, "#[cfg({cfg})] ");
    }
    for attr in &meta.attrs {
        let _ = write!(out, "#[{attr}] ");
    }
    out
}

/// Spells a qualifier the way Rust writes it.
fn qualifier(text: &str) -> &str {
    if text == "constexpr" { "const" } else { text }
}

/// Returns a function's signature on one line: its visibility, qualifiers,
/// name, generics, receiver and parameters, return type, and `where`
/// clause.
pub(crate) fn sig(sig: &Sig, written_vis: &str, refs: &[TypeRef]) -> String {
    let mut out = vis(written_vis);
    for qual in &sig.quals {
        out.push_str(qualifier(qual));
        out.push(' ');
    }
    let _ = write!(out, "fn {}{}(", sig.name, rust(&sig.generics, refs));
    let mut params: Vec<String> = match &sig.receiver {
        Receiver::Shared(text) | Receiver::Mutable(text) => vec![text.clone()],
        Receiver::None | Receiver::Value => Vec::new(),
    };
    params.extend(sig.params.iter().map(|p| rust(&p.text, refs)));
    out.push_str(&params.join(", "));
    out.push(')');
    if let Some(ret) = &sig.ret {
        let _ = write!(out, " -> {}", rust(ret, refs));
    }
    out.push_str(&where_text(sig.where_clause.as_ref(), refs));
    out
}

/// Returns a declaration's whole shape: a struct's or union's fields, an
/// enum's variants, a trait's header, or an alias's target.
pub(crate) fn shape(decl: &Decl, refs: &[TypeRef]) -> String {
    let mut out = vis(&decl.meta.vis);
    if let Shape::Trait { prefix, .. } = &decl.shape {
        if !prefix.trim().is_empty() {
            let _ = write!(out, "{} ", prefix.trim());
        }
    }
    let keyword = match decl.kind {
        DeclKind::Struct => "struct",
        DeclKind::Enum => "enum",
        DeclKind::Union => "union",
        DeclKind::Trait | DeclKind::TraitAlias => "trait",
        DeclKind::Alias => "type",
    };
    let _ = write!(out, "{keyword} {}{}", decl.name, rust(&decl.generics, refs));
    let clause = where_text(decl.where_clause.as_ref(), refs);
    match &decl.shape {
        Shape::Alias(target) => {
            let _ = write!(out, " = {}{clause}", rust(target, refs));
        }
        Shape::Unit => out.push_str(&clause),
        Shape::Tuple(fields) => {
            let fields: Vec<String> = fields.iter().map(|f| tuple_field(f, refs)).collect();
            let _ = write!(out, "({}){clause}", fields.join(", "));
        }
        Shape::Named => {
            let fields: Vec<String> = decl
                .members
                .iter()
                .filter_map(|member| match &member.kind {
                    MemberKind::Field(ty) => Some(format!(
                        "{}{}{}: {}",
                        inline_attrs(&member.meta),
                        vis(&member.meta.vis),
                        member.name,
                        rust(ty, refs)
                    )),
                    _ => None,
                })
                .collect();
            out.push_str(&clause);
            out.push_str(&braced(&fields));
        }
        Shape::Enum(variants) => {
            let variants: Vec<String> = variants.iter().map(|v| variant(v, refs)).collect();
            out.push_str(&clause);
            out.push_str(&braced(&variants));
        }
        Shape::Trait { bounds, .. } => {
            if !bounds.is_empty() {
                let _ = write!(out, ": {bounds}");
            }
            out.push_str(&clause);
        }
    }
    out
}

/// Returns entries in braces after a space, or ` {}` for none.
fn braced(entries: &[String]) -> String {
    if entries.is_empty() {
        " {}".to_owned()
    } else {
        format!(" {{ {} }}", entries.join(", "))
    }
}

/// Returns a tuple field or a variant's field as a shape writes it.
pub(crate) fn tuple_field(param: &Param, refs: &[TypeRef]) -> String {
    format!(
        "{}{}{}",
        inline_attrs(&param.meta),
        vis(&param.meta.vis),
        rust(&param.text, refs)
    )
}

/// Returns a named field as a shape writes it.
pub(crate) fn named_field(meta: &Meta, name: &str, ty: &Text, refs: &[TypeRef]) -> String {
    format!(
        "{}{}{name}: {}",
        inline_attrs(meta),
        vis(&meta.vis),
        rust(ty, refs)
    )
}

/// Returns a variant as its enum's shape writes it: its attributes, its
/// name, its fields, and its discriminant.
pub(crate) fn variant(variant: &Variant, refs: &[TypeRef]) -> String {
    let mut out = format!("{}{}", inline_attrs(&variant.meta), variant.name);
    match &variant.fields {
        VariantFields::Unit => {}
        VariantFields::Tuple(fields) => {
            let fields: Vec<String> = fields.iter().map(|f| tuple_field(f, refs)).collect();
            let _ = write!(out, "({})", fields.join(", "));
        }
        VariantFields::Named(fields) => {
            let fields: Vec<String> = fields.iter().map(|f| tuple_field(f, refs)).collect();
            out.push_str(&braced(&fields));
        }
    }
    if let Some(value) = &variant.discriminant {
        let _ = write!(out, " = {value}");
    }
    out
}

/// Returns an associated constant's or type's head: `const N: usize`, or
/// `type Item: Clone` in a trait and `type Item = u8` in an impl.
pub(crate) fn member_head(kind: &MemberKind, name: &str, meta: &Meta, refs: &[TypeRef]) -> String {
    match kind {
        MemberKind::Const { ty, .. } => {
            format!("{}const {name}: {}", vis(&meta.vis), rust(ty, refs))
        }
        MemberKind::Type(text) => format!("{}type {}", vis(&meta.vis), rust(text, refs)),
        MemberKind::Field(ty) => named_field(meta, name, ty, refs),
        MemberKind::Fn(sig) => self::sig(sig, &meta.vis, refs),
        MemberKind::Mark(text) | MemberKind::Unsupported(text) => text.clone(),
    }
}

/// Returns a constant's head, `pub const N: usize`.
pub(crate) fn const_head(name: &str, ty: &Text, meta: &Meta, refs: &[TypeRef]) -> String {
    format!("{}const {name}: {}", vis(&meta.vis), rust(ty, refs))
}

/// Returns a static's head, `pub static mut N: usize`.
pub(crate) fn static_head(
    name: &str,
    mutable: bool,
    ty: &Text,
    meta: &Meta,
    refs: &[TypeRef],
) -> String {
    let mutable = if mutable { "mut " } else { "" };
    format!(
        "{}static {mutable}{name}: {}",
        vis(&meta.vis),
        rust(ty, refs)
    )
}

/// Returns an extern block's item inside a block of its own ABI, such as
/// `extern "C" { pub fn write(fd: i32) -> isize }`.
pub(crate) fn extern_item(
    abi: &str,
    name: &str,
    item: &Foreign,
    meta: &Meta,
    refs: &[TypeRef],
) -> String {
    let inner = match item {
        Foreign::Fn(sig) => self::sig(sig, &meta.vis, refs),
        Foreign::Static { mutable, ty } => static_head(name, *mutable, ty, meta, refs),
        Foreign::Type => format!("{}type {name}", vis(&meta.vis)),
    };
    let abi = if abi.is_empty() {
        String::new()
    } else {
        format!("{abi} ")
    };
    format!("extern {abi}{{ {inner} }}")
}

/// Returns a re-export, `pub use path::Item as Alias`.
pub(crate) fn reexport(text: &Text, meta: &Meta, refs: &[TypeRef]) -> String {
    format!("{}use {}", vis(&meta.vis), rust(text, refs))
}

/// Returns an initializer when it is one literal, signed or not, and short
/// enough to show.
pub(crate) fn literal(value: &str) -> Option<String> {
    let expr: Expr = syn::parse_str(value).ok()?;
    let lit = match &expr {
        Expr::Lit(_) => true,
        Expr::Unary(unary) => {
            matches!(unary.op, UnOp::Neg(_)) && matches!(*unary.expr, Expr::Lit(_))
        }
        _ => false,
    };
    (lit && value.chars().count() <= VALUE_WIDTH).then(|| value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_short_literal_initializer_shows() {
        assert_eq!(literal("5").as_deref(), Some("5"));
        assert_eq!(literal("-1.5").as_deref(), Some("-1.5"));
        assert_eq!(literal("\"a | b\"").as_deref(), Some("\"a | b\""));
        assert_eq!(literal("Duration::from_secs(5)"), None);
        assert_eq!(literal("1 << 3"), None);
        assert_eq!(literal(&format!("\"{}\"", "x".repeat(200))), None);
    }
}
