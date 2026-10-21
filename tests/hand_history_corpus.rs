//! The hand-history corpus: every hand in the repo validates clean.
//!
//! Every `*.yaml` under `data/hands/` and `tests/defects/` is played through
//! [`HandHistory::validate`]. A file holds either one hand or a
//! [`HandCollection`]; every hand in it is checked. Two directories are left
//! out:
//!
//! - `data/hands/legacy/` — session collections recorded under the
//!   pre-dead-button blind rule, "not a specification" by its own README.
//! - `tests/defects/validate/` — hands that are wrong on purpose, checked
//!   finding by finding in `tests/hand_history_validate.rs`.
//!
//! A file that holds no hands fails: a clean run must mean every file was
//! checked. And every hand that validates clean must also build a
//! [`HandHistory::cursor`], the promise `validate` makes to a consumer.
//!
//! A defect fixture that is wrong **on purpose** for its own defect is named
//! in [`EXPECTED`], with the rule it breaks and why. Each entry must still
//! fire: a fixture that stops breaking its rule is as much news as one that
//! starts.

#![allow(non_snake_case)]

use pkcore::hand_history::{Finding, HandCollection, HandHistory, Rule};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Directories walked, relative to the crate root, and the subdirectory of
/// each that is left out.
const CORPUS: &[(&str, &str)] = &[("data/hands", "legacy"), ("tests/defects", "validate")];

/// Fixtures that break a rule on purpose: `(file, rule, reason)`. Every
/// finding of that rule in that file is expected, and at least one must fire.
const EXPECTED: &[(&str, Rule, &str)] = &[(
    "tests/defects/DEFECT_014_replay_table_size.yaml",
    Rule::Seats,
    "legacy seats field holds the player count",
)];

/// Every `*.yaml` under `dir`, recursively, except under `dir/excluded`.
fn yaml_files(dir: &Path, excluded: &Path, out: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir).unwrap_or_else(|error| panic!("{}: {error}", dir.display()));
    for entry in entries {
        let path = entry
            .unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
            .path();
        if path.is_dir() {
            if path != excluded {
                yaml_files(&path, excluded, out);
            }
        } else if path.extension().is_some_and(|extension| extension == "yaml") {
            out.push(path);
        }
    }
}

/// The corpus, as paths relative to the crate root, sorted.
fn corpus() -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for (dir, excluded) in CORPUS {
        let dir = root.join(dir);
        yaml_files(&dir, &dir.join(excluded), &mut files);
    }
    let mut relative: Vec<String> = files
        .iter()
        .map(|path| {
            path.strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    relative.sort();
    relative
}

/// The hands in a file: a [`HandCollection`] (it has a `hands:` list) or a
/// single [`HandHistory`]. A file with no hands is an error: a corpus file
/// that checks nothing must not pass as clean.
fn hands(text: &str) -> Result<Vec<HandHistory>, String> {
    let hands = match HandCollection::from_yaml(text) {
        Ok(collection) => collection.hands().to_vec(),
        Err(as_collection) => HandHistory::from_yaml(text)
            .map(|hand| vec![hand])
            .map_err(|as_hand| format!("not a hand ({as_hand}) or a collection ({as_collection})"))?,
    };
    if hands.is_empty() {
        return Err("holds no hands, so nothing in it is checked".to_string());
    }
    Ok(hands)
}

/// Every finding in one file. A file that does not parse is one line of its own.
fn findings(relative: &str) -> Result<Vec<Finding>, String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    let text = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
    Ok(hands(&text)?.iter().flat_map(HandHistory::validate).collect())
}

#[test]
fn corpus__every_hand_validates_clean() {
    let files = corpus();
    assert!(
        files.iter().any(|file| file == "data/hands/the_hand.yaml"),
        "the corpus lost data/hands/the_hand.yaml: {files:?}"
    );

    let mut failures: Vec<String> = Vec::new();
    let mut fired: HashSet<(&str, Rule)> = HashSet::new();
    for file in &files {
        let findings = match findings(file) {
            Ok(findings) => findings,
            Err(error) => {
                failures.push(format!("{file}: {error}"));
                continue;
            }
        };
        for finding in findings {
            match EXPECTED
                .iter()
                .find(|(expected, rule, _)| expected == file && *rule == finding.rule)
            {
                Some((expected, rule, _)) => {
                    fired.insert((expected, *rule));
                }
                None => failures.push(format!("{file}: {finding}")),
            }
        }
    }

    for (file, rule, reason) in EXPECTED {
        assert!(
            files.iter().any(|f| f == file),
            "EXPECTED names {file}, which is not in the corpus"
        );
        if !fired.contains(&(*file, *rule)) {
            failures.push(format!("{file}: expected a {rule} finding ({reason}), got none"));
        }
    }

    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

#[test]
fn hands__reads_a_collection_and_a_single_hand() {
    let single = "hand: { id: one, game: holdem }\n\
                  table: { name: t, seats: 2, button: 0, stakes: { small_blind: 1.0, big_blind: 2.0 } }\n\
                  players: []\n";
    let collection = "hands:\n- hand: { id: one, game: holdem }\n  \
                      table: { name: t, seats: 2, button: 0, stakes: { small_blind: 1.0, big_blind: 2.0 } }\n  \
                      players: []\n";

    assert_eq!(hands(single).map(|hands| hands.len()), Ok(1));
    assert_eq!(hands(collection).map(|hands| hands.len()), Ok(1));
    assert!(hands("not: [a, hand").is_err());
}

#[test]
fn hands__a_file_with_no_hands_is_an_error() {
    // A clean report must mean something was checked.
    assert!(hands("hands: []\n").is_err());
}

/// `validate` coming back empty means the hand replays: bowery vets a file
/// with `validate()` and then drives `cursor()`, so the two cannot drift.
#[test]
fn corpus__a_clean_hand_has_a_cursor() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut checked = 0;
    let mut failures: Vec<String> = Vec::new();
    for file in corpus() {
        let text = std::fs::read_to_string(root.join(&file)).unwrap_or_else(|error| panic!("{file}: {error}"));
        let Ok(hands) = hands(&text) else {
            continue;
        };
        for hand in hands {
            if hand.validate().is_empty() {
                checked += 1;
                if let Err(error) = hand.cursor() {
                    failures.push(format!(
                        "{file} ({}): validates clean, but cursor() is {error:?}",
                        hand.hand.id
                    ));
                }
            }
        }
    }
    assert!(checked > 0, "no hand in the corpus validates clean");
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}
