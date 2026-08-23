//! Fitness evaluation: runs `SimTable` sessions and returns mean BB/100.
//!
//! The core entry point is [`evaluate`], which measures how well a given
//! [`ExploitConfig`] performs against every opponent in the field over
//! `replicates` independent sessions of `hands_per_eval` hands each.

use crate::PKError;
use crate::analysis::player_stats::StatsRegistry;
use crate::bot::decider::{BotDecider, RuleBasedDecider};
use crate::bot::exploit::ExploitConfig;
use crate::bot::exploitative_decider::ExploitativeDecider;
use crate::bot::profile::BotProfile;
use crate::bot::sim::{SimResult, SimTable};
use crate::casino::game::ForcedBets;
use crate::casino::table::{Player, Seat, Seats, Table};

const SB: usize = 50;
const BB: usize = 100;
// 1,000 BB stacks keep pot swings bounded (max BB/100 ≈ ±1,000).
// Deep-stack 1B-chip sessions produce astronomical BB/100 from single all-ins,
// which swamps the fitness signal the optimizer needs.
const STARTING_CHIPS: usize = BB * 1_000;

/// Fitness assigned to a session that produced **no valid measurement** — a sim
/// error, or a session that completed zero hands. Set far below the legitimate
/// ±~1,000 BB/100 range so the (1+λ)-ES decisively selects *against* an
/// error-prone candidate instead of retaining it as break-even (audit II.9
/// follow-up: mapping errors to `0.0` let engine failures masquerade as neutral
/// candidates). It stays finite so the per-generation mean-BB/100 diagnostic and
/// the σ-adaptation arithmetic never see `NaN`/`-inf`.
const NO_RESULT_FITNESS: f64 = -1_000_000.0;

/// A labelled opponent entry: `(display_name, profile)`.
pub type FieldEntry = (String, BotProfile);

/// Returns all eight standard opponent profiles as the default training field.
///
/// # Examples
///
/// ```
/// use pkcore::bot::training::evaluator::default_field;
///
/// let field = default_field();
/// assert_eq!(field.len(), 8);
/// assert!(field.iter().all(|(name, _)| !name.is_empty()));
/// ```
#[must_use]
pub fn default_field() -> Vec<FieldEntry> {
    BotProfile::default_profiles()
        .into_iter()
        .map(|p| (p.name.clone(), p))
        .collect()
}

/// Evaluates `config` against every entry in `field`.
///
/// Runs `replicates` sessions of `hands_per_eval` hands per opponent and
/// returns the mean BB/100 over the full matrix.  A positive return value
/// means the exploit bot profited on average.
///
/// `seed` makes the result deterministic: each (opponent, replicate) session
/// gets a distinct seed derived from `seed`, but that derivation does *not*
/// depend on `config`. Every candidate config is therefore scored on the
/// *same* hands (common random numbers), which both removes the RNG noise that
/// made training irreproducible (audit II.9) and reduces the variance the
/// optimiser sees between candidates.
///
/// # Examples
///
/// ```no_run
/// use pkcore::bot::exploit::ExploitConfig;
/// use pkcore::bot::training::evaluator::{default_field, evaluate};
///
/// let field = default_field();
/// let bb100 = evaluate(&ExploitConfig::default(), &field, 200, 1, 42);
/// // No assertion — poker variance means any value is possible in 200 hands.
/// let _ = bb100;
/// ```
// `count` is a session tally in the thousands at most.
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub fn evaluate(
    config: &ExploitConfig,
    field: &[FieldEntry],
    hands_per_eval: usize,
    replicates: usize,
    seed: u64,
) -> f64 {
    let mut total = 0.0_f64;
    let mut count = 0_usize;
    for (opp_idx, (_, opp)) in field.iter().enumerate() {
        for replicate in 0..replicates {
            total += run_session(config, opp, hands_per_eval, session_seed(seed, opp_idx, replicate));
            count += 1;
        }
    }
    if count == 0 { 0.0 } else { total / count as f64 }
}

