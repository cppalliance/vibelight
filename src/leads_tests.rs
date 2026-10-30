//! Tests the leads of small packages that are read, resolved, and linked
//! the way a run reads them: the summary rule, the checks that reject a
//! broken lead, the glossary in `tools/vibelight.md`, and that the leads
//! are the same however often they are found. The detectors' own cases are
//! in the sibling test files.

use std::fs;
use std::path::Path;

use crate::analysis::AnalysisRun;
use crate::analysis_tests::{fixture, run, run_with};
use crate::leads::{self, Lead, Mode, SUMMARY_FROM, Summary};

/// The manifest of a package named `t`.
pub(crate) const MANIFEST: (&str, &str) = ("Cargo.toml", "[package]\nname = \"t\"\n");

/// Shows a lead's subjects joined by `, `, each as its path below its
/// crate's root, after its target's index when the run has several.
fn shown(run: &AnalysisRun<'_>, lead: &Lead) -> String {
    let several = run.targets.len() > 1;
    let subjects: Vec<String> = (lead.subjects.iter())
        .map(|subject| {
            let path = subject.path.join("::");
            if several {
                format!("{}:{path}", subject.target)
            } else {
                path
            }
        })
        .collect();
    subjects.join(", ")
}

/// Calls `check` with the run and the checked leads of a package made of
/// `files`.
pub(crate) fn with_leads(files: &[(&str, &str)], check: impl FnOnce(&AnalysisRun<'_>, &[Lead])) {
    let fx = fixture(files);
    let run = run(&fx);
    let leads = leads::find(&run, &[]).leads;
    leads::validate(&leads, &run).unwrap();
    check(&run, &leads);
}

/// Returns each checked lead of a package made of `files`, as its id and
/// its shown subjects.
pub(crate) fn leads_in(files: &[(&str, &str)]) -> Vec<(&'static str, String)> {
    let mut out = Vec::new();
    with_leads(files, |run, leads| {
        out = (leads.iter())
            .map(|lead| (lead.id, shown(run, lead)))
            .collect();
    });
    out
}

/// Returns the shown subjects of each lead with `id` of a package made of
/// `files`.
pub(crate) fn found(files: &[(&str, &str)], id: &str) -> Vec<String> {
    (leads_in(files).into_iter())
        .filter(|(lead, _)| *lead == id)
        .map(|(_, subjects)| subjects)
        .collect()
}

/// Returns the shown subjects of each lead with `id` of a library made of
/// one `lib.rs`.
pub(crate) fn lib(source: &str, id: &str) -> Vec<String> {
    found(&[MANIFEST, ("src/lib.rs", source)], id)
}

/// Returns the ids and shown subjects of the checked leads of a library
/// made of one `lib.rs`, whose config marks `boilerplate` texts.
pub(crate) fn lib_with(source: &str, boilerplate: &[&str]) -> Vec<(&'static str, String)> {
    let fx = fixture(&[MANIFEST, ("src/lib.rs", source)]);
    let texts: Vec<String> = boilerplate.iter().map(|t| (*t).to_owned()).collect();
    let run = run_with(&fx, &texts);
    let leads = leads::find(&run, &[]).leads;
    leads::validate(&leads, &run).unwrap();
    (leads.iter())
        .map(|lead| (lead.id, shown(&run, lead)))
        .collect()
}

/// A library that many detectors find something in.
const SAMPLE: &str = "pub struct Config { pub verbose: Option<bool>, pub status: String }\n\
    pub enum Only { One }\n\
    pub enum Event { Start(u64), Stop(u64) }\n\
    pub fn paint(red: bool, green: bool, blue: bool) {}\n\
    pub fn bounds() -> (u32, u32) { (0, 0) }\n\
    pub fn checksum_bytes(data: &[u8], seed: u32) -> u32 { seed }\n\
    #[allow(dead_code)]\n\
    fn legacy_reset_for_test(_unused: u8) {}\n";

#[test]
fn more_than_five_leads_of_one_id_in_one_module_become_one_summary() {
    let flags = |n: usize| {
        let fields = (0..n).map(|i| format!("pub flag_{i}: Option<bool>"));
        fields.collect::<Vec<_>>().join(",\n")
    };
    let source = format!(
        "pub struct Many {{ {} }}\npub mod few {{ pub struct Few {{ {} }} }}\n",
        flags(SUMMARY_FROM + 2),
        flags(SUMMARY_FROM)
    );
    with_leads(&[MANIFEST, ("src/lib.rs", &source)], |_, leads| {
        let flags: Vec<&Lead> = leads.iter().filter(|l| l.id == "option-as-flag").collect();
        assert_eq!(flags.len(), 1 + SUMMARY_FROM, "one summary and five leads");
        let summary = flags.iter().find(|l| l.summary.is_some()).unwrap();
        let counted = summary.summary.map(|s| (s.target, s.module, s.count));
        assert_eq!(counted, Some((0, 0, SUMMARY_FROM + 2)));
        let names: Vec<&str> = summary
            .subjects
            .iter()
            .map(|s| s.path[1].as_str())
            .collect();
        let expected: Vec<String> = (0..SUMMARY_FROM + 2).map(|i| format!("flag_{i}")).collect();
        assert_eq!(names, expected, "sorted by name");
        assert_eq!(summary.locs.len(), SUMMARY_FROM + 2);
        assert!(
            summary
                .evidence
                .contains("`flag_0: Option<bool>`; `flag_1: Option<bool>`")
        );
        let rest = flags.iter().filter(|l| l.summary.is_none());
        assert!(rest.clone().all(|l| l.subjects[0].path[0] == "few"));
        assert_eq!(rest.count(), SUMMARY_FROM, "five leads stay leads");
    });
}

#[test]
fn quiet_entries_drop_their_leads_before_the_summary_and_count_them() {
    let fx = fixture(&[MANIFEST, ("src/lib.rs", SAMPLE)]);
    let run = run(&fx);
    let all = leads::find(&run, &[]);
    assert!(all.quieted == [0] && all.notes.is_empty());
    let quiet = [
        "option-as-flag".to_owned(),
        "same-type-params:pa*".to_owned(),
        "tuple-return:Config::*".to_owned(),
    ];
    let found = leads::find(&run, &quiet);
    leads::validate(&found.leads, &run).unwrap();
    let ids = |leads: &[Lead], id: &str| leads.iter().filter(|l| l.id == id).count();
    assert!(ids(&all.leads, "option-as-flag") > 0 && ids(&all.leads, "same-type-params") == 1);
    assert_eq!(ids(&found.leads, "option-as-flag"), 0);
    assert_eq!(
        ids(&found.leads, "same-type-params"),
        0,
        "`paint` matches `pa*`"
    );
    let dropped = ids(&all.leads, "option-as-flag") + 1;
    assert_eq!(found.leads.len() + dropped, all.leads.len());
    assert_eq!(found.quieted, [dropped]);
    assert_eq!(
        found.notes,
        ["quiet entry tuple-return:Config::* matches no lead"]
    );
}

#[test]
fn a_broken_lead_fails_validation() {
    let files = [
        MANIFEST,
        (
            "src/lib.rs",
            "pub struct S { pub a: Option<bool>, pub b: Option<()> }\n",
        ),
    ];
    with_leads(&files, |run, good| {
        let broken = |change: &dyn Fn(&mut Vec<Lead>)| {
            let mut leads = good.to_vec();
            change(&mut leads);
            leads::validate(&leads, run).unwrap_err()
        };
        assert!(broken(&|l| l.reverse()).contains("order"));
        assert!(broken(&|l| l[0].id = "no-such-lead").contains("registry"));
        assert!(broken(&|l| l[0].subjects.clear()).contains("subject"));
        assert!(broken(&|l| l[0].target_files.push(9)).contains("targets"));
        assert!(broken(&|l| l[0].limits.push("telepathy")).contains("limit"));
        assert!(broken(&|l| l[0].locs[0].line = 0).contains("location"));
        let absent = |l: &mut Vec<Lead>| {
            l[0].id = "pub-off-surface";
            l[0].mode = Mode::Resolved;
        };
        assert!(broken(&absent).contains("absence"));
        let summary = Summary {
            target: 0,
            module: 0,
            count: 2,
        };
        assert!(broken(&|l| l[0].summary = Some(summary)).contains("summarizes"));
    });
}

#[test]
fn the_leads_are_the_same_wherever_and_however_often_they_are_found() {
    let files = [
        MANIFEST,
        ("src/lib.rs", SAMPLE),
        (
            "src/main.rs",
            "fn checksum_bytes(data: &[u8], seed: u32) -> u32 { seed }\nfn main() {}\n",
        ),
    ];
    let first = leads_in(&files);
    assert_eq!(first, leads_in(&files));
    let ids: Vec<&str> = leads::ids().collect();
    let ranks: Vec<usize> = (first.iter())
        .map(|(id, _)| ids.iter().position(|known| known == id).unwrap())
        .collect();
    assert!(
        ranks.windows(2).all(|w| w[0] <= w[1]),
        "grouped by id in registry order"
    );
    assert!(first.len() >= 8, "the sample finds leads of several ids");
    with_leads(&files, |run, leads| {
        assert_eq!(leads, leads::find(run, &[]).leads.as_slice());
        let twice: Vec<&Lead> = leads.iter().filter(|l| l.target_files.len() > 1).collect();
        assert!(!twice.is_empty(), "a lead across targets prints in both");
    });
}

/// Reads the lines between the glossary's tags: a blank line first and
/// last, `####` group headings, and one `- **<id>**: <rule>` bullet per id
/// with exactly a `  - Wrong when: ` and a `  - Verify: ` child, each a
/// sentence without an em dash or a double dash. Returns the ids in order,
/// or the first line that breaks the form.
fn glossary_ids<'t>(body: &[&'t str]) -> Result<Vec<&'t str>, String> {
    if body.first() != Some(&"") || body.last() != Some(&"") {
        return Err("the tags have no blank line beside them".to_owned());
    }
    let mut entries: Vec<(&str, Vec<&str>)> = Vec::new();
    for &line in body {
        if let Some(rest) = line.strip_prefix("- **") {
            let (id, rule) = rest.split_once("**: ").ok_or(line)?;
            entries.push((id, vec![rule]));
        } else if line.starts_with("  - ") {
            entries.last_mut().ok_or(line)?.1.push(line);
        } else if !line.is_empty() && !line.starts_with("#### ") {
            return Err(line.to_owned());
        }
    }
    let sentence =
        |text: &str| text.ends_with('.') && !text.contains('\u{2014}') && !text.contains("--");
    for (id, parts) in &entries {
        let [rule, wrong, verify] = parts.as_slice() else {
            return Err(format!(
                "{id} needs one `Wrong when:` and one `Verify:` line"
            ));
        };
        let children = wrong.starts_with("  - Wrong when: ") && verify.starts_with("  - Verify: ");
        if !children || ![rule, wrong, verify].into_iter().all(|text| sentence(text)) {
            return Err(format!("{id} is not in the glossary's form"));
        }
    }
    Ok(entries.into_iter().map(|(id, _)| id).collect())
}

