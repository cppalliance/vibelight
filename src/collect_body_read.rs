//! Reads a body's syntax for analysis: the paths it calls, reads,
//! constructs, and matches, its statements and loops, the chains of methods
//! on `self` fields and parameters, and how often it names each parameter.
//! Macro arguments are tokens, which the token scan reads instead.

use std::collections::BTreeSet;

use syn::visit::{self, Visit};
use syn::{Expr, ExprCall, ExprPath, Macro, Pat, Stmt, UnOp};

use crate::collect_body_code::{
    Called, Code, Read, Statement, is_assign, is_self, literal, member_name, passed, segments,
};
use crate::macros;
use crate::model_body::{OpChain, Role, Root};
use crate::source::SourceFile;
use crate::typerefs::{range_of, strip_raw};

/// The macros that always panic.
const PANICS: [&str; 4] = ["panic", "todo", "unimplemented", "unreachable"];

/// Where an expression sits in its parent, which gives a path there its
/// role.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Pos {
    /// Where a value may leave: an argument, an initializer, or a returned
    /// or stored value.
    Value,
    /// The callee of a call.
    Callee,
    /// A link inside a chain: a receiver, or the base of a field or index.
    Place,
    /// Read in place: an operand, a condition, or a subscript.
    Operand,
    /// Assigned to, or borrowed as `&mut`.
    Write,
}

struct Reader<'r> {
    file: &'r SourceFile,
    params: &'r [String],
    /// The names the body's patterns bind, which hide functions of the
    /// same name.
    locals: BTreeSet<String>,
    /// The position of the next expression visited.
    next: Pos,
    /// How many loops hold the expression being visited.
    in_loops: usize,
    /// How many statements hold the statement being visited.
    depth: usize,
    read: Read,
}

/// Reads a body's syntax. `params` are the names its function binds.
pub(crate) fn read(file: &SourceFile, code: Code<'_>, params: &[String]) -> Read {
    let mut reader = Reader {
        file,
        params,
        locals: BTreeSet::new(),
        next: Pos::Value,
        in_loops: 0,
        depth: 0,
        read: Read {
            param_uses: vec![0; params.len()],
            ..Read::default()
        },
    };
    if let Some(stmts) = code.stmts() {
        for stmt in stmts {
            reader.visit_stmt(stmt);
        }
    } else if let Code::Expr(expr) = code {
        reader.read.statements.push(Statement {
            range: range_of(expr, file),
            depth: 0,
        });
        reader.depth = 1;
        reader.visit_expr(expr);
    }
    reader
        .read
        .statements
        .sort_by_key(|s| (s.range.start, s.depth));
    reader.read
}

