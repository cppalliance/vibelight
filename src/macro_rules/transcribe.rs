//! Transcribes a rule's transcriber with the captures of a match:
//! metavariables substitute their captures, `$crate` becomes `crate`, and
//! repetitions repeat once per capture of the metavariables inside them.
//! Every token written spends from the budget.

use std::collections::BTreeSet;

use proc_macro2::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::macro_rules::fragment::Kind;
use crate::macro_rules::matching::{Bindings, Capture, Fragment};
use crate::macro_rules::{Budget, Failure, normalized, repetition};

/// One element of a transcriber.
pub(crate) enum Piece {
    /// An identifier, punctuation, or literal written as it is.
    Token(TokenTree),
    /// A group, written with its delimiter and span around its contents.
    Group(Delimiter, Span, Vec<Piece>),
    /// A metavariable `$name`.
    Var(Ident),
    /// `$crate`, with the span of `crate`.
    Crate(Span),
    /// A repetition `$( ... ) sep? op`.
    Repeat(Repeat),
}

/// A repetition in a transcriber.
pub(crate) struct Repeat {
    body: Vec<Piece>,
    /// The separator's tokens, empty when there is none.
    sep: Vec<TokenTree>,
    /// Every metavariable named inside the body, at any depth.
    names: BTreeSet<String>,
}

/// Parses a transcriber, or returns why the engine does not handle it.
pub(crate) fn parse(stream: &TokenStream) -> Result<Vec<Piece>, String> {
    level(stream, &mut BTreeSet::new())
}

/// Parses one level of a transcriber, adding each metavariable it names to
/// `names`.
fn level(stream: &TokenStream, names: &mut BTreeSet<String>) -> Result<Vec<Piece>, String> {
    let trees = normalized(stream.clone().into_iter().collect());
    let mut pieces = Vec::new();
    let mut at = 0;
    while let Some(tree) = trees.get(at) {
        at += 1;
        pieces.push(match tree {
            TokenTree::Group(group) => Piece::Group(
                group.delimiter(),
                group.span(),
                level(&group.stream(), names)?,
            ),
            TokenTree::Punct(dollar) if dollar.as_char() == '$' && at < trees.len() => {
                let (piece, used) = metavariable(&trees[at..], names)?;
                at += used;
                piece
            }
            token => Piece::Token(token.clone()),
        });
    }
    Ok(pieces)
}

/// Parses what follows a `$` in a transcriber, `$crate`, a metavariable, or
/// a repetition, and returns it with how many trees it took.
fn metavariable(
    rest: &[TokenTree],
    names: &mut BTreeSet<String>,
) -> Result<(Piece, usize), String> {
    match rest {
        [TokenTree::Ident(ident), ..] if ident == "crate" => Ok((Piece::Crate(ident.span()), 1)),
        [TokenTree::Ident(ident), ..] => {
            names.insert(ident.to_string());
            Ok((Piece::Var(ident.clone()), 1))
        }
        [TokenTree::Group(group), after @ ..] if group.delimiter() == Delimiter::Parenthesis => {
            let mut inner = BTreeSet::new();
            let body = level(&group.stream(), &mut inner)?;
            let (sep, _, used) = repetition(after)?;
            names.extend(inner.iter().cloned());
            let names = inner;
            Ok((Piece::Repeat(Repeat { body, sep, names }), 1 + used))
        }
        [TokenTree::Group(group), ..] if group.delimiter() == Delimiter::Brace => {
            Err("a metavariable expression `${...}`".to_owned())
        }
        _ => Err("a `$` that starts neither a metavariable nor a repetition".to_owned()),
    }
}

/// Transcribes `pieces` with the captures of a match. A metavariable the
/// matcher does not bind is written as `$name`, as rustc does, which keeps
/// the metavariables of a `macro_rules!` definition the expansion holds.
pub(crate) fn transcribe(
    pieces: &[Piece],
    bindings: &Bindings,
    budget: &mut Budget,
) -> Result<TokenStream, Failure> {
    let mut out = Vec::new();
    let mut writer = Writer {
        bindings,
        budget,
        path: Vec::new(),
    };
    writer.write(pieces, &mut out)?;
    Ok(out.into_iter().collect())
}

/// A transcription in progress: the captures, the budget, and the index of
/// the current iteration of each repetition it is inside, outermost first.
struct Writer<'a> {
    bindings: &'a Bindings,
    budget: &'a mut Budget,
    path: Vec<usize>,
}

