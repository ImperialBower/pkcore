//! Preflop equity from precomputed charts and ranges (EPIC-39 Phase 4).
//!
//! Preflop, the decider's hand strength is historically a **coin flip**: it
//! rolls against the hand's frequency in an opening range and returns `1.0` or
//! `0.0`. The `preflop_charts` knob replaces that with a real number.

#[cfg(feature = "hup-charts")]
use crate::analysis::gto::combos::Combos;
#[cfg(feature = "hup-charts")]
use crate::analysis::gto::twos::Twos;
#[cfg(feature = "hup-charts")]
use crate::analysis::store::db::hup::HUPResult;
use crate::arrays::two::Two;
#[cfg(feature = "hup-charts")]
use crate::bard::Bard;
use crate::bot::decision_config::PreflopCharts;
use crate::bot::profile::BotProfile;
#[cfg(feature = "hup-charts")]
use crate::bot::range_model::villain_range;
use crate::bot::table_snapshot::TableSnapshot;
use std::str::FromStr;

/// Exact heads-up preflop equity for `hero` against every hand in `range`.
///
/// Each matchup is read from the embedded HUP table, which enumerates all
/// `C(48,5)` boards, so the per-matchup number is exact rather than sampled.
/// Hands sharing a card with the hero are skipped, and the surviving matchups
/// are averaged with equal weight. Returns `None` when nothing survives.
///
/// # The perspective trap
///
/// [`HUPResult::lookup`] sorts its two arguments into a `SortedHeadsUp` and
/// **ignores the order they were passed in**: `lookup(aces, kings)` and
/// `lookup(kings, aces)` return the same record, and `odds.wins` always
/// belongs to the *higher* hand. This function compares `result.higher`
/// against the hero's [`Bard`] and flips when they differ.
///
/// # Examples
///
/// ```
/// use pkcore::analysis::gto::combos::Combos;
/// use pkcore::arrays::two::Two;
/// use pkcore::bot::preflop_equity::hup_equity_vs_range;
/// use std::str::FromStr;
///
/// let aces = Two::from_str("AS AD").unwrap();
/// let kings = Combos::from_str("KK").unwrap();
/// let equity = hup_equity_vs_range(&aces, &kings).unwrap();
/// assert!((equity - 0.8195).abs() < 0.001);
/// ```
#[cfg(feature = "hup-charts")]
#[must_use]
pub fn hup_equity_vs_range(hero: &Two, range: &Combos) -> Option<f64> {
    let hero_bard: Bard = (*hero).into();
    let mut total = 0.0;
    let mut counted = 0_u32;

    for villain in Twos::from(range).to_vec() {
        if shares_a_card(*hero, villain) {
            continue;
        }
        let Ok(result) = HUPResult::lookup(hero, &villain) else {
            continue;
        };
        let odds = if result.higher == hero_bard {
            result.odds
        } else {
            result.flip_mode().odds
        };
        let boards = odds.wins + odds.losses + odds.draws;
        if boards == 0 {
            continue;
        }
        #[allow(clippy::cast_precision_loss)]
        {
            total += (odds.wins as f64 + odds.draws as f64 / 2.0) / boards as f64;
        }
        counted += 1;
    }

    if counted == 0 {
        return None;
    }
    Some(total / f64::from(counted))
}

/// Returns `true` when the two hands cannot be dealt at the same time.
#[cfg(feature = "hup-charts")]
fn shares_a_card(hero: Two, villain: Two) -> bool {
    let (first, second) = (hero.first(), hero.second());
    villain.first() == first || villain.first() == second || villain.second() == first || villain.second() == second
}

