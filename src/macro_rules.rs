//! Expands `macro_rules!` invocations into Rust items the way rustc does,
//! hygiene aside: a definition parses into rules, an invocation takes the
//! first rule whose matcher matches all of its tokens, and the
//! item-position invocations an expansion holds expand in place. The token
//! syntax that rule parsing and matching share lives here too: which runs
//! of punctuation are one token, repetition operators, and how many tokens
//! each fragment kind takes.

use std::fmt;

use proc_macro2::{Delimiter, Punct, Spacing, TokenStream, TokenTree};
use syn::buffer::Cursor;
use syn::parse::{Parse, ParseStream, Parser};
use syn::{Item, Path};

use crate::macro_match::{self, Kind, Matcher};
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
        self.steps = self.steps.checked_sub(1).ok_or(Failure::StepLimit)?;
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

/// Returns how many trees at the start of `tokens` a fragment of `kind`
/// takes, or `None` when none starts there. A kind syn parses gets a short
/// window first, so that a long input is not parsed again for every
/// fragment: one tree for `block` and two for `vis`, which hold any such
/// fragment, and for the others the trees up to where the fragment must end
/// if it gets that far, a result that stands only when it fills the window.
pub(crate) fn fragment_len(kind: Kind, tokens: &[TokenTree]) -> Option<usize> {
    let parse: fn(ParseStream<'_>) -> syn::Result<()> = match kind {
        Kind::Ident => {
            return matches!(tokens.first(), Some(TokenTree::Ident(ident)) if ident != "_")
                .then_some(1);
        }
        Kind::Lifetime => {
            return matches!(tokens, [TokenTree::Punct(quote), TokenTree::Ident(_), ..]
                if quote.as_char() == '\'')
            .then_some(2);
        }
        Kind::Literal => return literal_len(tokens),
        Kind::Tt => return Some(token_len(tokens)).filter(|&len| len > 0),
        Kind::Block => skip::<syn::Block>,
        Kind::Expr => skip::<syn::Expr>,
        Kind::Item => skip::<syn::Item>,
        Kind::Meta => skip::<syn::Meta>,
        Kind::Pat => |input| syn::Pat::parse_multi_with_leading_vert(input).map(drop),
        Kind::PatParam => |input| syn::Pat::parse_single(input).map(drop),
        Kind::Path => skip::<syn::Path>,
        Kind::Stmt => skip::<syn::Stmt>,
        Kind::Ty => skip::<syn::Type>,
        Kind::Vis => skip::<syn::Visibility>,
    };
    let (end, exact) = match kind {
        Kind::Block => (1, true),
        Kind::Vis => (2, true),
        Kind::Item => (
            past(tokens, |tree| is_punct(tree, ';') || is_brace(tree)),
            false,
        ),
        Kind::Stmt => (past(tokens, |tree| is_punct(tree, ';')), false),
        _ => (expr_end(tokens), false),
    };
    let end = end.min(tokens.len());
    let found = parsed_len(parse, &tokens[..end]);
    if exact || end == tokens.len() || found == Some(end) {
        found
    } else {
        parsed_len(parse, tokens)
    }
}

/// Returns how many trees a `literal` fragment takes from the start of
/// `tokens`: a literal, `true`, or `false`, after an optional `-`.
fn literal_len(tokens: &[TokenTree]) -> Option<usize> {
    let minus = usize::from(matches!(tokens.first(), Some(tree) if is_punct(tree, '-')));
    match tokens.get(minus)? {
        TokenTree::Literal(_) => Some(minus + 1),
        TokenTree::Ident(word) if word == "true" || word == "false" => Some(minus + 1),
        _ => None,
    }
}

/// Returns the index just past the first tree of `tokens` that `ends` an
/// item or statement reaching that far, or the length when none does. A
/// parse of the trees before that index stands only when it takes them all.
fn past(tokens: &[TokenTree], ends: impl Fn(&TokenTree) -> bool) -> usize {
    tokens
        .iter()
        .position(ends)
        .map_or(tokens.len(), |at| at + 1)
}

/// Returns the index of the first `,`, `;`, or `=>` in `tokens`, or the
/// length when there is none. An expression, type, pattern, path, or meta
/// holds one outside a group only inside a closure's `|...|` or a generic
/// `<...>`, which the trees before it leave open, so a parse of them that
/// takes them all is the fragment's whole parse.
fn expr_end(tokens: &[TokenTree]) -> usize {
    (0..tokens.len())
        .find(|&at| {
            let arrow = matches!(&tokens[at], TokenTree::Punct(eq)
                if eq.as_char() == '=' && eq.spacing() == Spacing::Joint)
                && tokens.get(at + 1).is_some_and(|next| is_punct(next, '>'));
            arrow || is_punct(&tokens[at], ',') || is_punct(&tokens[at], ';')
        })
        .unwrap_or(tokens.len())
}

/// Runs `parse` with syn at the start of `tokens` and returns how many trees
/// it took, or `None` when it fails or stops inside an invisible group.
fn parsed_len(
    parse: fn(ParseStream<'_>) -> syn::Result<()>,
    tokens: &[TokenTree],
) -> Option<usize> {
    let counted = |input: ParseStream<'_>| {
        let start = input.cursor();
        parse(input)?;
        let end = input.cursor();
        let mut cursor = start;
        let mut taken = 0;
        while cursor != end {
            let Some((_, next)) = cursor.token_tree() else {
                return Err(input.error("the fragment stops inside an invisible group"));
            };
            cursor = next;
            taken += 1;
        }
        input.step(|_| Ok(((), Cursor::empty())))?;
        Ok(taken)
    };
    counted.parse2(tokens.iter().cloned().collect()).ok()
}

/// Parses one `T` and drops it.
fn skip<T: Parse>(input: ParseStream<'_>) -> syn::Result<()> {
    input.parse::<T>().map(drop)
}

/// Tells whether `tree` is the punctuation `ch`.
fn is_punct(tree: &TokenTree, ch: char) -> bool {
    matches!(tree, TokenTree::Punct(punct) if punct.as_char() == ch)
}

/// Tells whether `tree` is a brace group.
fn is_brace(tree: &TokenTree) -> bool {
    matches!(tree, TokenTree::Group(group) if group.delimiter() == Delimiter::Brace)
}
