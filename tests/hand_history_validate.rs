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
///
/// The file is checked for what it shows without playing it: unlisted seats
/// (Seats), a winner with the weaker hand and a rank value outside its class
/// (Result, items 2 and 3), and a loss larger than the stack (Money, item 5).
/// Items 4 (an unaffordable call, Amount), 6 (pots without the blinds, Pot)
/// and 7 (a raise below the minimum, Legality) need the engine's walk, and
/// this file stops the walk at its first action, by a seat that is not
/// listed. They are pinned by their own unit fixtures in
/// `src/hand_history/check.rs`: `validate__unaffordable_call_is_an_amount_finding`,
/// `validate__street_pot_excluding_blinds_is_a_pot_finding` and
/// `validate__raise_below_minimum_is_a_legality_finding`. Item 1 (the hole
/// cards swapped between the players) breaks no rule: the file agrees with
/// itself about who held what, so no checker can see it.
#[test]
fn the_hand_before__reports_defect_040() {
    let hand = load("tests/defects/validate/DEFECT_040_the_hand_before.yaml");
    let rules: HashSet<Rule> = hand.validate().iter().map(|finding| finding.rule).collect();
    for rule in [Rule::Seats, Rule::Result, Rule::Money] {
        assert!(rules.contains(&rule), "{rule:?} missing from {rules:?}");
    }
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