/// Derives a deterministic per-session seed from the master `seed`.
///
/// Distinct per `(opp_idx, replicate)` so different opponents and replicates
/// play different hands, but independent of the candidate config so every
/// candidate faces an identical hand distribution (common random numbers).
fn session_seed(seed: u64, opp_idx: usize, replicate: usize) -> u64 {
    seed.wrapping_add((opp_idx as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
        .wrapping_add((replicate as u64).wrapping_mul(0xD1B5_4A32_D192_ED03))
}

/// Runs a single heads-up session of `hands` hands and returns the exploit
/// bot's BB/100 (seat 0 vs seat 1). `seed` fixes the deck shuffle and every
/// seeded decider draw, so the session is fully reproducible.
fn run_session(config: &ExploitConfig, opp_profile: &BotProfile, hands: usize, seed: u64) -> f64 {
    let exploit = Player::new_with_chips("exploit".to_string(), STARTING_CHIPS);
    let opp = Player::new_with_chips("opp".to_string(), STARTING_CHIPS);
    let seats = Seats::new(vec![Seat::new(exploit), Seat::new(opp)]);
    let table = Table::nlh_from_seats(seats, ForcedBets::new(SB, BB));

    let bots: Vec<(u8, BotProfile, Box<dyn BotDecider>)> = vec![
        (
            0,
            BotProfile::tight_aggressive(),
            Box::new(ExploitativeDecider::wrap_with_config(RuleBasedDecider, config.clone())),
        ),
        (1, opp_profile.clone(), Box::new(RuleBasedDecider)),
    ];

    let mut sim = SimTable::new_with_registry(table, bots, StatsRegistry::new()).with_seed(seed);
    session_bb100(sim.run_n_hands(hands))
}

/// Maps a session outcome to seat 0's BB/100.
///
/// A sim error or a zero-hand session yields no usable measurement; rather than
/// scoring it break-even (`0.0`, which the optimiser would retain over a
/// legitimately-losing candidate), it is logged and scored [`NO_RESULT_FITNESS`]
/// so error-prone configs are selected against (audit II.9 follow-up). A real
/// result returns the exploit bot's BB/100 for seat 0.
// Chip counts and hand counts are both far below f64's exact-integer ceiling.
#[allow(clippy::cast_precision_loss)]
fn session_bb100(outcome: Result<SimResult, PKError>) -> f64 {
    match outcome {
        Err(e) => {
            log::warn!("[pkcore::training] session errored, scoring as failure: {e:?}");
            NO_RESULT_FITNESS
        }
        Ok(result) if result.hands_played == 0 => {
            log::warn!("[pkcore::training] session completed 0 hands, scoring as failure");
            NO_RESULT_FITNESS
        }
        Ok(result) => {
            let net = result.net_chips.get(&0).copied().unwrap_or(0);
            (net as f64 / BB as f64) / result.hands_played as f64 * 100.0
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(non_snake_case)]
mod bot__training__evaluator_tests {
    use super::*;

    #[test]
    fn default_field_has_eight_archetypes() {
        let field = default_field();
        assert_eq!(field.len(), 8, "default field must contain exactly 8 profiles");
        // All names must be non-empty and distinct.
        let names: std::collections::HashSet<&str> = field.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names.len(), 8, "all field entries must have distinct names");
    }

    #[test]
    fn evaluate_returns_finite_value() {
        let field = vec![("lp".to_string(), BotProfile::loose_passive())];
        let bb100 = evaluate(&ExploitConfig::default(), &field, 100, 1, 42);
        assert!(bb100.is_finite(), "evaluate must return a finite value; got {bb100}");
    }

    #[test]
    fn evaluate_empty_field_returns_zero() {
        let bb100 = evaluate(&ExploitConfig::default(), &[], 100, 1, 42);
        assert_eq!(bb100, 0.0);
    }

    #[test]
    fn evaluate_is_deterministic_for_fixed_seed() {
        // II.9: identical (config, field, seed) → identical score.
        let field = vec![("lp".to_string(), BotProfile::loose_passive())];
        let a = evaluate(&ExploitConfig::default(), &field, 200, 2, 7);
        let b = evaluate(&ExploitConfig::default(), &field, 200, 2, 7);
        assert_eq!(a, b, "seeded evaluate must be reproducible");
    }

    // ── Audit II.9 follow-up: a failed session must not score as break-even ──

    #[test]
    fn session_bb100_errors_score_as_a_failure_not_break_even() {
        // A sim error mapped to 0.0 (break-even) would be retained by the
        // (1+λ)-ES over a legitimately-losing candidate. It must score as a
        // decisive failure instead.
        let score = session_bb100(Err(crate::PKError::InvalidAction));
        assert_eq!(NO_RESULT_FITNESS, score);
        assert!(score < -1_000.0, "an errored session must sort below any real BB/100");
    }

    #[test]
    fn session_bb100_zero_hands_scores_as_a_failure() {
        // A session that completes no hands is as uninformative as an error.
        let score = session_bb100(Ok(crate::bot::sim::SimResult::default()));
        assert_eq!(NO_RESULT_FITNESS, score);
    }

    #[test]
    fn session_bb100_computes_real_bb100_for_seat_zero() {
        // Seat 0 net +10 BB over 100 hands → +10 BB/100.
        let mut net = std::collections::HashMap::new();
        net.insert(0u8, (BB as i64) * 10);
        let result = crate::bot::sim::SimResult {
            hands_played: 100,
            net_chips: net,
            ..Default::default()
        };
        assert!((session_bb100(Ok(result)) - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn session_seed_is_distinct_per_opponent_and_replicate() {
        let s0 = session_seed(42, 0, 0);
        let s1 = session_seed(42, 1, 0);
        let s2 = session_seed(42, 0, 1);
        assert_ne!(s0, s1);
        assert_ne!(s0, s2);
        assert_ne!(s1, s2);
    }
}
