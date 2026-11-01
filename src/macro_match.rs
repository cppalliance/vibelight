//! Matches an invocation's tokens against one rule's matcher: literal
//! tokens and groups match their own kind, fragments parse with syn, and
//! repetitions are searched by backtracking until a match consumes all of
//! the input. Every attempted step spends from the budget.

use std::collections::BTreeMap;
use std::rc::Rc;

use proc_macro2::{Delimiter, TokenStream, TokenTree};

use crate::macro_fragment::{Kind, fragment_len};
use crate::macro_rules::{Budget, Failure, Op, alone, normalized, repetition};

/// One element of a matcher.
pub(crate) enum Matcher {
    /// An identifier, punctuation, or literal that the input must repeat.
    Token(TokenTree),
    /// A group whose contents the inner matcher must match whole.
    Group(Delimiter, Vec<Matcher>),
    /// A fragment `$name:kind`.
    Fragment(String, Kind),
    /// A repetition `$( ... ) sep? op`.
    Repeat(Repeat),
}

/// A repetition in a matcher.
pub(crate) struct Repeat {
    body: Vec<Matcher>,
    /// The separator's tokens, empty when there is none.
    sep: Vec<TokenTree>,
    op: Op,
    /// Every metavariable the body binds, at any depth.
    names: Vec<String>,
}

/// The token trees one fragment consumed, and its kind.
pub(crate) struct Fragment {
    pub(crate) tokens: Vec<TokenTree>,
    pub(crate) kind: Kind,
}

/// What a metavariable captured: one fragment, or one capture for each
/// iteration of the outermost repetition it is inside.
pub(crate) enum Capture {
    One(Fragment),
    Many(Vec<Capture>),
}

/// The captures of a match, by metavariable name.
pub(crate) type Bindings = BTreeMap<String, Capture>;

/// Parses a matcher, or returns why the engine does not handle it.
pub(crate) fn parse(stream: &TokenStream) -> Result<Vec<Matcher>, String> {
    let mut names = Vec::new();
    let matcher = level(stream, &mut names)?;
    let count = names.len();
    names.sort();
    names.dedup();
    if names.len() == count {
        Ok(matcher)
    } else {
        Err("a metavariable bound twice".to_owned())
    }
}

/// Parses one level of a matcher, adding each name it binds to `names`.
fn level(stream: &TokenStream, names: &mut Vec<String>) -> Result<Vec<Matcher>, String> {
    let trees = normalized(stream.clone().into_iter().collect());
    let mut matcher = Vec::new();
    let mut at = 0;
    while let Some(tree) = trees.get(at) {
        at += 1;
        matcher.push(match tree {
            TokenTree::Group(group) => {
                Matcher::Group(group.delimiter(), level(&group.stream(), names)?)
            }
            TokenTree::Punct(dollar) if dollar.as_char() == '$' && at < trees.len() => {
                let (element, used) = metavariable(&trees[at..], names)?;
                at += used;
                element
            }
            token => Matcher::Token(token.clone()),
        });
    }
    Ok(matcher)
}

/// Parses what follows a `$` in a matcher, a fragment or a repetition, and
/// returns it with how many trees it took.
fn metavariable(rest: &[TokenTree], names: &mut Vec<String>) -> Result<(Matcher, usize), String> {
    match rest {
        [
            TokenTree::Ident(name),
            TokenTree::Punct(colon),
            TokenTree::Ident(kind),
            ..,
        ] if colon.as_char() == ':' && name != "crate" => {
            let kind = Kind::named(&kind.to_string())
                .ok_or_else(|| format!("the fragment kind `{kind}`"))?;
            names.push(name.to_string());
            Ok((Matcher::Fragment(name.to_string(), kind), 3))
        }
        [TokenTree::Group(group), after @ ..] if group.delimiter() == Delimiter::Parenthesis => {
            let first = names.len();
            let body = level(&group.stream(), names)?;
            let (sep, op, used) = repetition(after)?;
            let names = names[first..].to_vec();
            Ok((
                Matcher::Repeat(Repeat {
                    body,
                    sep,
                    op,
                    names,
                }),
                1 + used,
            ))
        }
        _ => Err("a `$` that starts neither a fragment nor a repetition".to_owned()),
    }
}

/// Matches `tokens` against `matcher` and returns the captures of the first
/// match that consumes them all, or `None` when there is none.
pub(crate) fn matches(
    matcher: &[Matcher],
    tokens: &[TokenTree],
    budget: &mut Budget,
) -> Result<Option<Bindings>, Failure> {
    let mut search = Search {
        log: Vec::new(),
        budget,
    };
    let found = search.level(matcher, &normalized(tokens.to_vec()))?;
    Ok(found.then(|| bindings(search.log)))
}

/// A backtracking search for one match, with the log of the path it is on.
struct Search<'m, 'b> {
    log: Vec<Entry<'m>>,
    budget: &'b mut Budget,
}

