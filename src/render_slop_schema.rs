//! The records of a markdown outline: after the stamp line, every line is
//! one compact JSON object whose `k` key names its kind. These types are
//! the schema: a field that is not an `Option` is required, an optional
//! field is left out when it is empty, and the fields print in the order
//! they are declared. The catalog's records are in `render_slop_catalog`,
//! and `render_slop_split` cuts a record too wide for one line into `more`
//! records.

use std::fmt;

#[cfg(test)]
use serde::{Deserialize, Deserializer};
use serde::{Serialize, Serializer};

use crate::render_slop_catalog::{
    AliasRecord, ExternRecord, FieldRecord, FnRecord, ImplRecord, MacroRecord, ModRecord,
    TraitRecord, TypeRecord, UnsupportedRecord, UseRecord, ValueRecord, VariantRecord,
};
use crate::render_slop_split::Node;

/// The schema version the outline header carries.
pub(crate) const SCHEMA: u32 = 1;

/// One line of an outline after the stamp, tagged by its kind. The kinds
/// print in this order: the header, the modules, the leads, the surface,
/// the external items, the spellings, the wire keys, and then the catalog,
/// module by module, where a `doc` record follows the record it belongs to.
/// A `more` record follows the record it continues, or another `more`
/// record of that record.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Deserialize))]
#[serde(tag = "k")]
pub(crate) enum Record {
    #[serde(rename = "vibelight-outline")]
    Header(Header),
    #[serde(rename = "modules")]
    Modules(Modules),
    #[serde(rename = "lead")]
    Lead(LeadRecord),
    #[serde(rename = "surface")]
    Surface(SurfaceRecord),
    #[serde(rename = "external")]
    External(ExternalRecord),
    #[serde(rename = "spelling")]
    Spelling(SpellingRecord),
    #[serde(rename = "wire-keys")]
    WireKeys(WireKeysRecord),
    #[serde(rename = "doc")]
    Doc(DocRecord),
    #[serde(rename = "mod")]
    Mod(ModRecord),
    #[serde(rename = "struct")]
    Struct(TypeRecord),
    #[serde(rename = "enum")]
    Enum(TypeRecord),
    #[serde(rename = "union")]
    Union(TypeRecord),
    #[serde(rename = "trait")]
    Trait(TraitRecord),
    #[serde(rename = "alias")]
    Alias(AliasRecord),
    #[serde(rename = "field")]
    Field(FieldRecord),
    #[serde(rename = "variant")]
    Variant(VariantRecord),
    #[serde(rename = "fn")]
    Fn(FnRecord),
    #[serde(rename = "const")]
    Const(ValueRecord),
    #[serde(rename = "static")]
    Static(ValueRecord),
    #[serde(rename = "extern")]
    Extern(ExternRecord),
    #[serde(rename = "use")]
    Use(UseRecord),
    #[serde(rename = "impl")]
    Impl(ImplRecord),
    #[serde(rename = "macro")]
    Macro(MacroRecord),
    #[serde(rename = "unsupported")]
    Unsupported(UnsupportedRecord),
    #[serde(rename = "more")]
    More(MoreRecord),
}

/// Leaves an optional list out of the output when it is absent or empty.
#[expect(
    clippy::ref_option,
    reason = "serde's skip_serializing_if passes the field itself by reference"
)]
pub(crate) fn absent<T>(list: &Option<Vec<T>>) -> bool {
    list.as_ref().is_none_or(Vec::is_empty)
}

/// What kind of target an outline is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(Deserialize))]
#[serde(rename_all = "kebab-case")]
pub(crate) enum TargetName {
    Lib,
    Bin,
    SingleFile,
}

/// The first record: what the outline is of.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct Header {
    pub(crate) schema: u32,
    #[serde(rename = "crate")]
    pub(crate) crate_name: String,
    pub(crate) target: TargetName,
    /// The crate's folder below the base of the run; left out for the base.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) dir: Option<String>,
    /// The crate's folder from the markdown folder, with forward slashes,
    /// which a location's file is below; the goldens leave it out.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) root: Option<String>,
    /// How many leads the config's `quiet` entries dropped from the
    /// outline; left out when none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) quieted: Option<usize>,
}

/// The second record: the output lines each module's catalog records take.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct Modules {
    pub(crate) items: Vec<ModuleRange>,
}

/// One module's 1-based, inclusive output-line range; left out for a
/// module with no record.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct ModuleRange {
    pub(crate) path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) from: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) to: Option<usize>,
}

