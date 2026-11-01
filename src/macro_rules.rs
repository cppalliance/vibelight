//! Expands `macro_rules!` invocations into Rust items the way rustc does,
//! hygiene aside: a definition parses into rules, an invocation takes the
//! first rule whose matcher matches all of its tokens, and the
//! item-position invocations an expansion holds expand in place. The token
//! syntax that rule parsing and matching share lives here too: which runs
//! of punctuation are one token, and repetition operators.

use std::fmt;

use proc_macro2::{Punct, Spacing, TokenStream, TokenTree};
use syn::{Item, Path};

use crate::macro_match::{self, Matcher};
use crate::macro_transcribe::{self, Piece};

/// rustc's default recursion limit: how deep expansions may nest, with the
/// top-level expansion at depth 1.
const DEPTH_LIMIT: usize = 128;

/// The matcher steps one top-level expansion may take.
const STEP_LIMIT: usize = 1_000_000;

/// The output tokens one top-level expansion may write.
const TOKEN_LIMIT: usize = 200_000;

/// The runs of punctuation that rustc lexes as one token. Each entry less
/// its last character is an entry too, or a single character.
const COMPOUND: [&str; 24] = [
    "!=", "%=", "&&", "&=", "*=", "+=", "-=", "->", "..", "...", "..=", "/=", "::", "<<", "<<=",
    "<=", "==", "=>", ">=", ">>", ">>=", "^=", "|=", "||",
];

/// One parsed `macro_rules!` definition.
pub(crate) struct MacroRules {
    rules: Vec<Rule>,
}

/// One rule: its matcher and its transcriber, each parsed, or the reason the
/// engine does not handle it, which fails an expansion only when the
/// expansion reaches it.
struct Rule {
    matcher: Result<Vec<Matcher>, String>,
    transcriber: Result<Vec<Piece>, String>,
}

/// Why an expansion stopped.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Failure {
    /// A construct the engine does not handle, with a short reason.
    Unsupported(String),
    /// No rule's matcher matches all of the input.
    NoRuleMatched,
    /// The budget's matcher steps ran out.
    StepLimit,
    /// Expansions nested deeper than rustc's default recursion limit.
    DepthLimit,
    /// The budget's output tokens ran out.
    TooLarge,
    /// The transcribed tokens are not Rust items, with syn's message.
    NotItems(String),
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported(reason) => write!(f, "unsupported: {reason}"),
            Self::NoRuleMatched => f.write_str("no rule matched"),
            Self::StepLimit => f.write_str("the step limit was reached"),
            Self::DepthLimit => f.write_str("the depth limit was reached"),
            Self::TooLarge => f.write_str("the expansion is too large"),
            Self::NotItems(message) => write!(f, "the expansion is not items: {message}"),
        }
    }
}

impl MacroRules {
    /// Parses the tokens inside a `macro_rules! name { ... }` definition's
    /// braces: rules `(matcher) => {transcriber}` separated by `;`, with an
    /// optional trailing `;`. Each side may use any delimiter. A rule that
    /// holds a construct the engine does not handle still parses, and only
    /// an expansion that reaches it fails, with `Failure::Unsupported`.
    pub(crate) fn parse(body: &TokenStream) -> Result<Self, Failure> {
        let trees: Vec<TokenTree> = body.clone().into_iter().collect();
        let mut rest = trees.as_slice();
        let mut rules = Vec::new();
        while !rest.is_empty() {
            let [
                TokenTree::Group(matcher),
                TokenTree::Punct(eq),
                TokenTree::Punct(gt),
                TokenTree::Group(transcriber),
                after @ ..,
            ] = rest
            else {
                return Err(unsupported(
                    "a rule that is not `(matcher) => {transcriber}`",
                ));
            };
            if eq.as_char() != '=' || gt.as_char() != '>' {
                return Err(unsupported("a rule whose sides `=>` does not separate"));
            }
            rules.push(Rule {
                matcher: macro_match::parse(&matcher.stream()),
                transcriber: macro_transcribe::parse(&transcriber.stream()),
            });
            rest = match after {
                [TokenTree::Punct(semi), next @ ..] if semi.as_char() == ';' => next,
                [] => after,
                _ => return Err(unsupported("rules that `;` does not separate")),
            };
        }
        Ok(Self { rules })
    }