/// Preflop equity for the hero, per the profile's `preflop_charts` knob.
///
/// `None` means "no chart answer" and the caller should keep its historical
/// frequency roll. That is the answer whenever the knob is `Off`, the hero is
/// not holding exactly two cards, the hand is not preflop, no villain range
/// resolves, or — for [`PreflopCharts::Hup`] — the pot is not heads-up, since
/// the embedded table is strictly two-handed.
///
/// # Examples
///
/// ```
/// use pkcore::bot::decision_config::PreflopCharts;
/// use pkcore::bot::preflop_equity::preflop_equity;
/// use pkcore::bot::profile::BotProfile;
/// use pkcore::bot::table_snapshot::TableSnapshot;
/// use pkcore::casino::game::ForcedBets;
/// use pkcore::casino::table::{Player, Seat, Seats, Table};
/// use rand::SeedableRng;
/// use rand::rngs::SmallRng;
///
/// let seats = Seats::new(vec![
///     Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
///     Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
/// ]);
/// let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
/// let state = TableSnapshot::from_table(&table, 0);
/// let mut rng = SmallRng::seed_from_u64(1);
///
/// // The default profile leaves the knob off, so there is no chart answer.
/// assert!(preflop_equity(&BotProfile::gto(), &state, &mut rng).is_none());
/// ```
#[must_use]
pub fn preflop_equity<R: rand::Rng + ?Sized>(profile: &BotProfile, state: &TableSnapshot, rng: &mut R) -> Option<f64> {
    if !state.phase.is_preflop() || state.hole_cards.len() != 2 {
        return None;
    }
    let hero = Two::from_str(&state.hole_cards.to_string()).ok()?;

    match profile.decision.preflop_charts {
        PreflopCharts::Off => None,
        PreflopCharts::Hup => hup_chart_equity(profile, state, hero),
        PreflopCharts::Solver => solver_equity(profile, state, hero, rng),
    }
}

/// The chart path: exact, but strictly heads-up.
#[cfg(feature = "hup-charts")]
fn hup_chart_equity(profile: &BotProfile, state: &TableSnapshot, hero: Two) -> Option<f64> {
    let mut villains = active_villain_indices(state);
    let only = villains.pop().filter(|_| villains.is_empty())?;
    let range = villain_range(profile, state, only)?;
    hup_equity_vs_range(&hero, &range)
}

/// Without `hup-charts` the 15.8 MB chart is not linked, so there is no answer
/// to give and the decider falls back to the preflop frequency roll. This is a
/// build-time choice by the consumer, documented on the feature in `Cargo.toml`;
/// `preflop_charts: solver` is the multi-way path and needs no chart.
#[cfg(not(feature = "hup-charts"))]
fn hup_chart_equity(_profile: &BotProfile, _state: &TableSnapshot, _hero: Two) -> Option<f64> {
    None
}

/// Indices into [`TableSnapshot::stacks`] for every active villain.
#[cfg(feature = "hup-charts")]
fn active_villain_indices(state: &TableSnapshot) -> Vec<usize> {
    state
        .stacks
        .iter()
        .enumerate()
        .filter(|(_, villain)| villain.is_active && villain.seat != state.seat)
        .map(|(index, _)| index)
        .collect()
}

/// The Monte Carlo budget for the `Solver` path, taken from the `equity` knob.
///
/// Preflop has no board, so a request left on `EquityOptions::default()` runs
/// `max_samples` of **25,000** — 50x a profile asking for 500, on the most
/// frequent decision in a hand. The `equity` knob is where a profile states
/// what it is willing to spend, so the preflop path spends the same, exactly as
/// `real_equity` does postflop. `Off` still needs a number, since `Solver` runs
/// without the engine knob: it gets `DecisionConfig`'s own default.
///
/// See EPIC-39 corrigendum 18.
#[cfg(feature = "equity")]
fn solver_sample_budget(profile: &BotProfile) -> u64 {
    use crate::bot::decision_config::EquityMode;
    match profile.decision.equity {
        EquityMode::Fast { samples } => u64::from(samples),
        EquityMode::Exact => SOLVER_EXACT_SAMPLES,
        EquityMode::Off => SOLVER_DEFAULT_SAMPLES,
    }
}

/// Budget for `equity: exact`, matching `decider::EXACT_EQUITY_SAMPLES`.
#[cfg(feature = "equity")]
const SOLVER_EXACT_SAMPLES: u64 = 100_000;

/// Budget when `equity` is `off` but `preflop_charts` is `solver`, matching
/// `decision_config`'s own default sample count.
#[cfg(feature = "equity")]
const SOLVER_DEFAULT_SAMPLES: u64 = 2_000;

