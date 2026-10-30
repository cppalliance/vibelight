//! Writes a target's markdown outline from the run's analysis view: the
//! stamp line, then one compact JSON record per line. The header and the
//! module ranges come first, then the target's leads, its public surface,
//! the outside items it names, its spellings, and its wire keys, and then
//! the catalog of every module, declaration, member, and module item,
//! module by module in path order and by source line inside each. The
//! outline reads the crates only as resolve and `link` left them, before
//! any impl is attached, and prints no body's code. A record too wide for
//! one line continues in `more` records right after it.

use anyhow::{Result, bail};

use crate::analysis::AnalysisRun;
use crate::leads::Lead;
use crate::model::{self, Crate, ModId, path_order};
use crate::render_slop_facts::{Facts, module_path};
use crate::render_slop_schema::{
    DocRecord, Header, Loc, ModuleRange, Modules, Record, SCHEMA, TargetName,
};
use crate::root::TargetKind;
use crate::{
    render_slop_decl, render_slop_items, render_slop_lists, render_slop_split, render_slop_wire,
    stamp,
};

/// One target of the run's analysis view.
#[derive(Clone, Copy)]
pub(crate) struct AnalysisTarget<'a> {
    pub(crate) run: &'a AnalysisRun<'a>,
    pub(crate) target: usize,
}

/// What one target's markdown is rendered from: the target's analysis view
/// and the run's leads, and what depends on where the run writes, which the
/// analysis view never holds.
pub(crate) struct RenderInput<'a> {
    pub(crate) analysis_target: AnalysisTarget<'a>,
    /// Every lead of the run; the outline prints those it is named in.
    pub(crate) leads: &'a [Lead],
    /// How many leads the config's `quiet` entries dropped from this
    /// outline.
    pub(crate) quieted: usize,
    pub(crate) stamp: &'a str,
    pub(crate) target_kind: TargetKind,
    /// The crate's folder below the base of the run, `.` for the base.
    pub(crate) shown_dir: String,
    /// The crate's folder from the markdown folder, with forward slashes.
    pub(crate) root: String,
}

/// One record of the catalog and the `doc` record after it, with where it
/// sorts in its module: the module's own record first, then its re-exports
/// by their text, since rustfmt reorders `use` items, then the rest by file
/// and line.
pub(crate) struct Entry {
    pub(crate) module: ModId,
    /// 0 for the module's own record, 1 for a re-export, and 2 otherwise.
    pub(crate) group: u8,
    pub(crate) file: String,
    pub(crate) line: u32,
    /// What a re-export sorts by: its text.
    pub(crate) key: String,
    pub(crate) records: Vec<Record>,
}

/// A target's outline while its records are built.
pub(crate) struct Outline<'a> {
    pub(crate) run: &'a AnalysisRun<'a>,
    pub(crate) t: usize,
    pub(crate) krate: &'a Crate,
    pub(crate) facts: Facts<'a>,
    pub(crate) entries: Vec<Entry>,
}

impl Outline<'_> {
    /// Returns a source line of target `target` as this outline writes it.
    pub(crate) fn loc(&self, target: usize, loc: model::Loc) -> Loc {
        Loc {
            target: (target != self.t).then(|| self.run.targets[target].stem.clone()),
            file: self.run.krates[target].files[loc.file]
                .rel
                .replace('\\', "/"),
            line: loc.line,
        }
    }

    /// Returns a source line of this outline's target.
    pub(crate) fn own(&self, loc: model::Loc) -> Loc {
        self.loc(self.t, loc)
    }

    /// Adds a catalog record of module `module`, written at `at`, with its
    /// doc text, when it has one, in a `doc` record after it.
    pub(crate) fn push(&mut self, module: ModId, at: model::Loc, record: Record, doc: Doc<'_>) {
        let mut records = vec![record];
        if let Some(text) = doc.text {
            records.push(Record::Doc(DocRecord {
                path: doc.path.to_owned(),
                text: text.to_owned(),
            }));
        }
        self.entries.push(Entry {
            module,
            group: 2,
            file: self.krate.files[at.file].rel.replace('\\', "/"),
            line: at.line,
            key: String::new(),
            records,
        });
    }

    /// Moves the record just pushed to the front of its module: the
    /// module's own record in group 0, or a re-export in group 1 under
    /// `key`.
    pub(crate) fn lift(&mut self, group: u8, key: String) {
        if let Some(entry) = self.entries.last_mut() {
            entry.group = group;
            entry.key = key;
            entry.line = 0;
        }
    }
}

