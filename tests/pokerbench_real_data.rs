//! Real-dataset validation for the `pokerbench` loaders (EPIC-43 Phase 1).
//!
//! These tests are `#[ignore]`d: they require the actual PokerBench test splits,
//! which are not committed. Download them first (≈14 MB), then run explicitly:
//!
//! ```text
//! mkdir -p data/pokerbench && cd data/pokerbench
//! base=https://huggingface.co/datasets/RZ412/PokerBench/resolve/main
//! for f in preflop_1k_test_set_game_scenario_information.csv \
//!          preflop_1k_test_set_prompt_and_label.json \
//!          postflop_10k_test_set_game_scenario_information.csv \
//!          postflop_10k_test_set_prompt_and_label.json; do curl -sSLO "$base/$f"; done
//! cd ../..
//! cargo test --features pokerbench --test pokerbench_real_data -- --ignored --nocapture
//! ```
//!
//! Set `POKERBENCH_DATA_DIR` to override the default `./data/pokerbench`.
#![cfg(feature = "pokerbench")]

use pkcore::pokerbench::{PokerBenchScenario, PokerBenchSplit};
use std::path::{Path, PathBuf};

fn data_dir() -> PathBuf {
    std::env::var("POKERBENCH_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("data/pokerbench"))
}

fn require(path: &Path) {
    assert!(
        path.exists(),
        "missing {}; download the PokerBench test splits first (see this file's module docs)",
        path.display()
    );
}

/// Every scenario should have hole cards and a non-instruction field set.
fn assert_scenarios_sane(scenarios: &[PokerBenchScenario], split: PokerBenchSplit) {
    for (i, s) in scenarios.iter().enumerate() {
        assert_eq!(s.split, split, "scenario {i} carries the wrong split");
        assert_eq!(s.hole.len(), 2, "scenario {i} should have 2 hole cards");
        assert_eq!(s.big_blind, pkcore::pokerbench::PB_BIG_BLIND);
        if split == PokerBenchSplit::Postflop {
            assert!(!s.board.is_empty(), "post-flop scenario {i} should have a board");
        } else {
            assert!(s.board.is_empty(), "pre-flop scenario {i} should have no board");
        }
        // canonical_seating must resolve the hero to one of the seated positions.
        let seating = s.canonical_seating();
        assert!(
            seating.seats.iter().any(|seat| seat.seat == seating.hero_seat),
            "scenario {i}: hero_seat not among seats"
        );
    }
}

#[test]
#[ignore = "requires the PokerBench dataset under POKERBENCH_DATA_DIR"]
fn preflop_csv_loads_all_1000() {
    let path = data_dir().join("preflop_1k_test_set_game_scenario_information.csv");
    require(&path);
    let scenarios = PokerBenchScenario::load_csv(&path, PokerBenchSplit::Preflop)
        .expect("real preflop CSV should load without error");
    assert_eq!(scenarios.len(), 1000, "preflop test split has 1000 rows");
    assert_scenarios_sane(&scenarios, PokerBenchSplit::Preflop);
}

#[test]
#[ignore = "requires the PokerBench dataset under POKERBENCH_DATA_DIR"]
fn preflop_json_loads_all_1000() {
    let path = data_dir().join("preflop_1k_test_set_prompt_and_label.json");
    require(&path);
    let scenarios = PokerBenchScenario::load_json(&path, PokerBenchSplit::Preflop)
        .expect("real preflop JSON should load without error");
    assert_eq!(scenarios.len(), 1000);
    assert_scenarios_sane(&scenarios, PokerBenchSplit::Preflop);
    assert!(scenarios.iter().all(|s| !s.instruction.is_empty()));
}

#[test]
#[ignore = "requires the PokerBench dataset under POKERBENCH_DATA_DIR"]
fn postflop_csv_loads_all_10000() {
    let path = data_dir().join("postflop_10k_test_set_game_scenario_information.csv");
    require(&path);
    let scenarios = PokerBenchScenario::load_csv(&path, PokerBenchSplit::Postflop)
        .expect("real postflop CSV should load without error");
    assert_eq!(scenarios.len(), 10_000, "postflop test split has 10000 rows");
    assert_scenarios_sane(&scenarios, PokerBenchSplit::Postflop);
}

#[test]
#[ignore = "requires the PokerBench dataset under POKERBENCH_DATA_DIR"]
fn postflop_json_loads_all_10000() {
    let path = data_dir().join("postflop_10k_test_set_prompt_and_label.json");
    require(&path);
    let scenarios = PokerBenchScenario::load_json(&path, PokerBenchSplit::Postflop)
        .expect("real postflop JSON should load without error");
    assert_eq!(scenarios.len(), 10_000);
    assert_scenarios_sane(&scenarios, PokerBenchSplit::Postflop);
}
