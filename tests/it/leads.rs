//! Runs the binary on a small crate written to show representative slop,
//! and checks that its markdown carries the leads the detectors find, each
//! with its evidence mode, and each lead that rests on absence with its
//! limits, without another golden fixture.

use std::fs;

use serde_json::Value;

use crate::support::run;

/// A library with one instance of each lead the test looks for.
const LIB: &str = "//! Slop on purpose.

/// A setting that is set, unset, or neither.
pub struct Settings {
    /// Whether to be loud.
    pub verbose: Option<bool>,
}

/// An enum of one variant.
pub enum Only {
    One,
}

/// Places a point.
pub fn place(x: u32, y: u32, z: u32) -> u32 {
    x + y + z
}

/// TODO: say what this keeps.
#[allow(dead_code)]
fn kept() {}
";

/// Returns every record of the outline after its stamp line.
fn records(text: &str) -> Vec<Value> {
    text.lines()
        .skip(1)
        .map(|line| serde_json::from_str(line).unwrap_or_else(|e| panic!("{e}: {line}")))
        .collect()
}

#[test]
fn a_crate_with_slop_prints_its_leads_first_with_modes_and_limits() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let dir = tmp.path().join("sloppy");
    fs::create_dir_all(dir.join("src")).expect("a crate folder can be made");
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"sloppy\"\nedition = \"2024\"\n",
    )
    .expect("a manifest can be written");
    fs::write(dir.join("src").join("lib.rs"), LIB).expect("a source can be written");
    let out = tmp.path().join("out");
    let result = run(&dir, &out);
    assert!(result.status.success(), "{:?}", result.stderr);
    let text = fs::read_to_string(out.join("sloppy.md")).expect("the outline was written");
    let records = records(&text);
    assert_eq!(records[0]["k"], "vibelight-outline");
    assert_eq!(records[0]["schema"], 1);
    assert_eq!(records[1]["k"], "modules");
    let leads: Vec<&Value> = records.iter().filter(|r| r["k"] == "lead").collect();
    let heads = ["vibelight-outline", "modules", "lead"];
    let rest = records
        .iter()
        .position(|r| !heads.iter().any(|k| r["k"] == *k));
    let last_lead = records.iter().rposition(|r| r["k"] == "lead");
    assert!(
        last_lead.is_some() && last_lead < rest,
        "the leads come first"
    );
    let find = |id: &str, subject: &str| -> &Value {
        leads
            .iter()
            .find(|lead| {
                lead["id"] == id
                    && lead["subjects"]
                        .as_array()
                        .is_some_and(|s| s.contains(&subject.into()))
            })
            .unwrap_or_else(|| panic!("no {id} lead names {subject}:\n{text}"))
    };
    assert_eq!(
        find("option-as-flag", "Settings::verbose")["mode"],
        "direct-syntax"
    );
    assert_eq!(find("single-variant-enum", "Only")["mode"], "direct-syntax");
    assert_eq!(find("same-type-params", "place")["mode"], "direct-syntax");
    assert_eq!(find("lint-suppression", "kept")["mode"], "direct-syntax");
    assert_eq!(find("papercut", "kept")["mode"], "direct-syntax");
    let unused = find("no-observed-name-use", "kept");
    assert_eq!(unused["mode"], "lexical");
    let limits = unused["limits"]
        .as_array()
        .expect("an absence lead names its limits");
    assert!(limits.contains(&"macros".into()), "{limits:?}");
    for lead in &leads {
        let locs = lead["locs"].as_array().expect("a lead has locations");
        assert!(locs.iter().all(|loc| {
            (loc.as_str())
                .and_then(|loc| loc.strip_prefix("src/lib.rs:"))
                .and_then(|line| line.parse::<u32>().ok())
                .is_some_and(|line| line > 0)
        }));
    }
}
