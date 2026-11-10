//! How many tokens a `macro_rules!` fragment takes from an invocation's
//! input: the fragment kinds, the tokens each kind reads by itself, and
//! the kinds that syn parses, each in a short window first so that a long
//! input is not parsed again for every fragment.

use proc_macro2::{Delimiter, Punct, Spacing, TokenTree};
use syn::Item;
use syn::buffer::Cursor;
use syn::parse::{Parse, ParseStream, Parser};

use crate::macro_rules::{Budget, Failure, token_len};

/// What a fragment `$name:kind` matches; `expr_2021` is `expr` without `_`
/// and `const` blocks.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Block,
    Expr,
    Expr2021,
    Ident,
    Item,
    Lifetime,
    Literal,
    Meta,
    Pat,
    PatParam,
    Path,
    Stmt,
    Tt,
    Ty,
    Vis,
}

impl Kind {
    /// Returns the kind a fragment specifier names.
    pub(crate) fn named(name: &str) -> Option<Self> {
        Some(match name {
            "block" => Self::Block,
            "expr" => Self::Expr,
            "expr_2021" => Self::Expr2021,
            "ident" => Self::Ident,
            "item" => Self::Item,
            "lifetime" => Self::Lifetime,
            "literal" => Self::Literal,
            "meta" => Self::Meta,
            "pat" => Self::Pat,
            "pat_param" => Self::PatParam,
            "path" => Self::Path,
            "stmt" => Self::Stmt,
            "tt" => Self::Tt,
            "ty" => Self::Ty,
            "vis" => Self::Vis,
            _ => return None,
        })
    }
}

/// Returns how many trees at the start of `tokens` a fragment of `kind`
/// takes, or `None` when none starts there, spending a step for each tree a
/// parse reads. A kind syn parses gets a short window first, so that a long
/// input is not parsed again for every fragment: one tree for `block` and
/// two for `vis`, which hold any such fragment, and for the others the
/// trees up to where the fragment must end if it gets that far, a result
/// that stands only when it fills the window. No type, path, or parameter
/// pattern holds a `|` outside a group, so their windows end there too.
pub(crate) fn fragment_len(
    kind: Kind,
    tokens: &[TokenTree],
    budget: &mut Budget,
) -> Result<Option<usize>, Failure> {
    let parse: fn(ParseStream<'_>) -> syn::Result<()> = match kind {
        Kind::Ident => {
            let found = matches!(tokens.first(), Some(TokenTree::Ident(ident)) if ident != "_");
            return Ok(found.then_some(1));
        }
        Kind::Lifetime => {
            return Ok(
                matches!(tokens, [TokenTree::Punct(quote), TokenTree::Ident(_), ..]
                if quote.as_char() == '\'')
                .then_some(2),
            );
        }
        Kind::Literal => return Ok(literal_len(tokens)),
        Kind::Tt => return Ok(Some(token_len(tokens)).filter(|&len| len > 0)),
        Kind::Stmt => return stmt_len(tokens, budget),
        Kind::Expr2021 if matches!(tokens.first(), Some(TokenTree::Ident(word)) if word == "_" || word == "const") =>
        {
            return Ok(None);
        }
        Kind::Block => skip::<syn::Block>,
        Kind::Expr | Kind::Expr2021 => skip::<syn::Expr>,
        Kind::Item => skip::<syn::Item>,
        Kind::Meta => skip::<syn::Meta>,
        Kind::Pat => |input| syn::Pat::parse_multi_with_leading_vert(input).map(drop),
        Kind::PatParam => |input| syn::Pat::parse_single(input).map(drop),
        Kind::Path => skip::<syn::Path>,
        Kind::Ty => skip::<syn::Type>,
        Kind::Vis => skip::<syn::Visibility>,
    };
    let bar = || (tokens.iter()).position(|tree| is_punct(tree, '|'));
    let (end, exact) = match kind {
        Kind::Block => (1, true),
        Kind::Vis => (2, true),
        Kind::Item => (
            past(tokens, |tree| is_punct(tree, ';') || is_brace(tree)),
            false,
        ),
        Kind::Ty | Kind::Path | Kind::PatParam => {
            (expr_end(tokens).min(bar().unwrap_or(tokens.len())), false)
        }
        _ => (expr_end(tokens), false),
    };
    let end = end.min(tokens.len());
    budget.steps(end)?;
    let found = parsed_len(parse, &tokens[..end]);
    if exact || end == tokens.len() || found == Some(end) {
        return Ok(found);
    }
    budget.steps(tokens.len())?;
    Ok(parsed_len(parse, tokens))
}

/// Returns how many trees a `stmt` fragment takes from the start of
/// `tokens`, as rustc reads one: a statement that ends where a `,`, `;`, or
/// `=>` may follow it, without a `;` it does not own, and an item with the
/// `;` its own syntax ends in, as `struct S;` has.
fn stmt_len(tokens: &[TokenTree], budget: &mut Budget) -> Result<Option<usize>, Failure> {
    let arrow = |at: usize| tokens.get(at + 1).is_some_and(|next| is_punct(next, '>'));
    let follows = |at: &usize| match &tokens[*at] {
        TokenTree::Punct(punct) if punct.as_char() == ',' || punct.as_char() == ';' => true,
        TokenTree::Punct(eq) => eq.as_char() == '=' && eq.spacing() == Spacing::Joint && arrow(*at),
        _ => false,
    };
    for end in (1..tokens.len()).filter(follows).chain([tokens.len()]) {
        if end > 0 && tokens.get(end).is_some_and(|tree| is_punct(tree, ';')) {
            budget.steps(end + 1)?;
            if parsed_len(skip::<Item>, &tokens[..=end]) == Some(end + 1) {
                return Ok(Some(end + 1));
            }
        }
        let mut window = tokens[..end].to_vec();
        window.push(Punct::new(';', Spacing::Alone).into());
        budget.steps(end + 1)?;
        let found = parsed_len(skip::<syn::Stmt>, &window);
        if end > 0 && (found == Some(end) || found == Some(end + 1)) {
            return Ok(Some(end));
        }
    }
    Ok(None)
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