impl<'a> Writer<'a> {
    /// Writes `pieces` onto `out`.
    fn write(&mut self, pieces: &[Piece], out: &mut Vec<TokenTree>) -> Result<(), Failure> {
        for piece in pieces {
            match piece {
                Piece::Token(token) => self.push(token.clone(), out)?,
                Piece::Group(delimiter, span, inner) => {
                    let mut tokens = Vec::new();
                    self.write(inner, &mut tokens)?;
                    let mut group = Group::new(*delimiter, tokens.into_iter().collect());
                    group.set_span(*span);
                    self.push(group.into(), out)?;
                }
                Piece::Var(ident) => self.var(ident, out)?,
                Piece::Crate(span) => self.push(Ident::new("crate", *span).into(), out)?,
                Piece::Repeat(repeat) => self.repeat(repeat, out)?,
            }
        }
        Ok(())
    }

    /// Writes one token tree, spending one output token; the trees inside a
    /// group are spent as they are written.
    fn push(&mut self, tree: TokenTree, out: &mut Vec<TokenTree>) -> Result<(), Failure> {
        self.budget.emit(1)?;
        out.push(tree);
        Ok(())
    }

    /// Writes the capture of `$ident` at the current iterations, or `$ident`
    /// itself when the matcher binds no such name.
    fn var(&mut self, ident: &Ident, out: &mut Vec<TokenTree>) -> Result<(), Failure> {
        match self.capture(&ident.to_string())? {
            Some(Capture::One(fragment)) => self.fragment(fragment, out),
            Some(Capture::Many(_)) => Err(Failure::Unsupported(format!(
                "`${ident}` used at a shallower depth than it repeats"
            ))),
            None => {
                let mut dollar = Punct::new('$', Spacing::Alone);
                dollar.set_span(ident.span());
                self.push(dollar.into(), out)?;
                self.push(ident.clone().into(), out)
            }
        }
    }

    /// Writes a captured fragment: an expression inside an invisible group,
    /// which keeps its precedence, and any other fragment as its tokens.
    fn fragment(&mut self, fragment: &Fragment, out: &mut Vec<TokenTree>) -> Result<(), Failure> {
        spend(self.budget, &fragment.tokens)?;
        let tokens = fragment.tokens.iter().cloned();
        if matches!(fragment.kind, Kind::Expr | Kind::Expr2021) {
            self.push(Group::new(Delimiter::None, tokens.collect()).into(), out)
        } else {
            out.extend(tokens);
            Ok(())
        }
    }

    /// Writes a repetition once per element of the metavariables inside it
    /// that repeat deeper than the current iterations, with the separator
    /// between iterations.
    fn repeat(&mut self, repeat: &Repeat, out: &mut Vec<TokenTree>) -> Result<(), Failure> {
        let count = self.count(repeat)?;
        for index in 0..count {
            if index > 0 {
                for token in &repeat.sep {
                    self.push(token.clone(), out)?;
                }
            }
            self.path.push(index);
            self.write(&repeat.body, out)?;
            self.path.pop();
        }
        Ok(())
    }

    /// Returns how many times `repeat` repeats: the length that all the
    /// metavariables inside it that still repeat at the current iterations
    /// share. Unequal lengths, or no such metavariable, are unsupported.
    fn count(&self, repeat: &Repeat) -> Result<usize, Failure> {
        let mut found: Option<(&str, usize)> = None;
        for name in &repeat.names {
            let Some(Capture::Many(captures)) = self.capture(name)? else {
                continue;
            };
            match found {
                None => found = Some((name, captures.len())),
                Some((_, len)) if len == captures.len() => {}
                Some((first, len)) => {
                    return Err(Failure::Unsupported(format!(
                        "`${first}` repeats {len} times but `${name}` repeats {} times",
                        captures.len()
                    )));
                }
            }
        }
        found.map(|(_, len)| len).ok_or_else(|| {
            Failure::Unsupported("a repetition with no metavariable that repeats there".to_owned())
        })
    }

    /// Returns the capture of `name` at the current iterations: its capture
    /// indexed by the current iteration of each repetition it repeats in,
    /// outermost first, or `None` when the matcher binds no such name.
    fn capture(&self, name: &str) -> Result<Option<&'a Capture>, Failure> {
        let Some(mut capture) = self.bindings.get(name) else {
            return Ok(None);
        };
        for &index in &self.path {
            let Capture::Many(captures) = capture else {
                break;
            };
            capture = captures.get(index).ok_or_else(|| {
                Failure::Unsupported(format!("`${name}` repeats fewer times than its repetition"))
            })?;
        }
        Ok(Some(capture))
    }
}

/// Spends one output token from `budget` for each tree in `tokens`, the
/// trees inside groups included, so that a capture counts at its full size.
fn spend(budget: &mut Budget, tokens: &[TokenTree]) -> Result<(), Failure> {
    let mut pending: Vec<TokenStream> = Vec::new();
    for tree in tokens {
        budget.emit(1)?;
        if let TokenTree::Group(group) = tree {
            pending.push(group.stream());
        }
    }
    while let Some(stream) = pending.pop() {
        for tree in stream {
            budget.emit(1)?;
            if let TokenTree::Group(group) = &tree {
                pending.push(group.stream());
            }
        }
    }
    Ok(())
}
