//! Defect hands: one YAML hand per fixed defect, replayed through the engine.
//!
//! Each `tests/defects/DEFECT_NNN*.yaml` file is a normal
//! [`HandHistory`](pkcore::hand_history::HandHistory) — it loads with
//! `HandHistory::from_yaml` and replays with `HandHistory::replay` — plus two
//! top-level keys that only this runner reads:
//!
//! - `expect:` — checks on the table **after** a recorded action, addressed by
//!   street and the action's position in that street's `actions` list.
//! - `final:` — checks on the finished hand.
//!
//! `HandHistory` ignores keys it does not know, so the extra blocks never reach
//! the library. Read a file to see the exact hand that produced the defect, and
//! the state the fix guarantees at the moment it went wrong.
//!
//! ```yaml
//! expect:
//!   - after: { street: preflop, index: 1 }
//!     bet: 400                     # table.bet
//!     raise_increment: 300         # table.raise_increment
//!     min_raise_to: 700            # table.min_raise_to()
//!     next_to_act: 2               # table.next_to_act()
//!     to_call: { seat: 2, amount: 300 }
//!     legal: { seat: 2, has: [call], lacks: [check] }
//! final:
//!   consistent: true               # ReplayResult::is_consistent
//!   stacks: { 0: 10700, 1: 0 }     # seat → chips after the hand
//! ```

use pkcore::casino::action::PlayerAction;
use pkcore::casino::table::Table;
use pkcore::hand_history::{HandHistory, ReplayStep, ReplayStreet};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The runner-only part of a defect file.
#[derive(Debug, Default, Deserialize)]
struct DefectChecks {
    #[serde(default)]
    expect: Vec<Checkpoint>,
    #[serde(default, rename = "final")]
    finish: Option<FinalCheck>,
}

#[derive(Debug, Deserialize)]
struct Checkpoint {
    after: StepRef,
    bet: Option<usize>,
    raise_increment: Option<usize>,
    min_raise_to: Option<usize>,
    next_to_act: Option<u8>,
    to_call: Option<ToCall>,
    legal: Option<Legal>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct StepRef {
    street: ReplayStreet,
    index: usize,
}

#[derive(Debug, Deserialize)]
struct ToCall {
    seat: u8,
    amount: usize,
}

#[derive(Debug, Deserialize)]
struct Legal {
    seat: u8,
    #[serde(default)]
    has: Vec<String>,
    #[serde(default)]
    lacks: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct FinalCheck {
    consistent: Option<bool>,
    #[serde(default)]
    stacks: BTreeMap<u8, usize>,
}

fn action_name(action: &PlayerAction) -> &'static str {
    match action {
        PlayerAction::Fold => "fold",
        PlayerAction::Check => "check",
        PlayerAction::Call => "call",
        PlayerAction::Bet(_) => "bet",
        PlayerAction::Raise(_) => "raise",
        PlayerAction::AllIn => "all_in",
    }
}

/// Compares one expected value and records a failure line when it differs.
fn compare<T: PartialEq + std::fmt::Debug>(failures: &mut Vec<String>, at: &str, what: &str, want: &T, got: &T) {
    if want != got {
        failures.push(format!("{at}: {what} expected {want:?}, got {got:?}"));
    }
}

fn check(point: &Checkpoint, table: &Table, failures: &mut Vec<String>) {
    let at = format!("after {} #{}", point.after.street, point.after.index);
    if let Some(want) = point.bet {
        compare(failures, &at, "bet", &want, &table.bet);
    }
    if let Some(want) = point.raise_increment {
        compare(failures, &at, "raise_increment", &want, &table.raise_increment);
    }
    if let Some(want) = point.min_raise_to {
        compare(failures, &at, "min_raise_to", &want, &table.min_raise_to());
    }
    if let Some(want) = point.next_to_act {
        compare(failures, &at, "next_to_act", &want, &table.next_to_act());
    }
    if let Some(to_call) = &point.to_call {
        let what = format!("to_call(seat {})", to_call.seat);
        compare(failures, &at, &what, &to_call.amount, &table.to_call(to_call.seat));
    }
    if let Some(legal) = &point.legal {
        let offered: Vec<&str> = table.legal_actions(legal.seat).iter().map(action_name).collect();
        for name in &legal.has {
            if !offered.contains(&name.as_str()) {
                failures.push(format!(
                    "{at}: seat {} should be offered {name}, got {offered:?}",
                    legal.seat
                ));
            }
        }
        for name in &legal.lacks {
            if offered.contains(&name.as_str()) {
                failures.push(format!(
                    "{at}: seat {} must not be offered {name}, got {offered:?}",
                    legal.seat
                ));
            }
        }
    }
}

/// Replays one defect file and returns every failed check as a line of text.
fn run_defect_file(path: &Path) -> Vec<String> {
    let yaml = match std::fs::read_to_string(path) {
        Ok(yaml) => yaml,
        Err(error) => return vec![format!("cannot read: {error}")],
    };
    let hand = match HandHistory::from_yaml(&yaml) {
        Ok(hand) => hand,
        Err(error) => return vec![format!("not a valid hand history: {error}")],
    };
    let checks: DefectChecks = match serde_yaml_bw::from_str(&yaml) {
        Ok(checks) => checks,
        Err(error) => return vec![format!("expect/final blocks do not parse: {error}")],
    };

    let mut failures = Vec::new();
    let mut reached = vec![false; checks.expect.len()];
    let result = hand.replay_with(|step: &ReplayStep<'_>, table| {
        for (i, point) in checks.expect.iter().enumerate() {
            if point.after.street == step.street && point.after.index == step.index {
                reached[i] = true;
                check(point, table, &mut failures);
            }
        }
    });
    let result = match result {
        Ok(result) => result,
        Err(error) => {
            failures.push(format!("replay failed: {error:?}"));
            return failures;
        }
    };