    /// Transcribes `input`, the tokens inside an invocation's delimiters,
    /// with the first rule whose matcher matches all of it.
    pub(crate) fn apply(
        &self,
        input: &TokenStream,
        budget: &mut Budget,
    ) -> Result<TokenStream, Failure> {
        let tokens: Vec<TokenTree> = input.clone().into_iter().collect();
        for rule in &self.rules {
            let matcher = rule
                .matcher
                .as_ref()
                .map_err(|reason| unsupported(reason))?;
            if let Some(bindings) = macro_match::matches(matcher, &tokens, budget)? {
                let pieces = rule
                    .transcriber
                    .as_ref()
                    .map_err(|reason| unsupported(reason))?;
                return macro_transcribe::transcribe(pieces, &bindings, budget);
            }
        }
        Err(Failure::NoRuleMatched)
    }
}

/// Shared limits for one top-level expansion: matcher steps and output tokens.
pub(crate) struct Budget {
    steps: usize,
    tokens: usize,
}

impl Budget {
    /// Returns a budget of 1,000,000 matcher steps and 200,000 output tokens.
    pub(crate) fn new() -> Self {
        Self {
            steps: STEP_LIMIT,
            tokens: TOKEN_LIMIT,
        }
    }

    /// Spends one matcher step, or fails with `Failure::StepLimit` when none
    /// is left.
    pub(crate) fn step(&mut self) -> Result<(), Failure> {
        self.steps(1)
    }

    /// Spends `count` matcher steps, one for each token a parse or a copy
    /// reads, or fails with `Failure::StepLimit` when fewer are left.
    pub(crate) fn steps(&mut self, count: usize) -> Result<(), Failure> {
        self.steps = self.steps.checked_sub(count).ok_or(Failure::StepLimit)?;
        Ok(())
    }

    /// Spends `count` output tokens, or fails with `Failure::TooLarge` when
    /// fewer are left.
    pub(crate) fn emit(&mut self, count: usize) -> Result<(), Failure> {
        self.tokens = self.tokens.checked_sub(count).ok_or(Failure::TooLarge)?;
        Ok(())
    }
}

/// Expands an item-position invocation into items: transcribes `input` with
/// `rules`, parses the result as items, and expands each nested
/// item-position invocation whose path `lookup` resolves, in place and depth
/// first, up to depth 128 (rustc's default recursion limit).
pub(crate) fn expand_items<'r>(
    rules: &'r MacroRules,
    input: &TokenStream,
    lookup: &dyn Fn(&Path) -> Option<&'r MacroRules>,
) -> Result<Vec<Item>, Failure> {
    expand(rules, input, lookup, &mut Budget::new(), 1)
}

/// Expands one invocation at `depth` for `expand_items`, spending from the
/// budget of the whole top-level call.
fn expand<'r>(
    rules: &'r MacroRules,
    input: &TokenStream,
    lookup: &dyn Fn(&Path) -> Option<&'r MacroRules>,
    budget: &mut Budget,
    depth: usize,
) -> Result<Vec<Item>, Failure> {
    if depth > DEPTH_LIMIT {
        return Err(Failure::DepthLimit);
    }
    let tokens = rules.apply(input, budget)?;
    let file =
        syn::parse2::<syn::File>(tokens).map_err(|error| Failure::NotItems(error.to_string()))?;
    let mut items = Vec::new();
    for item in file.items {
        match invocation(&item, lookup) {
            Some((nested, tokens)) => {
                items.extend(expand(nested, tokens, lookup, budget, depth + 1)?);
            }
            None => items.push(item),
        }
    }
    Ok(items)
}