/// The doc text of a record, under the record's path.
#[derive(Clone, Copy)]
pub(crate) struct Doc<'d> {
    pub(crate) path: &'d str,
    pub(crate) text: Option<&'d str>,
}

/// Renders one target's markdown outline, with `stamp` on its first line.
///
/// # Errors
/// Returns an internal error when a record cannot be written as JSON, or
/// when the modules record finds no line count that its ranges agree with.
pub(crate) fn render(input: &RenderInput<'_>) -> Result<String> {
    let AnalysisTarget { run, target: t } = input.analysis_target;
    let krate = &run.krates[t];
    let mut outline = Outline {
        run,
        t,
        krate,
        facts: Facts::new(run, t),
        entries: Vec::new(),
    };
    let mut lists = render_slop_lists::leads(&outline, input.leads);
    lists.extend(render_slop_lists::surface(&outline));
    lists.extend(render_slop_lists::external(&outline));
    lists.extend(render_slop_lists::spellings(&outline));
    lists.extend(render_slop_wire::wire_keys(&outline));
    render_slop_items::modules(&mut outline);
    render_slop_decl::decls(&mut outline);
    render_slop_decl::impls(&mut outline);
    let order = sorted_modules(krate);
    let mut rank = vec![0; krate.modules.len()];
    for (place, &m) in order.iter().enumerate() {
        rank[m] = place;
    }
    let mut entries = outline.entries;
    entries.sort_by(|a, b| {
        (rank[a.module].cmp(&rank[b.module]))
            .then(a.group.cmp(&b.group))
            .then_with(|| a.key.cmp(&b.key))
            .then_with(|| a.file.cmp(&b.file))
            .then(a.line.cmp(&b.line))
    });
    let mut listed = Vec::new();
    for record in &lists {
        listed.extend(render_slop_split::lines(record)?);
    }
    let mut catalog = Vec::new();
    let mut spans: Vec<Option<(usize, usize)>> = vec![None; krate.modules.len()];
    for entry in &entries {
        for record in &entry.records {
            let first = catalog.len();
            catalog.extend(render_slop_split::lines(record)?);
            let span = &mut spans[entry.module];
            *span = Some((span.map_or(first, |(from, _)| from), catalog.len() - 1));
        }
    }
    let header = Record::Header(Header {
        schema: SCHEMA,
        crate_name: krate.name.clone(),
        target: match input.target_kind {
            TargetKind::Lib => TargetName::Lib,
            TargetKind::Bin => TargetName::Bin,
            TargetKind::SingleFile => TargetName::SingleFile,
        },
        dir: (input.shown_dir != ".").then(|| input.shown_dir.clone()),
        root: Some(input.root.clone()),
        quieted: (input.quieted > 0).then_some(input.quieted),
    });
    let header = serde_json::to_string(&header)?;
    let modules = modules(krate, &order, &spans, listed.len())?;
    let mut out = stamp::markdown(input.stamp);
    out.push('\n');
    for line in [&header]
        .into_iter()
        .chain(&modules)
        .chain(&listed)
        .chain(&catalog)
    {
        out.push_str(line);
        out.push('\n');
    }
    Ok(out)
}

/// Writes the modules record, whose ranges count the lines it takes
/// itself. `spans` holds each module's first and last line in the catalog,
/// counted from 0, and `listed` lines sit between the modules record and
/// the catalog.
fn modules(
    krate: &Crate,
    order: &[ModId],
    spans: &[Option<(usize, usize)>],
    listed: usize,
) -> Result<Vec<String>> {
    let mut taken = 1;
    for _ in 0..8 {
        let before = 2 + taken + listed;
        let record = Record::Modules(Modules {
            items: (order.iter())
                .map(|&m| ModuleRange {
                    path: module_path(krate, m),
                    from: spans[m].map(|(from, _)| before + from + 1),
                    to: spans[m].map(|(_, to)| before + to + 1),
                })
                .collect(),
        });
        let lines = render_slop_split::lines(&record)?;
        if lines.len() == taken {
            return Ok(lines);
        }
        taken = lines.len();
    }
    bail!("internal error: the modules record takes no settled number of lines")
}

/// Returns every module of a crate in path order.
fn sorted_modules(krate: &Crate) -> Vec<ModId> {
    let modules = &krate.modules;
    let mut out: Vec<ModId> = (0..modules.len()).collect();
    out.sort_by(|&x, &y| path_order(&modules[x].path, &modules[y].path).then(x.cmp(&y)));
    out
}