impl Reader<'_> {
    /// Visits an expression at a position.
    fn at(&mut self, expr: &Expr, pos: Pos) {
        self.next = pos;
        self.visit_expr(expr);
    }

    /// Starts the part of a loop that runs on every pass.
    fn enter_loop(&mut self) {
        self.in_loops += 1;
        self.read.loop_depth = self.read.loop_depth.max(self.in_loops);
    }

    /// Counts a parameter's use and passes over a local. Any other path
    /// without a qualified self type becomes a call fact of the role its
    /// position gives it.
    fn path_fact(&mut self, node: &ExprPath, pos: Pos) -> Option<Called> {
        if node.qself.is_some() {
            return None;
        }
        let (global, path) = segments(&node.path);
        match path.last().map(String::as_str) {
            Some("Err") => self.read.counts.errs += 1,
            Some("None") => self.read.counts.nones += 1,
            _ => {}
        }
        if let (false, [name]) = (global, path.as_slice()) {
            if let Some(at) = self.params.iter().position(|p| p == name) {
                self.read.param_uses[at] += 1;
                return None;
            }
            if name == "self" || self.locals.contains(name) {
                return None;
            }
        }
        let role = match pos {
            Pos::Callee => Role::Call,
            Pos::Value => Role::ValueEscape,
            Pos::Place | Pos::Operand | Pos::Write => Role::OtherReference,
        };
        Some(Called {
            global,
            path,
            role,
            args: 0,
            literal_args: Vec::new(),
            passed_params: Vec::new(),
        })
    }

    /// Reads a call: its callee, then its arguments as values.
    fn call(&mut self, node: &ExprCall) {
        let called = match &*node.func {
            Expr::Path(path) => self.path_fact(path, Pos::Callee),
            func => {
                self.at(func, Pos::Callee);
                None
            }
        };
        if let Some(mut called) = called {
            called.args = node.args.len();
            for (i, arg) in node.args.iter().enumerate() {
                if let Some(text) = literal(arg, self.file) {
                    called.literal_args.push((i, text));
                }
                if let Some(name) = passed(arg).filter(|name| self.params.contains(name)) {
                    called.passed_params.push((i, name));
                }
            }
            self.read.called.push(called);
        }
        for arg in &node.args {
            self.at(arg, Pos::Value);
        }
    }

    /// Records the chain an expression is when it starts at a field of
    /// `self` or at a parameter, and tells whether it is one.
    fn chain(&mut self, node: &Expr, write: bool) -> bool {
        let write = write || matches!(node, Expr::Reference(r) if r.mutability.is_some());
        // Each link from the outside in: a method and its first literal
        // argument, or `None` for a field access or an index.
        let mut links: Vec<Option<(String, Option<String>)>> = Vec::new();
        let mut at = node;
        let root = loop {
            at = match at {
                Expr::MethodCall(call) => {
                    let first = call.args.first().and_then(|a| literal(a, self.file));
                    links.push(Some((strip_raw(&call.method.to_string()), first)));
                    &call.receiver
                }
                Expr::Field(field) if is_self(&field.base) => {
                    break Root::Field(member_name(&field.member));
                }
                Expr::Field(field) => {
                    links.push(None);
                    &field.base
                }
                Expr::Index(index) => {
                    links.push(None);
                    &index.expr
                }
                Expr::Paren(paren) => &paren.expr,
                Expr::Group(group) => &group.expr,
                Expr::Reference(reference) => &reference.expr,
                Expr::Try(try_) => &try_.expr,
                Expr::Await(await_) => &await_.base,
                Expr::Unary(unary) if matches!(unary.op, UnOp::Deref(_)) => &unary.expr,
                Expr::Path(_) => {
                    let param = passed(at).filter(|n| n != "self" && self.params.contains(n));
                    let Some(name) = param else {
                        return false;
                    };
                    break Root::Param(name);
                }
                _ => return false,
            };
        };
        let mut methods = Vec::new();
        let mut first_literal = None;
        for link in links.into_iter().rev() {
            let Some((name, literal)) = link else {
                break;
            };
            if methods.is_empty() {
                first_literal = literal;
            }
            methods.push(name);
        }
        self.read.chains.insert(OpChain {
            root,
            methods,
            first_literal,
            in_loop: self.in_loops > 0,
            write,
        });
        true
    }

    /// Reads an expression that passes its position on to the one inside
    /// it, and tells whether it was one. Inside a recorded chain, or inside
    /// a place, the inner expression is a place too, so no chain inside a
    /// chain is recorded again.
    fn link(&mut self, node: &Expr, pos: Pos, chained: bool) -> bool {
        let inner = |own: Pos| {
            if chained || pos == Pos::Place {
                Pos::Place
            } else {
                own
            }
        };
        match node {
            Expr::Path(path) => {
                if let Some(called) = self.path_fact(path, pos) {
                    self.read.called.push(called);
                }
            }
            Expr::Call(call) => self.call(call),
            Expr::MethodCall(call) => {
                self.at(&call.receiver, Pos::Place);
                for arg in &call.args {
                    self.at(arg, Pos::Value);
                }
            }
            Expr::Field(field) => self.at(&field.base, Pos::Place),
            Expr::Index(index) => {
                self.at(&index.expr, Pos::Place);
                self.at(&index.index, Pos::Operand);
            }
            Expr::Paren(paren) => self.at(&paren.expr, inner(pos)),
            Expr::Group(group) => self.at(&group.expr, inner(pos)),
            Expr::Reference(reference) => {
                let own = if reference.mutability.is_some() {
                    Pos::Write
                } else {
                    pos
                };
                self.at(&reference.expr, inner(own));
            }
            Expr::Unary(unary) => {
                let own = if matches!(unary.op, UnOp::Deref(_)) {
                    inner(pos)
                } else {
                    Pos::Operand
                };
                self.at(&unary.expr, own);
            }
            Expr::Try(try_) => {
                self.read.counts.tries += 1;
                self.at(&try_.expr, inner(pos));
            }
            Expr::Await(await_) => {
                self.read.counts.awaits += 1;
                self.at(&await_.base, inner(pos));
            }
            _ => return false,
        }
        true
    }

    /// Reads an expression with parts in fixed positions: assignments,
    /// operators, conditions, matches, loops, closures, struct literals,
    /// and macros. Tells whether it was one.
    fn control(&mut self, node: &Expr) -> bool {
        match node {
            Expr::Assign(assign) => {
                self.at(&assign.left, Pos::Write);
                self.at(&assign.right, Pos::Value);
            }
            Expr::Binary(binary) => {
                let left = if is_assign(&binary.op) {
                    Pos::Write
                } else {
                    Pos::Operand
                };
                self.at(&binary.left, left);
                self.at(&binary.right, Pos::Operand);
            }
            Expr::Range(range) => {
                for end in range.start.iter().chain(&range.end) {
                    self.at(end, Pos::Operand);
                }
            }
            Expr::If(if_) => {
                self.at(&if_.cond, Pos::Operand);
                self.visit_block(&if_.then_branch);
                if let Some((_, other)) = &if_.else_branch {
                    self.at(other, Pos::Value);
                }
            }
            Expr::Match(match_) => {
                self.at(&match_.expr, Pos::Operand);
                for arm in &match_.arms {
                    self.visit_pat(&arm.pat);
                    self.at(&arm.body, Pos::Value);
                }
            }
            Expr::Let(let_) => {
                self.visit_pat(&let_.pat);
                self.at(&let_.expr, Pos::Operand);
            }
            Expr::ForLoop(for_) => {
                self.read.loops += 1;
                self.visit_pat(&for_.pat);
                self.at(&for_.expr, Pos::Operand);
                self.enter_loop();
                self.visit_block(&for_.body);
                self.in_loops -= 1;
            }
            Expr::While(while_) => {
                self.read.loops += 1;
                self.enter_loop();
                self.at(&while_.cond, Pos::Operand);
                self.visit_block(&while_.body);
                self.in_loops -= 1;
            }
            Expr::Loop(loop_) => {
                self.read.loops += 1;
                self.enter_loop();
                self.visit_block(&loop_.body);
                self.in_loops -= 1;
            }
            Expr::Closure(closure) => {
                for input in &closure.inputs {
                    self.visit_pat(input);
                }
                self.at(&closure.body, Pos::Value);
            }
            Expr::Struct(literal) => {
                if literal.qself.is_none() {
                    self.read.constructs.push(segments(&literal.path));
                }
                for field in &literal.fields {
                    self.at(&field.expr, Pos::Value);
                }
                if let Some(rest) = &literal.rest {
                    self.at(rest, Pos::Value);
                }
            }
            Expr::Macro(call) => self.visit_macro(&call.mac),
            _ => return false,
        }
        true
    }
}