/// Sampled preflop equity against the villain ranges, for any table size.
///
/// Unlike the HUP chart this works multi-way, at the cost of Monte Carlo error.
#[cfg(feature = "equity")]
fn solver_equity<R: rand::Rng + ?Sized>(
    profile: &BotProfile,
    state: &TableSnapshot,
    hero: Two,
    rng: &mut R,
) -> Option<f64> {
    use crate::analysis::equity::{EquityOptions, EquityRequest, PlayerSpec};
    use crate::play::board::Board;

    let mut players = vec![PlayerSpec::Exact(hero)];
    players.extend(crate::bot::range_model::villain_range_specs(profile, state));
    if !(2..=10).contains(&players.len()) {
        return None;
    }
    let opts = EquityOptions {
        max_samples: solver_sample_budget(profile),
        seed: Some(rng.random::<u64>()),
        ..EquityOptions::default()
    };
    let report = EquityRequest {
        players,
        board: Board::default(),
        opts,
    }
    .compute()
    .ok()?;
    report.players.first().map(|player| player.equity)
}

/// Feature-off stub: without the `equity` feature there is no engine to sample,
/// so `Solver` falls back to the historical frequency roll.
#[cfg(not(feature = "equity"))]
fn solver_equity<R: rand::Rng + ?Sized>(
    _profile: &BotProfile,
    _state: &TableSnapshot,
    _hero: Two,
    _rng: &mut R,
) -> Option<f64> {
    None
}

#[cfg(test)]
#[allow(non_snake_case)]
mod bot__preflop_equity_tests {
    use super::*;
    use crate::bot::decision_config::PreflopCharts;
    use crate::bot::playbook::Playbook;
    use crate::cards::Cards;
    use crate::casino::game::ForcedBets;
    use crate::casino::table::{Player, Seat, Seats, Table};
    use crate::games::GamePhase;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use std::str::FromStr;

    #[allow(dead_code)]
    fn two(s: &str) -> Two {
        Two::from_str(s).expect("a legal two-card hand")
    }

    #[cfg(feature = "hup-charts")]
    fn range(s: &str) -> Combos {
        Combos::from_str(s).expect("a legal range")
    }