/// Returns the definition `lookup` resolves for an item-position invocation,
/// with the invocation's tokens, or `None` for any other item, a
/// `macro_rules!` definition included.
fn invocation<'i, 'r>(
    item: &'i Item,
    lookup: &dyn Fn(&Path) -> Option<&'r MacroRules>,
) -> Option<(&'r MacroRules, &'i TokenStream)> {
    let Item::Macro(item) = item else {
        return None;
    };
    if item.mac.path.is_ident("macro_rules") {
        return None;
    }
    Some((lookup(&item.mac.path)?, &item.mac.tokens))
}

/// Returns `Failure::Unsupported` with `reason`.
fn unsupported(reason: &str) -> Failure {
    Failure::Unsupported(reason.to_owned())
}

/// A repetition operator.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Op {
    /// `*`: any number of iterations.
    Any,
    /// `+`: at least one iteration.
    Many,
    /// `?`: at most one iteration.
    Maybe,
}

/// Parses what follows the parentheses of a repetition, `sep? op`, and
/// returns the separator's tokens, empty when there is none, the operator,
/// and how many trees they took. The separator is one token as rustc lexes
/// it, so `=>` and `::` are separators too.
pub(crate) fn repetition(rest: &[TokenTree]) -> Result<(Vec<TokenTree>, Op, usize), String> {
    if let Some(op) = operator(rest) {
        return Ok((Vec::new(), op, 1));
    }
    let len = token_len(rest);
    let missing = "a repetition without `*`, `+`, or `?`".to_owned();
    if len == 0 || matches!(rest.first(), Some(TokenTree::Group(_))) {
        return Err(missing);
    }
    match operator(&rest[len..]) {
        Some(Op::Maybe) => Err("a `?` repetition with a separator".to_owned()),
        Some(op) => Ok((rest[..len].to_vec(), op, len + 1)),
        None => Err(missing),
    }
}

/// Returns the repetition operator that is the whole first token of `tokens`.
fn operator(tokens: &[TokenTree]) -> Option<Op> {
    let Some(TokenTree::Punct(punct)) = tokens.first() else {
        return None;
    };
    if token_len(tokens) != 1 {
        return None;
    }
    match punct.as_char() {
        '*' => Some(Op::Any),
        '+' => Some(Op::Many),
        '?' => Some(Op::Maybe),
        _ => None,
    }
}

/// Returns how many trees at the start of `tokens` rustc lexes as one token:
/// two for a lifetime, the length of a run of joint punctuation that forms
/// one compound operator, or else one; none for no tokens.
pub(crate) fn token_len(tokens: &[TokenTree]) -> usize {
    let Some(TokenTree::Punct(first)) = tokens.first() else {
        return usize::from(!tokens.is_empty());
    };
    if first.as_char() == '\'' && matches!(tokens.get(1), Some(TokenTree::Ident(_))) {
        return 2;
    }
    let mut text = first.as_char().to_string();
    let mut len = 1;
    while let (Some(TokenTree::Punct(last)), Some(TokenTree::Punct(next))) =
        (tokens.get(len - 1), tokens.get(len))
    {
        text.push(next.as_char());
        if last.spacing() != Spacing::Joint || !COMPOUND.contains(&text.as_str()) {
            break;
        }
        len += 1;
    }
    len
}

/// Returns `tokens` with the spacing that only a neighbor gave made alone:
/// punctuation stays joint only inside a token rustc lexes as one, so that
/// the `,` of `,$x` matches a `,` with any spacing and writes no glue.
pub(crate) fn normalized(mut tokens: Vec<TokenTree>) -> Vec<TokenTree> {
    let mut at = 0;
    while at < tokens.len() {
        let len = token_len(&tokens[at..]);
        if let Some(TokenTree::Punct(last)) = tokens.get_mut(at + len - 1) {
            *last = alone(last);
        }
        at += len;
    }
    tokens
}

/// Returns `punct` with alone spacing.
pub(crate) fn alone(punct: &Punct) -> Punct {
    let mut alone = Punct::new(punct.as_char(), Spacing::Alone);
    alone.set_span(punct.span());
    alone
}