/// A source line, written as one string: `<file>:<line>`, or
/// `<stem>:<file>:<line>` for a file of another target of the run. The
/// file is relative to the crate, with forward slashes, and holds no
/// colon, so a reader splits the string from the right.
#[derive(Clone, Debug, Default)]
pub(crate) struct Loc {
    /// The outline stem of the other target of the run that holds the file.
    pub(crate) stem: Option<String>,
    pub(crate) file: String,
    pub(crate) line: u32,
}

impl fmt::Display for Loc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(stem) = &self.stem {
            write!(f, "{stem}:")?;
        }
        write!(f, "{}:{}", self.file, self.line)
    }
}

impl Serialize for Loc {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

#[cfg(test)]
impl Loc {
    /// Reads a location the way it prints, or `None` when the text is no
    /// location in its printed form.
    pub(crate) fn parse(text: &str) -> Option<Loc> {
        let mut parts = text.rsplitn(3, ':');
        let (line, file) = (parts.next()?, parts.next()?);
        let stem = parts.next();
        let valid = !file.is_empty()
            && !file.contains('\\')
            && stem.is_none_or(|stem| !stem.is_empty())
            && line.bytes().all(|b| b.is_ascii_digit());
        let loc = Loc {
            stem: stem.map(str::to_owned),
            file: file.to_owned(),
            line: line.parse().ok().filter(|_| valid)?,
        };
        (loc.to_string() == text).then_some(loc)
    }
}

#[cfg(test)]
impl<'de> Deserialize<'de> for Loc {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Loc, D::Error> {
        let text = String::deserialize(deserializer)?;
        Loc::parse(&text)
            .ok_or_else(|| serde::de::Error::custom(format!("`{text}` is no location")))
    }
}

/// How a lead's evidence was found.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(Deserialize))]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Mode {
    Resolved,
    DirectSyntax,
    Lexical,
}

/// A lead: something for an analyst to confirm or extinguish.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct LeadRecord {
    pub(crate) id: String,
    pub(crate) mode: Mode,
    pub(crate) subjects: Vec<String>,
    pub(crate) evidence: String,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) locs: Option<Vec<Loc>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) limits: Option<Vec<String>>,
    /// For a summary, the module whose leads it stands for; set together
    /// with `count`, and never alone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) module: Option<String>,
    /// For a summary, how many leads it stands for; set together with
    /// `module`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) count: Option<usize>,
}

/// What a public path names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(Deserialize))]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SurfaceKind {
    Mod,
    Struct,
    Enum,
    Union,
    Trait,
    Alias,
    Fn,
    Value,
    Macro,
    External,
}

/// One path a user of a library can name.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct SurfaceRecord {
    pub(crate) path: String,
    pub(crate) kind: SurfaceKind,
    /// Where the item is declared, when that is not `path`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) item: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) test_support: Option<bool>,
}

/// An item of another crate that the declarations name.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct ExternalRecord {
    pub(crate) path: String,
    pub(crate) uses: usize,
    pub(crate) named_by: Vec<String>,
    /// The first paragraph of its doc, for an item of the run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) summary: Option<String>,
}

/// An outside item the declarations write in more than one way.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct SpellingRecord {
    pub(crate) path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) preferred: Option<String>,
    pub(crate) spellings: Vec<Spelling>,
}

/// One way of writing an outside item.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct Spelling {
    pub(crate) text: String,
    pub(crate) count: usize,
    pub(crate) locs: Vec<Loc>,
}

/// The keys a wire type reads or writes.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct WireKeysRecord {
    pub(crate) path: String,
    pub(crate) keys: Vec<String>,
    pub(crate) loc: Loc,
}

/// The doc text of the record just before it.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct DocRecord {
    pub(crate) path: String,
    pub(crate) text: String,
}

/// The rest of a record that is too wide for one line: more elements of
/// one of its arrays, or more of one of its strings. Exactly one of `items`
/// and `text` is set: `items` continues an array field and `text` continues
/// a string field.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct MoreRecord {
    /// The owner's `path`, a lead's first subject, or `modules`.
    pub(crate) path: String,
    /// A lead's `id`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<String>,
    /// The field this part continues, its steps joined with dots, such as
    /// `locs`, `spellings.3.locs`, or `ops.2`.
    pub(crate) field: String,
    /// The part's number, from 2; the owner is part 1.
    pub(crate) part: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) items: Option<Vec<Node>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) text: Option<String>,
}