    for (point, hit) in checks.expect.iter().zip(&reached) {
        if !hit {
            failures.push(format!("{:?}: no such action in the hand", point.after));
        }
    }
    if let Some(finish) = &checks.finish {
        if let Some(want) = finish.consistent {
            compare(&mut failures, "final", "consistent", &want, &result.is_consistent);
        }
        for (seat, want) in &finish.stacks {
            let got = result
                .final_stacks
                .iter()
                .find(|(s, _)| s == seat)
                .map(|(_, chips)| *chips);
            compare(
                &mut failures,
                "final",
                &format!("stack(seat {seat})"),
                &Some(*want),
                &got,
            );
        }
    }
    failures
}

/// The defect files, read from disk on every run so a new file needs no rebuild.
fn defect_files() -> std::io::Result<Vec<PathBuf>> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/defects");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.extension().is_some_and(|ext| ext == "yaml")
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("DEFECT_"))
        })
        .collect();
    files.sort();
    Ok(files)
}

#[test]
fn every_defect_hand_replays_as_expected() {
    let files = defect_files().expect("tests/defects exists");
    assert!(!files.is_empty(), "no DEFECT_*.yaml files in tests/defects");

    let mut report = Vec::new();
    for path in &files {
        let name = path.file_name().and_then(|name| name.to_str()).unwrap_or("?");
        let failures = run_defect_file(path);
        if failures.is_empty() {
            println!("ok   {name}");
        } else {
            println!("FAIL {name}");
            report.push(format!("{name}:\n  {}", failures.join("\n  ")));
        }
    }
    assert!(report.is_empty(), "\n{}", report.join("\n"));
}

#[test]
fn a_wrong_expectation_fails_the_file() {
    let dir = std::env::temp_dir().join(format!("pkcore_defect_hands_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let source = defect_files()
        .expect("tests/defects exists")
        .into_iter()
        .find(|path| path.to_string_lossy().contains("DEFECT_027"))
        .expect("DEFECT_027 file present");
    let yaml = std::fs::read_to_string(source).expect("read DEFECT_027");
    let broken = dir.join("DEFECT_000_broken.yaml");
    std::fs::write(&broken, yaml.replace("    bet: 400\n", "    bet: 999\n")).expect("write");

    let failures = run_defect_file(&broken);
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(
        vec!["after preflop #1: bet expected 999, got 400".to_string()],
        failures
    );
}
