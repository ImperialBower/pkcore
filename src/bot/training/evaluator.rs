//! Fitness evaluation: runs `SimTable` sessions and returns mean BB/100.
//!
//! The core entry point is [`evaluate`], which measures how well a given
//! [`ExploitConfig`] performs against every opponent in the field over
//! `replicates` independent sessions of `hands_per_eval` hands each.

use crate::analysis::player_stats::StatsRegistry;
use crate::bot::decider::{BotDecider, RuleBasedDecider};
use crate::bot::exploit::ExploitConfig;
use crate::bot::exploitative_decider::ExploitativeDecider;
use crate::bot::profile::BotProfile;
use crate::bot::sim::SimTable;
use crate::casino::game::ForcedBets;
use crate::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};

const SB: usize = 50;
const BB: usize = 100;
// 1,000 BB stacks keep pot swings bounded (max BB/100 ≈ ±1,000).
// Deep-stack 1B-chip sessions produce astronomical BB/100 from single all-ins,
// which swamps the fitness signal the optimizer needs.
const STARTING_CHIPS: usize = BB * 1_000;

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
/// # Examples
///
/// ```no_run
/// use pkcore::bot::exploit::ExploitConfig;
/// use pkcore::bot::training::evaluator::{default_field, evaluate};
///
/// let field = default_field();
/// let bb100 = evaluate(&ExploitConfig::default(), &field, 200, 1);
/// // No assertion — poker variance means any value is possible in 200 hands.
/// let _ = bb100;
/// ```
#[must_use]
pub fn evaluate(config: &ExploitConfig, field: &[FieldEntry], hands_per_eval: usize, replicates: usize) -> f64 {
    let mut total = 0.0_f64;
    let mut count = 0_usize;
    for (_, opp) in field {
        for _ in 0..replicates {
            total += run_session(config, opp, hands_per_eval);
            count += 1;
        }
    }
    if count == 0 { 0.0 } else { total / count as f64 }
}

/// Runs a single heads-up session of `hands` hands and returns the exploit
/// bot's BB/100 (seat 0 vs seat 1).
fn run_session(config: &ExploitConfig, opp_profile: &BotProfile, hands: usize) -> f64 {
    let exploit = PlayerNoCell::new_with_chips("exploit".to_string(), STARTING_CHIPS);
    let opp = PlayerNoCell::new_with_chips("opp".to_string(), STARTING_CHIPS);
    let seats = SeatsNoCell::new(vec![SeatNoCell::new(exploit), SeatNoCell::new(opp)]);
    let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(SB, BB));

    let bots: Vec<(u8, BotProfile, Box<dyn BotDecider>)> = vec![
        (
            0,
            BotProfile::tight_aggressive(),
            Box::new(ExploitativeDecider::wrap_with_config(RuleBasedDecider, config.clone())),
        ),
        (1, opp_profile.clone(), Box::new(RuleBasedDecider)),
    ];

    let mut sim = SimTable::new_with_registry(table, bots, StatsRegistry::new());
    match sim.run_n_hands(hands) {
        Err(_) => 0.0,
        Ok(result) if result.hands_played == 0 => 0.0,
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
        let bb100 = evaluate(&ExploitConfig::default(), &field, 100, 1);
        assert!(bb100.is_finite(), "evaluate must return a finite value; got {bb100}");
    }

    #[test]
    fn evaluate_empty_field_returns_zero() {
        let bb100 = evaluate(&ExploitConfig::default(), &[], 100, 1);
        assert_eq!(bb100, 0.0);
    }
}
