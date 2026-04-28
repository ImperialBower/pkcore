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

const SESSION_YAML: &str = include_str!("../data/hands/pkarena0-session_2026-04-15.yaml");

/// Session captured by a user reporting "GTO bot is out of chips but keeps
/// playing". 56 hands, blinds escalate from 50/100 → 400/800, gto's stack
/// drops to 100 chips for the final hand and goes all-in for less than the SB.
const SESSION_2026_04_28_YAML: &str = include_str!("../data/hands/pkarena0-session_2026-04-28.yaml");

fn load_session() -> HandCollection {
    HandCollection::from_yaml(SESSION_YAML).expect("fixture YAML should parse")
}

fn load_session_2026_04_28() -> HandCollection {
    HandCollection::from_yaml(SESSION_2026_04_28_YAML).expect("fixture YAML should parse")
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
        let replay =
            result.unwrap_or_else(|e| panic!("replay() failed for hand {}: {e}", collection.hands()[idx].hand.id));
        assert!(
            replay.is_consistent,
            "replay mismatch for hand {}: final stacks = {:?}",
            collection.hands()[idx].hand.id,
            replay.final_stacks,
        );
    }
}

// ── 2026-04-28 session: stakes-mid-hand defect captured ────────────────────
//
// User-reported defect: "I am seeing errors during this game with the GTO bot.
// It's out of chips but keeps playing."
//
// Investigation revealed that the YAML records `stakes` taken from the table's
// **current** `ForcedBets` at hand-end, while the action log preserves the
// blinds that were actually posted. If `set_blinds` is invoked while a hand is
// in flight (or after it ends but before `next_hand()` runs in pkarena0-web),
// the captured stakes drift from the actual posts. Replaying a drift-affected
// hand fails with `InsufficientIncrement` because the replay engine seeds its
// `min_raise()` from the (new) BB while the recorded raises only satisfy the
// (old) BB increment.
//
// The chip-conservation check still applies — net amounts in `results` come
// from observed stack deltas, not from `stakes` — so this test guards the
// *audit invariant* that survives even when blinds are reported wrong. The
// fix lives in pkarena0-web (defer `set_blinds` mid-hand); see
// `casino::session::PokerSession::set_blinds` for the deferral primitive.

#[test]
fn session_2026_04_28_all_nets_sum_to_zero() {
    let collection = load_session_2026_04_28();
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

/// Structural assertion: every hand's recorded `stakes` must match its
/// SB/BB `Post` amounts, *unless* the under-posting blinder had insufficient
/// chips to cover the full blind (legitimate short-stack all-in).
///
/// A drift signature — both SB and BB posts under their recorded stakes
/// while both blinders had ample chips — almost always means `set_blinds`
/// was called while the hand was in flight. The 2026-04-28 fixture has this
/// drift in 5 hands (the boundary hand of every blind-level bump). Future
/// captures from pkarena0-web ≥ 0.1.6 (the deferral fix) should not
/// reproduce this.
///
/// The fixture is left as-is rather than being post-edited so the test
/// continues to document the data-shape pkarena0-web produced before the
/// fix. The stack-aware exception lets the test pass for legitimate
/// short-stack all-in blinds (which the same fixture also contains).
///
/// `#[ignore]` because the 2026-04-28 fixture is known-bad (5 drift hands).
/// Re-enable on a fresh capture from a fixed pkarena0-web.
#[test]
#[ignore = "2026-04-28 fixture has known pre-fix drift; see PokerSession::set_blinds"]
fn session_2026_04_28_stakes_match_post_amounts() {
    use pkcore::hand_history::ActionType;

    let collection = load_session_2026_04_28();
    for hand in collection.hands() {
        let Some(streets) = &hand.streets else {
            continue;
        };
        let Some(preflop) = &streets.preflop else {
            continue;
        };
        let post_actions: Vec<(u8, f64)> = preflop
            .actions
            .iter()
            .filter(|a| a.action == ActionType::Post)
            .filter_map(|a| a.amount.map(|amt| (a.seat, amt)))
            .collect();

        if post_actions.len() < 2 {
            continue;
        }

        let recorded_sb = hand.table.stakes.small_blind;
        let recorded_bb = hand.table.stakes.big_blind;
        let (sb_seat, sb_post) = post_actions[0];
        let (bb_seat, bb_post) = post_actions[1];

        let stack_for = |seat: u8| -> Option<f64> { hand.players.iter().find(|p| p.seat == seat).map(|p| p.stack) };

        let sb_could_post = stack_for(sb_seat).is_none_or(|s| s >= recorded_sb - 0.01);
        let bb_could_post = stack_for(bb_seat).is_none_or(|s| s >= recorded_bb - 0.01);
        let sb_under = sb_post < recorded_sb - 0.01;
        let bb_under = bb_post < recorded_bb - 0.01;

        // Both posts under-paid AND both blinders could have covered = drift.
        // A single under-paid post with that blinder short on chips = legit.
        if sb_under && bb_under && sb_could_post && bb_could_post {
            panic!(
                "{}: stakes-vs-posts drift detected (recorded {recorded_sb}/{recorded_bb}, \
                 posted {sb_post}/{bb_post}). \
                 Likely cause: set_blinds called during this hand in pkarena0-web.",
                hand.hand.id,
            );
        }
    }
}

/// Diagnostic helper: prints every hand whose recorded stakes don't match its
/// post amounts, and the first replay error each one would produce. Run via
/// `cargo test list_drift_hands --features bot-profiles -- --nocapture --ignored`.
#[test]
#[ignore = "diagnostic — prints drift report"]
fn list_drift_hands() {
    use pkcore::hand_history::ActionType;
    let collection = load_session_2026_04_28();
    for hand in collection.hands() {
        let Some(streets) = &hand.streets else { continue };
        let Some(preflop) = &streets.preflop else { continue };
        let posts: Vec<f64> = preflop
            .actions
            .iter()
            .filter(|a| a.action == ActionType::Post)
            .filter_map(|a| a.amount)
            .collect();
        if posts.len() < 2 {
            continue;
        }
        if (posts[0] - hand.table.stakes.small_blind).abs() > 0.01
            || (posts[1] - hand.table.stakes.big_blind).abs() > 0.01
        {
            println!(
                "DRIFT {}: stakes={}/{}, posts={}/{}",
                hand.hand.id, hand.table.stakes.small_blind, hand.table.stakes.big_blind, posts[0], posts[1],
            );
        }
    }
}
