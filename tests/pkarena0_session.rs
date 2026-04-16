//! Regression tests for the pkarena0 session fixture.
//!
//! Uses `data/hands/pkarena0-session_2026-04-15.yaml` — a corrected copy of an
//! actual game session — to validate three properties:
//!
//! 1. Chip conservation: net outcomes sum to zero for every hand.
//! 2. Pot completeness: `pot_won` amounts sum to the final street pot.
//! 3. Replay consistency: every hand replays without error and final stacks match.
//!
//! Test 3 is the **regression gate** for the uncalled-turn-bet bug
//! (fixed in `SeatsNoCell::bring_it_in`). If that bug returns, the engine sets
//! the lone non-all-in player to `YetToAct` at the start of the turn; `replay()`
//! then calls `bring_it_in()` which fails with `PKError::ActionIsntFinished`
//! because there is no recorded action in the turn to resolve that state.

use pkcore::hand_history::HandCollection;

const SESSION_YAML: &str =
    include_str!("../data/hands/pkarena0-session_2026-04-15.yaml");

fn load_session() -> HandCollection {
    HandCollection::from_yaml(SESSION_YAML).expect("fixture YAML should parse")
}

#[test]
fn all_nets_sum_to_zero() {
    let collection = load_session();
    for hand in collection.hands() {
        let net_sum: f64 = hand
            .results
            .as_ref()
            .map_or(0.0, |r| r.iter().filter_map(|e| e.net).sum());
        assert!(
            net_sum.abs() < 0.01,
            "chips not conserved in hand {}: net sum = {net_sum}",
            hand.hand.id,
        );
    }
}

#[test]
fn pot_won_matches_final_pot() {
    let collection = load_session();
    for hand in collection.hands() {
        let Some(ref streets) = hand.streets else {
            continue;
        };
        // The final pot is the last street that has one recorded.
        let final_pot = streets
            .river
            .as_ref()
            .and_then(|s| s.pot)
            .or_else(|| streets.turn.as_ref().and_then(|s| s.pot))
            .or_else(|| streets.flop.as_ref().and_then(|s| s.pot))
            .or_else(|| streets.preflop.as_ref().and_then(|s| s.pot));

        let Some(expected_pot) = final_pot else {
            continue;
        };

        let pot_won_sum: f64 = hand
            .results
            .as_ref()
            .map_or(0.0, |r| r.iter().filter_map(|e| e.pot_won).sum());

        assert!(
            (pot_won_sum - expected_pot).abs() < 0.01,
            "pot_won sum ({pot_won_sum}) ≠ final pot ({expected_pot}) in hand {}",
            hand.hand.id,
        );
    }
}

/// Regression gate: replaying every hand through the engine must succeed
/// and produce chip counts consistent with the recorded results.
///
/// If the uncalled-turn-bet bug returns, this test will error with
/// `bring_it_in: ActionIsntFinished` rather than an assertion failure.
#[test]
fn all_hands_replay_consistently() {
    let collection = load_session();
    for (idx, result) in collection.replay_all().into_iter().enumerate() {
        let replay = result.unwrap_or_else(|e| {
            panic!(
                "replay() failed for hand {}: {e}",
                collection.hands()[idx].hand.id
            )
        });
        assert!(
            replay.is_consistent,
            "replay mismatch for hand {}: final stacks = {:?}",
            collection.hands()[idx].hand.id,
            replay.final_stacks,
        );
    }
}
