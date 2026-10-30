//! The keys each wire type of a target reads or writes, so a drift check
//! can search the docs for every key. A type is a wire type as
//! `Decl::is_wire` says. A field's or a variant's key is its `rename`, or its
//! name under the container's `rename_all`, and each `alias` is a key too; a
//! skipped or flattened field has no key of its own, and a container's `tag`
//! and `content` are keys.

use syn::punctuated::Punctuated;
use syn::{Meta, Token};

use crate::attrs::string_value;
use crate::model::{MemberKind, Shape, path_order};
use crate::render_slop::Outline;
use crate::render_slop_schema::{Record, WireKeysRecord};
use crate::typerefs::strip_raw;

/// What a node's `serde` attributes say about its key.
#[derive(Default)]
struct Serde {
    rename: Vec<String>,
    rename_all: Option<String>,
    aliases: Vec<String>,
    tags: Vec<String>,
    skip: bool,
    flatten: bool,
}

impl Serde {
    /// Reads the `serde` attributes among a node's API attributes, each as
    /// written, such as `serde(rename = "id")`.
    fn of(attrs: &[String]) -> Serde {
        let mut out = Serde::default();
        for attr in attrs.iter().filter(|a| a.starts_with("serde")) {
            let Ok(Meta::List(list)) = syn::parse_str::<Meta>(attr) else {
                continue;
            };
            let Ok(items) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            else {
                continue;
            };
            for item in items {
                out.read(&item);
            }
        }
        out
    }

    fn read(&mut self, item: &Meta) {
        let key = item
            .path()
            .get_ident()
            .map(ToString::to_string)
            .unwrap_or_default();
        let values: Vec<String> = match item {
            Meta::NameValue(nv) => string_value(&nv.value).into_iter().collect(),
            Meta::List(list) => list
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .map(|inner| {
                    inner
                        .iter()
                        .filter_map(|m| match m {
                            Meta::NameValue(nv) => string_value(&nv.value),
                            _ => None,
                        })
                        .collect()
                })
                .unwrap_or_default(),
            Meta::Path(_) => Vec::new(),
        };
        match key.as_str() {
            "rename" => self.rename.extend(values),
            "rename_all" => self.rename_all = values.into_iter().next(),
            "alias" => self.aliases.extend(values),
            "tag" | "content" => self.tags.extend(values),
            "skip" => self.skip = true,
            "flatten" => self.flatten = true,
            _ => {}
        }
    }

    /// Returns the keys of a field or variant named `name` under a
    /// container's `rename_all`.
    fn keys(&self, name: &str, rule: Option<&str>, variant: bool) -> Vec<String> {
        if self.skip || self.flatten {
            return Vec::new();
        }
        let name = strip_raw(name);
        let mut keys = if self.rename.is_empty() {
            vec![rule.map_or_else(|| name.clone(), |rule| renamed(&name, rule, variant))]
        } else {
            self.rename.clone()
        };
        keys.extend(self.aliases.iter().cloned());
        keys
    }
}

/// Splits a name into lowercase words: a field's at its underscores, and a
/// variant's at each capital.
fn words(name: &str, variant: bool) -> Vec<String> {
    if !variant {
        return name.split('_').map(str::to_lowercase).collect();
    }
    let mut out: Vec<String> = Vec::new();
    for c in name.chars() {
        if c.is_uppercase() || out.is_empty() {
            out.push(String::new());
        }
        if let Some(last) = out.last_mut() {
            last.extend(c.to_lowercase());
        }
    }
    out
}

/// Applies a `rename_all` rule the way `serde` does.
fn renamed(name: &str, rule: &str, variant: bool) -> String {
    let words = words(name, variant);
    let capital = |word: &String| {
        let mut chars = word.chars();
        chars.next().map_or_else(String::new, |first| {
            first.to_uppercase().chain(chars).collect()
        })
    };
    match rule {
        "lowercase" if variant => name.to_lowercase(),
        "UPPERCASE" => name.to_uppercase(),
        "PascalCase" => words.iter().map(capital).collect(),
        "camelCase" => {
            let pascal: String = words.iter().map(capital).collect();
            let mut chars = pascal.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_lowercase().chain(chars).collect()
            })
        }
        "snake_case" => words.join("_"),
        "SCREAMING_SNAKE_CASE" => words.join("_").to_uppercase(),
        "kebab-case" => words.join("-"),
        "SCREAMING-KEBAB-CASE" => words.join("-").to_uppercase(),
        _ => name.to_owned(),
    }
}

/// Returns a record for each wire type of the target that has keys, in
/// path order, its keys sorted.
pub(crate) fn wire_keys(outline: &Outline<'_>) -> Vec<Record> {
    let krate = outline.krate();
    let mut found: Vec<(Vec<String>, Record)> = Vec::new();
    for (d, decl) in krate.decls.iter().enumerate().filter(|(_, d)| d.is_wire()) {
        let container = Serde::of(&decl.meta.attrs);
        let rule = container.rename_all.as_deref();
        let mut keys = container.tags.clone();
        match &decl.shape {
            Shape::Named => {
                for member in &decl.members {
                    if let MemberKind::Field(_) = member.kind {
                        keys.extend(Serde::of(&member.meta.attrs).keys(&member.name, rule, false));
                    }
                }
            }
            Shape::Enum(variants) => {
                for variant in variants {
                    keys.extend(Serde::of(&variant.meta.attrs).keys(&variant.name, rule, true));
                }
            }
            Shape::Tuple(_) | Shape::Unit | Shape::Trait { .. } | Shape::Alias(_) => {}
        }
        keys.sort();
        keys.dedup();
        if keys.is_empty() {
            continue;
        }
        let path = krate.decl_path(d);
        let record = Record::WireKeys(WireKeysRecord {
            path: path.join("::"),
            keys,
            loc: outline.own(decl.loc),
        });
        found.push((path, record));
    }
    found.sort_by(|a, b| path_order(&a.0, &b.0));
    found.into_iter().map(|(_, record)| record).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rename_all_rules_spell_fields_and_variants_as_serde_does() {
        assert_eq!(renamed("total_tokens", "camelCase", false), "totalTokens");
        assert_eq!(renamed("total_tokens", "kebab-case", false), "total-tokens");
        assert_eq!(renamed("total_tokens", "PascalCase", false), "TotalTokens");
        assert_eq!(renamed("total_tokens", "lowercase", false), "total_tokens");
        assert_eq!(renamed("ReadOnly", "snake_case", true), "read_only");
        assert_eq!(renamed("ReadOnly", "lowercase", true), "readonly");
        assert_eq!(
            renamed("ReadOnly", "SCREAMING-KEBAB-CASE", true),
            "READ-ONLY"
        );
        assert_eq!(renamed("ReadOnly", "camelCase", true), "readOnly");
    }

    #[test]
    fn a_rename_wins_aliases_add_and_skipped_fields_have_none() {
        let attrs = |text: &str| vec![text.to_owned()];
        let renamed = Serde::of(&attrs("serde(rename = \"id\", alias = \"key\")"));
        assert_eq!(
            renamed.keys("name", Some("camelCase"), false),
            ["id", "key"]
        );
        let none: [&str; 0] = [];
        assert_eq!(
            Serde::of(&attrs("serde(skip)")).keys("x", None, false),
            none
        );
        assert_eq!(
            Serde::of(&attrs("serde(flatten)")).keys("x", None, false),
            none
        );
        let both = Serde::of(&attrs(
            "serde(rename(serialize = \"a\", deserialize = \"b\"))",
        ));
        assert_eq!(both.keys("x", None, false), ["a", "b"]);
    }
}