    fn snapshot(players: usize, hero_seat: u8, hole: &str) -> TableSnapshot<'static> {
        let seats = Seats::new(
            (0..players)
                .map(|i| Seat::new(Player::new_with_chips(format!("P{i}"), 1_000)))
                .collect(),
        );
        let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
        let mut state = TableSnapshot::from_table(&table, hero_seat);
        state.phase = GamePhase::BettingPreFlop;
        state.hole_cards = Cards::from_str(hole).expect("legal hole cards");
        state
    }

    fn profile_with(charts: PreflopCharts) -> BotProfile {
        let mut profile = BotProfile::gto().with_playbook(Playbook::gto());
        profile.decision.preflop_charts = charts;
        profile
    }

    /// Aces over kings is the most-quoted preflop matchup there is: 81.95%
    /// averaged over all six king combinations. (A *single* suit pairing such
    /// as A♠A♦ vs K♥K♣ is 81.06% — averaging the six is what gives 0.8195.)
    /// Cross-checked against the equity engine, an independent code path,
    /// which returns 0.8195 on three separate seeds.
    #[cfg(feature = "hup-charts")]
    #[test]
    fn hup_equity_matches_the_known_aces_versus_kings_number() {
        let equity = hup_equity_vs_range(&two("AS AD"), &range("KK")).expect("KK is in the table");
        assert!(
            (equity - 0.8195).abs() < 0.001,
            "AA over KK should be ~0.8195, got {equity:.4}"
        );
    }

    /// The load-bearing test. `HUPResult::lookup` ignores argument order and
    /// always reports from the *higher* hand's side, so a missing perspective
    /// flip inverts every answer. Two complementary equities must sum to one.
    #[cfg(feature = "hup-charts")]
    #[test]
    fn hup_equity_is_symmetric_between_the_two_sides() {
        let hero = hup_equity_vs_range(&two("AS AD"), &range("KK")).expect("a result");
        let villain = hup_equity_vs_range(&two("KH KC"), &range("AA")).expect("a result");
        assert!(
            (hero + villain - 1.0).abs() < 1e-9,
            "complementary equities must sum to 1.0, got {hero:.6} + {villain:.6}"
        );
    }

    #[cfg(feature = "hup-charts")]
    #[test]
    fn hup_equity_skips_hands_that_share_a_card_with_the_hero() {
        // Only A♥A♣ avoids the hero's A♠ and A♦, and aces against aces chop.
        let equity = hup_equity_vs_range(&two("AS AD"), &range("AA")).expect("one hand survives");
        assert!(
            (equity - 0.5).abs() < 0.02,
            "aces against aces is a chop, got {equity:.4}"
        );
    }

    #[cfg(feature = "hup-charts")]
    #[test]
    fn hup_equity_is_none_for_an_empty_range() {
        assert!(hup_equity_vs_range(&two("AS AD"), &Combos::default()).is_none());
    }

    /// The regression for the 50x over-budget preflop request. A profile states
    /// its Monte Carlo budget on the `equity` knob; the `Solver` path used to
    /// ignore it and take `EquityOptions::default()`'s 25,000 samples on the
    /// most frequent decision in a hand.
    #[cfg(feature = "equity")]
    #[test]
    fn solver_budget_comes_from_the_equity_knob() {
        use crate::bot::decision_config::EquityMode;

        let mut fast = BotProfile::gto();
        fast.decision.equity = EquityMode::Fast { samples: 500 };
        assert_eq!(500, solver_sample_budget(&fast), "a profile asking for 500 gets 500");

        let mut exact = BotProfile::gto();
        exact.decision.equity = EquityMode::Exact;
        assert_eq!(SOLVER_EXACT_SAMPLES, solver_sample_budget(&exact));

        let mut off = BotProfile::gto();
        off.decision.equity = EquityMode::Off;
        assert_eq!(SOLVER_DEFAULT_SAMPLES, solver_sample_budget(&off));

        assert!(
            solver_sample_budget(&fast) < 25_000,
            "the point of the fix: a browser budget must not silently become the engine default"
        );
    }

    #[test]
    fn preflop_equity_is_none_when_the_knob_is_off() {
        let state = snapshot(2, 0, "AS AD");
        let mut rng = SmallRng::seed_from_u64(1);
        assert!(preflop_equity(&profile_with(PreflopCharts::Off), &state, &mut rng).is_none());
    }

    #[test]
    fn preflop_equity_hup_is_none_when_more_than_one_villain_is_active() {
        let state = snapshot(6, 0, "AS AD");
        let mut rng = SmallRng::seed_from_u64(1);
        assert!(
            preflop_equity(&profile_with(PreflopCharts::Hup), &state, &mut rng).is_none(),
            "the HUP table is strictly heads-up"
        );
    }

    #[cfg(feature = "hup-charts")]
    #[test]
    fn preflop_equity_hup_prices_aces_far_above_a_coin_flip() {
        let state = snapshot(2, 0, "AS AD");
        let mut rng = SmallRng::seed_from_u64(1);
        let equity = preflop_equity(&profile_with(PreflopCharts::Hup), &state, &mut rng)
            .expect("heads-up aces resolve through the chart");
        assert!(
            equity > 0.7,
            "aces against any opening range should be well above 0.7, got {equity:.4}"
        );
    }
    /// The HUP chart is strictly two-handed, so `Solver` is the multi-way
    /// answer: sampled equity against every villain's range.
    #[cfg(feature = "equity")]
    #[test]
    fn preflop_equity_solver_answers_multiway_where_hup_cannot() {
        let state = snapshot(6, 0, "AS AD");
        let mut rng = SmallRng::seed_from_u64(5);
        assert!(
            preflop_equity(&profile_with(PreflopCharts::Hup), &state, &mut rng).is_none(),
            "the chart declines a six-handed pot"
        );
        let equity = preflop_equity(&profile_with(PreflopCharts::Solver), &state, &mut rng)
            .expect("the engine answers any table size");
        assert!(
            equity > 0.3 && equity < 0.8,
            "aces against five opening ranges sit well inside the middle, got {equity:.4}"
        );
    }

    /// Without `hup-charts` the chart is not linked, so the exact path has no
    /// answer to give and the decider falls back to the preflop frequency roll.
    /// The consumer chose this in their manifest; see the feature's `Cargo.toml`
    /// note and EPIC-39 corrigendum 16.
    #[cfg(not(feature = "hup-charts"))]
    #[test]
    fn preflop_equity_hup_is_none_without_the_chart_feature() {
        let state = snapshot(2, 0, "AS AD");
        let mut rng = SmallRng::seed_from_u64(1);
        assert!(
            preflop_equity(&profile_with(PreflopCharts::Hup), &state, &mut rng).is_none(),
            "no chart linked means no chart answer"
        );
    }
}