/// One event on the path of a match. Replaying a match's log in order
/// rebuilds its captures.
enum Entry<'m> {
    /// A fragment bound a metavariable.
    Bind(&'m str, Fragment),
    /// A repetition began.
    Open(&'m Repeat),
    /// An iteration of the innermost open repetition began.
    Next,
    /// The innermost open repetition ended.
    Close,
}

/// Where a match stands at one level: the elements left in the sequence it
/// is in, its position in the tokens, and the iteration it is inside.
struct State<'m> {
    elems: &'m [Matcher],
    pos: usize,
    frame: Option<Rc<At<'m>>>,
}

/// A repetition between iterations: `count` iterations are done, the next
/// tokens start at `pos`, `rest` follows the repetition, and `outer` is the
/// iteration the repetition is inside. The frame of an iteration in progress
/// is the point where that iteration began.
#[derive(Clone)]
struct At<'m> {
    rep: &'m Repeat,
    count: usize,
    pos: usize,
    rest: &'m [Matcher],
    outer: Option<Rc<At<'m>>>,
}

/// A point to come back to when the path ahead fails: ending the
/// repetition at `at`, with the log cut back to `log` entries.
struct Choice<'m> {
    log: usize,
    at: At<'m>,
}

impl<'m> Search<'m, '_> {
    /// Tells whether `matcher` matches all of `tokens`, leaving the entries
    /// of the first such match on the log.
    fn level(&mut self, matcher: &'m [Matcher], tokens: &[TokenTree]) -> Result<bool, Failure> {
        let mut choices = Vec::new();
        let mut state = State {
            elems: matcher,
            pos: 0,
            frame: None,
        };
        while !self.run(&mut state, &mut choices, tokens)? {
            let Some(choice) = choices.pop() else {
                return Ok(false);
            };
            self.log.truncate(choice.log);
            state = self.stop(choice.at);
        }
        Ok(true)
    }

    /// Advances `state` until its path matches all of `tokens` or fails,
    /// pushing a choice at each repetition that could also end where the
    /// path takes one more iteration. A group matches when the inner matcher
    /// matches its contents whole, and its first such match stands, since
    /// whether the rest matches depends only on where the group ends.
    fn run(
        &mut self,
        state: &mut State<'m>,
        choices: &mut Vec<Choice<'m>>,
        tokens: &[TokenTree],
    ) -> Result<bool, Failure> {
        loop {
            self.budget.step()?;
            let Some((first, rest)) = state.elems.split_first() else {
                let Some(frame) = state.frame.take() else {
                    return Ok(state.pos == tokens.len());
                };
                let progressed = state.pos > frame.pos;
                let at = At {
                    count: frame.count + 1,
                    pos: state.pos,
                    ..At::clone(&frame)
                };
                if !self.decide(state, choices, tokens, at, progressed)? {
                    return Ok(false);
                }
                continue;
            };
            let taken = match first {
                Matcher::Token(want) => tokens
                    .get(state.pos)
                    .is_some_and(|got| same(want, got))
                    .then_some(1),
                Matcher::Group(delimiter, inner) => match tokens.get(state.pos) {
                    Some(TokenTree::Group(group)) if group.delimiter() == *delimiter => {
                        let inside = normalized(group.stream().into_iter().collect());
                        self.budget.steps(inside.len())?;
                        self.level(inner, &inside)?.then_some(1)
                    }
                    _ => None,
                },
                Matcher::Fragment(name, kind) => {
                    self.fragment(name, *kind, tokens.get(state.pos..).unwrap_or_default())?
                }
                Matcher::Repeat(rep) => {
                    let outer = state.frame.take();
                    let at = At {
                        rep,
                        count: 0,
                        pos: state.pos,
                        rest,
                        outer,
                    };
                    if !self.decide(state, choices, tokens, at, true)? {
                        return Ok(false);
                    }
                    continue;
                }
            };
            let Some(taken) = taken else {
                return Ok(false);
            };
            state.elems = rest;
            state.pos += taken;
        }
    }

    /// Goes on from a repetition between iterations: into one more iteration
    /// when the operator allows it and any separator is there, keeping the
    /// end of the repetition as a choice, or else past the end. Returns false
    /// when neither is possible. An iteration that took no tokens is the
    /// last unless a separator follows it, so a body that matches nothing
    /// cannot loop.
    fn decide(
        &mut self,
        state: &mut State<'m>,
        choices: &mut Vec<Choice<'m>>,
        tokens: &[TokenTree],
        at: At<'m>,
        progressed: bool,
    ) -> Result<bool, Failure> {
        let may_end = at.count > 0 || at.rep.op != Op::Many;
        let moves = progressed || !at.rep.sep.is_empty();
        let may_go_on = moves && (at.count == 0 || at.rep.op != Op::Maybe);
        let start = if may_go_on {
            self.separator(&at, tokens)?
        } else {
            None
        };
        let Some(start) = start else {
            if may_end {
                *state = self.stop(at);
            }
            return Ok(may_end);
        };
        if may_end {
            choices.push(Choice {
                log: self.log.len(),
                at: at.clone(),
            });
        }
        let rep = at.rep;
        if at.count == 0 {
            self.log.push(Entry::Open(rep));
        }
        self.log.push(Entry::Next);
        *state = State {
            elems: &rep.body,
            pos: start,
            frame: Some(Rc::new(at)),
        };
        Ok(true)
    }

    /// Ends the repetition at `at` and returns where the match goes on.
    fn stop(&mut self, at: At<'m>) -> State<'m> {
        if at.count == 0 {
            self.log.push(Entry::Open(at.rep));
        }
        self.log.push(Entry::Close);
        State {
            elems: at.rest,
            pos: at.pos,
            frame: at.outer,
        }
    }

    /// Returns where one more iteration's body starts: past the separator
    /// when one is due, or `None` when it is missing.
    fn separator(&mut self, at: &At<'m>, tokens: &[TokenTree]) -> Result<Option<usize>, Failure> {
        let sep = &at.rep.sep;
        if at.count == 0 || sep.is_empty() {
            return Ok(Some(at.pos));
        }
        self.budget.step()?;
        let end = at.pos + sep.len();
        let found = tokens
            .get(at.pos..end)
            .is_some_and(|got| sep.iter().zip(got).all(|(want, got)| same(want, got)));
        Ok(found.then_some(end))
    }

    /// Binds `name` to the fragment of `kind` at the start of `tokens` and
    /// returns how many trees it took, or `None` when none starts there.
    /// The capture's last punctuation turns alone, since what it joined
    /// stays behind.
    fn fragment(
        &mut self,
        name: &'m str,
        kind: Kind,
        tokens: &[TokenTree],
    ) -> Result<Option<usize>, Failure> {
        let Some(len) = fragment_len(kind, tokens, self.budget)? else {
            return Ok(None);
        };
        let mut taken = tokens[..len].to_vec();
        if let Some(TokenTree::Punct(last)) = taken.last_mut() {
            *last = alone(last);
        }
        self.log.push(Entry::Bind(
            name,
            Fragment {
                tokens: taken,
                kind,
            },
        ));
        Ok(Some(len))
    }
}