#[test]
fn the_glossary_in_the_lens_gives_every_registered_id_one_bullet_in_order() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tools")
        .join("vibelight.md");
    let text = fs::read_to_string(path).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    let at = |wanted: &str| {
        let found: Vec<usize> = (lines.iter().enumerate())
            .filter(|(_, line)| line.starts_with(wanted))
            .map(|(n, _)| n)
            .collect();
        assert_eq!(found.len(), 1, "{wanted} is on one line of its own");
        found[0]
    };
    let open = at("<lead-glossary");
    assert_eq!(lines[open], "<lead-glossary schema=\"1\">");
    let close = at("</lead-glossary>");
    assert!(at("<slop-lens>") < open && close < at("</slop-lens>"));
    let body = &lines[open + 1..close];
    assert_eq!(glossary_ids(body), Ok(leads::ids().collect::<Vec<_>>()));
    let headings = body.iter().filter(|line| line.starts_with("#### ")).count();
    assert_eq!(headings, 7, "one heading per detector group");
    let entry = [
        "",
        "#### Shapes",
        "",
        "- **option-as-flag**: An `Option<bool>` field.",
        "  - Wrong when: the callers need three states.",
        "  - Verify: read the readers.",
        "",
    ];
    assert_eq!(glossary_ids(&entry), Ok(vec!["option-as-flag"]));
    for dropped in [4, 5] {
        let mut broken = entry.to_vec();
        broken.remove(dropped);
        assert!(glossary_ids(&broken).is_err(), "a missing child fails");
    }
    let mut dashed = entry.to_vec();
    dashed[5] = "  - Verify: read the readers -- all of them.";
    assert!(glossary_ids(&dashed).is_err(), "a double dash fails");
}
