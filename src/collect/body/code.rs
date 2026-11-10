//! The code of one body, what reading its syntax finds, and the syntax
//! helpers the reading shares: what a body of one statement does, and how
//! a path, a literal, or a parameter passed on is read.

use std::collections::BTreeSet;
use std::ops::Range;

use syn::{BinOp, Block, Expr, Member, Path, Stmt, UnOp};

use crate::model_body::{Counts, OpChain, Role, Single};
use crate::source::SourceFile;
use crate::typerefs::{range_of, strip_raw};

/// A path in expression position, before it has references.
pub(crate) struct Called {
    pub(crate) global: bool,
    pub(crate) path: Vec<String>,
    pub(crate) role: Role,
    pub(crate) args: usize,
    pub(crate) literal_args: Vec<(usize, String)>,
    pub(crate) passed_params: Vec<(usize, String)>,
}

/// One statement: its bytes, and how many statements hold it.
pub(crate) struct Statement {
    pub(crate) range: Range<usize>,
    pub(crate) depth: usize,
}

/// What reading a body's syntax finds.
#[derive(Default)]
pub(crate) struct Read {
    pub(crate) called: Vec<Called>,
    /// Each struct literal's path, with whether it starts with `::`.
    pub(crate) constructs: Vec<(bool, Vec<String>)>,
    /// Each path a pattern names, with whether it starts with `::`.
    pub(crate) patterns: Vec<(bool, Vec<String>)>,
    pub(crate) macros: BTreeSet<String>,
    pub(crate) counts: Counts,
    /// Every statement at every depth, in source order.
    pub(crate) statements: Vec<Statement>,
    pub(crate) loops: usize,
    pub(crate) loop_depth: usize,
    pub(crate) chains: BTreeSet<OpChain>,
    /// How often the syntax names each parameter, in the order of `params`.
    pub(crate) param_uses: Vec<usize>,
}

/// The code of one body.
#[derive(Clone, Copy)]
pub(crate) enum Code<'a> {
    /// A function's block.
    Block(&'a Block),
    /// An initializer or a discriminant.
    Expr(&'a Expr),
}

impl<'a> Code<'a> {
    /// Returns the bytes the body covers.
    pub(crate) fn range(self, file: &SourceFile) -> Range<usize> {
        match self {
            Code::Block(block) => range_of(block, file),
            Code::Expr(expr) => range_of(expr, file),
        }
    }

    /// Returns the statements directly in a block body, or `None` for an
    /// initializer that is not a block.
    pub(crate) fn stmts(self) -> Option<&'a [Stmt]> {
        match self {
            Code::Block(block) => Some(&block.stmts),
            Code::Expr(Expr::Block(expr)) => Some(&expr.block.stmts),
            Code::Expr(_) => None,
        }
    }

    /// Returns how many statements are directly in the body; an initializer
    /// that is not a block is one.
    pub(crate) fn statements(self) -> usize {
        self.stmts().map_or(1, <[Stmt]>::len)
    }

    /// Returns what the body does when it is one statement that calls a
    /// path or a method or reads a field.
    pub(crate) fn single(self) -> Option<Single> {
        let ((_, Some([Stmt::Expr(expr, _)])) | (Code::Expr(expr), None)) = (self, self.stmts())
        else {
            return None;
        };
        match peel(expr) {
            Expr::Call(call) => {
                let Expr::Path(path) = peel(&call.func) else {
                    return None;
                };
                path.qself
                    .is_none()
                    .then(|| Single::Call(segments(&path.path).1))
            }
            Expr::MethodCall(call) => Some(Single::Method(strip_raw(&call.method.to_string()))),
            Expr::Field(field) => Some(Single::Field(member_name(&field.member))),
            _ => None,
        }
    }
}

/// Returns a path's segments without `r#`, and whether it starts with `::`.
pub(crate) fn segments(path: &Path) -> (bool, Vec<String>) {
    let names = path
        .segments
        .iter()
        .map(|s| strip_raw(&s.ident.to_string()))
        .collect();
    (path.leading_colon.is_some(), names)
}

/// Returns an expression past `return`, `?`, `.await`, `&`, `*`, and
/// parentheses.
fn peel(mut expr: &Expr) -> &Expr {
    loop {
        expr = match expr {
            Expr::Return(ret) => match &ret.expr {
                Some(value) => value,
                None => return expr,
            },
            Expr::Try(try_) => &try_.expr,
            Expr::Await(await_) => &await_.base,
            Expr::Reference(reference) => &reference.expr,
            Expr::Unary(unary) if matches!(unary.op, UnOp::Deref(_)) => &unary.expr,
            Expr::Paren(paren) => &paren.expr,
            Expr::Group(group) => &group.expr,
            _ => return expr,
        };
    }
}

/// Returns a field's name, or a tuple field's index.
pub(crate) fn member_name(member: &Member) -> String {
    match member {
        Member::Named(ident) => strip_raw(&ident.to_string()),
        Member::Unnamed(index) => index.index.to_string(),
    }
}

/// Tells whether an expression is `self`.
pub(crate) fn is_self(expr: &Expr) -> bool {
    matches!(expr, Expr::Path(path) if path.qself.is_none() && path.path.is_ident("self"))
}

/// Returns a literal argument as written, a negated number included.
pub(crate) fn literal(expr: &Expr, file: &SourceFile) -> Option<String> {
    let number = |e: &Expr| matches!(e, Expr::Lit(lit) if matches!(lit.lit, syn::Lit::Int(_) | syn::Lit::Float(_)));
    let is_literal = match expr {
        Expr::Lit(_) => true,
        Expr::Unary(unary) => matches!(unary.op, UnOp::Neg(_)) && number(&unary.expr),
        _ => false,
    };
    is_literal.then(|| file.text[range_of(expr, file)].to_owned())
}

/// Returns the name an argument passes on unchanged: a bare name, or one
/// behind `&` or `&mut`.
pub(crate) fn passed(arg: &Expr) -> Option<String> {
    let arg = match arg {
        Expr::Reference(reference) => &*reference.expr,
        other => other,
    };
    let Expr::Path(path) = arg else {
        return None;
    };
    let ident = path.path.get_ident().filter(|_| path.qself.is_none())?;
    Some(strip_raw(&ident.to_string()))
}

/// Tells whether an operator assigns, such as `+=`.
pub(crate) fn is_assign(op: &BinOp) -> bool {
    matches!(
        op,
        BinOp::AddAssign(_)
            | BinOp::SubAssign(_)
            | BinOp::MulAssign(_)
            | BinOp::DivAssign(_)
            | BinOp::RemAssign(_)
            | BinOp::BitXorAssign(_)
            | BinOp::BitAndAssign(_)
            | BinOp::BitOrAssign(_)
            | BinOp::ShlAssign(_)
            | BinOp::ShrAssign(_)
    )
}