/// Tells whether the input token `got` is the matcher's literal token
/// `want`: the same identifier, literal text, or punctuation with the same
/// spacing. Both sides are normalized, so a punctuation is joint exactly
/// when it is not the last of a token rustc lexes as one, and a lone `:`
/// never matches half of a `::`.
fn same(want: &TokenTree, got: &TokenTree) -> bool {
    match (want, got) {
        (TokenTree::Ident(want), TokenTree::Ident(got)) => want == got,
        (TokenTree::Literal(want), TokenTree::Literal(got)) => want.to_string() == got.to_string(),
        (TokenTree::Punct(want), TokenTree::Punct(got)) => {
            want.as_char() == got.as_char() && want.spacing() == got.spacing()
        }
        _ => false,
    }
}

/// A repetition being replayed: the captures of its finished iterations by
/// name, and the captures of the iteration in progress.
struct Open {
    done: BTreeMap<String, Vec<Capture>>,
    current: Option<Bindings>,
}

impl Open {
    /// Moves the captures of the iteration in progress, if any, into `done`.
    fn finish(&mut self) {
        let Some(mut current) = self.current.take() else {
            return;
        };
        for (name, captures) in &mut self.done {
            if let Some(capture) = current.remove(name) {
                captures.push(capture);
            }
        }
    }
}

/// Replays a match's log into its captures. A repetition's metavariables
/// get one capture per iteration, none when it had no iterations.
fn bindings(log: Vec<Entry<'_>>) -> Bindings {
    let mut top = Bindings::new();
    let mut open: Vec<Open> = Vec::new();
    for entry in log {
        match entry {
            Entry::Bind(name, fragment) => {
                scope(&mut top, &mut open).insert(name.to_owned(), Capture::One(fragment));
            }
            Entry::Open(rep) => open.push(Open {
                done: rep
                    .names
                    .iter()
                    .map(|name| (name.clone(), Vec::new()))
                    .collect(),
                current: None,
            }),
            Entry::Next => {
                if let Some(innermost) = open.last_mut() {
                    innermost.finish();
                    innermost.current = Some(Bindings::new());
                }
            }
            Entry::Close => {
                if let Some(mut ended) = open.pop() {
                    ended.finish();
                    let scope = scope(&mut top, &mut open);
                    for (name, captures) in ended.done {
                        scope.insert(name, Capture::Many(captures));
                    }
                }
            }
        }
    }
    top
}

/// Returns where a capture goes: the innermost iteration in progress, or
/// the top level outside every repetition.
fn scope<'s>(top: &'s mut Bindings, open: &'s mut [Open]) -> &'s mut Bindings {
    match open
        .last_mut()
        .and_then(|innermost| innermost.current.as_mut())
    {
        Some(current) => current,
        None => top,
    }
}
