//! `HandHistory::validate` against the files in the repo.
//!
//! `tests/defects/validate/` holds hands that are wrong on purpose, one file per
//! defect. `data/hands/` holds the hands that must be right.

#![allow(non_snake_case)]

use pkcore::hand_history::{HandHistory, Rule};
use std::collections::HashSet;
use std::path::Path;

fn load(relative: &str) -> HandHistory {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    HandHistory::from_yaml(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// `DEFECT_040`: the hand as it stood before EPIC-93 repaired it. It lists
/// only seats 1 and 4 of its six-seat table, and has seats 2, 3, 5 and 6 act
/// without being listed.
#[test]
fn the_hand_before__reports_defect_040() {
    let hand = load("tests/defects/validate/DEFECT_040_the_hand_before.yaml");
    let rules: HashSet<Rule> = hand.validate().iter().map(|finding| finding.rule).collect();
    assert!(rules.contains(&Rule::Seats), "{rules:?}");
}

/// The repaired `the_hand.yaml` must validate clean. It does not yet: the file
/// is repaired in EPIC-93 Task 9.
#[test]
#[ignore = "DEFECT_040: red until EPIC-93 Task 9"]
fn the_hand_yaml__validates() {
    let findings = load("data/hands/the_hand.yaml").validate();
    assert!(
        findings.is_empty(),
        "{}",
        findings.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n")
    );
}

/// `DEFECT_013`: a dead button (on empty chair 1) and a zero post by empty
/// chair 2 are legal (TDA Rule 32), so the Seats rule has nothing to say.
#[test]
fn dead_button_fixture__has_no_seats_finding() {
    let findings = load("tests/defects/DEFECT_013_dead_button.yaml").validate();
    let seats: Vec<String> = findings
        .iter()
        .filter(|finding| finding.rule == Rule::Seats)
        .map(ToString::to_string)
        .collect();
    assert!(seats.is_empty(), "{}", seats.join("\n"));
}