impl<'ast> Visit<'ast> for Reader<'_> {
    fn visit_stmt(&mut self, node: &'ast Stmt) {
        self.read.statements.push(Statement {
            range: range_of(node, self.file),
            depth: self.depth,
        });
        self.depth += 1;
        match node {
            Stmt::Local(local) => {
                self.visit_pat(&local.pat);
                if let Some(init) = &local.init {
                    self.at(&init.expr, Pos::Value);
                    if let Some((_, diverge)) = &init.diverge {
                        self.at(diverge, Pos::Value);
                    }
                }
            }
            Stmt::Item(item) => self.visit_item(item),
            Stmt::Expr(expr, _) => self.at(expr, Pos::Value),
            Stmt::Macro(call) => self.visit_macro(&call.mac),
        }
        self.depth -= 1;
    }

    fn visit_expr(&mut self, node: &'ast Expr) {
        let pos = std::mem::replace(&mut self.next, Pos::Value);
        let chained = pos != Pos::Place && self.chain(node, pos == Pos::Write);
        if !self.link(node, pos, chained) && !self.control(node) {
            visit::visit_expr(self, node);
        }
    }

    /// Binds the names a pattern introduces, and records the paths it
    /// names. A bare name that starts with a capital, such as `None`, is
    /// read as a path rather than a binding.
    fn visit_pat(&mut self, node: &'ast Pat) {
        match node {
            Pat::Ident(ident) => {
                let name = strip_raw(&ident.ident.to_string());
                let bare = ident.by_ref.is_none() && ident.mutability.is_none();
                if bare && ident.subpat.is_none() && name.starts_with(char::is_uppercase) {
                    self.read.patterns.push((false, vec![name]));
                } else {
                    self.locals.insert(name);
                }
                if let Some((_, sub)) = &ident.subpat {
                    self.visit_pat(sub);
                }
            }
            Pat::Path(path) if path.qself.is_none() => {
                self.read.patterns.push(segments(&path.path));
            }
            Pat::TupleStruct(tuple) => {
                if tuple.qself.is_none() {
                    self.read.patterns.push(segments(&tuple.path));
                }
                for elem in &tuple.elems {
                    self.visit_pat(elem);
                }
            }
            Pat::Struct(fields) => {
                if fields.qself.is_none() {
                    self.read.patterns.push(segments(&fields.path));
                }
                for field in &fields.fields {
                    self.visit_pat(&field.pat);
                }
            }
            Pat::Guard(guard) => {
                self.visit_pat(&guard.pat);
                self.at(&guard.guard, Pos::Operand);
            }
            Pat::Macro(call) => self.visit_macro(&call.mac),
            _ => visit::visit_pat(self, node),
        }
    }

    /// Records the macro; its arguments are tokens, left to the token scan.
    fn visit_macro(&mut self, node: &'ast Macro) {
        let last = node.path.segments.last().map(|s| s.ident.to_string());
        if last.is_some_and(|name| PANICS.contains(&name.as_str())) {
            self.read.counts.panics += 1;
        }
        self.read.macros.insert(macros::path_text(&node.path));
    }
}
