//! Two probabilities at one table.
//!
//! Companion to the essay *Two Probabilities at One Table: Bayesian and
//! Frequentist Perspectives on Poker and pkcore*. It replays "The Hand"
//! (Negreanu vs. Hansen, High Stakes Poker S2E11) through both lenses:
//!
//! 1. **The frequentist table** counts the deck: exact heads-up odds from the
//!    embedded chart, Monte Carlo estimates with their error bars, exact
//!    runout enumeration on each street, and the long-run EV of a call.
//! 2. **The Bayesian table** tracks Daniel's belief about Gus's cards: a
//!    position chart as the prior, blockers as conditioning, and Bayes' rule
//!    applied to each of Gus's actions, once by hand and once with the solver.
//! 3. **Where they meet**: Daniel's full house against the posterior, with
//!    and without the weights, and the EV of his call under two reads.
//!
//! ```text
//! cargo run --release --features full --example two_tables
//! ```
//!
//! Gus Hansen holds 5♦ 5♣, Daniel Negreanu holds 6♠ 6♥, and the board runs
//! out 9♣ 6♦ 5♥ 5♠ 8♠. See `examples/the_hand.rs` for the betting replay.

// Probabilities are lossy ratios of counts; the casts here are deliberate.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use pkcore::PKError;
use pkcore::Pile;
use pkcore::analysis::equity::{EquityOptions, EquityRequest, Method, PlayerSpec};
use pkcore::analysis::ev::Ev;
use pkcore::analysis::gto::combo::Combo;
use pkcore::analysis::gto::combos::Combos;
use pkcore::analysis::gto::game_tree::{Action, GameTree, Node};
use pkcore::analysis::gto::odds::WinLoseDraw;
use pkcore::analysis::gto::solver::Solver;
use pkcore::analysis::gto::solver_config::SolverConfig;
use pkcore::analysis::gto::twos::Twos;
use pkcore::analysis::gto::vs::Versus;
use pkcore::analysis::gto::weighted_combos::WeightedCombos;
use pkcore::analysis::name::HandRankName;
use pkcore::analysis::pot_odds::PotOdds;
use pkcore::analysis::store::db::hup::HUPResult;
use pkcore::arrays::HandRanker;
use pkcore::arrays::five::Five;
use pkcore::arrays::seven::Seven;
use pkcore::arrays::six::Six;
use pkcore::arrays::two::Two;
use pkcore::bot::position_ranges::PositionRanges;
use pkcore::bot::weighted_range::WeightedRange;
use pkcore::cards::Cards;
use pkcore::casino::position::Position;
use pkcore::play::board::Board;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Instant;

// ── The Hand ──────────────────────────────────────────────────────────────────

const GUS: Two = Two::HAND_5D_5C;
const DANIEL: Two = Two::HAND_6S_6H;
const BOARD: &str = "9♣ 6♦ 5♥ 5♠ 8♠";

/// Chips in the middle when the river is dealt (blinds, preflop, flop, turn).
const POT_AT_RIVER: u64 = 110_150;
/// Daniel's river bet after Gus checks.
const DANIEL_RIVER_BET: u64 = 65_000;
/// What each player still had behind on the river.
const STACK_AT_RIVER: u64 = 945_000;

/// Scale used to turn a belief into counts for [`Ev`], which wants integers.
const BELIEF_SCALE: f64 = 10_000.0;

fn main() -> Result<(), PKError> {
    let board = Board::from_str(BOARD)?;
    let clock = Instant::now();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║   Two Probabilities at One Table — The Hand, both ways       ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!("Gus Hansen       {GUS}");
    println!("Daniel Negreanu  {DANIEL}");
    println!("Board            {}", board.cards());

    frequentist_table(&board)?;
    let (before_shove, posterior) = bayesian_table(&board)?;
    where_they_meet(&board, &before_shove, &posterior)?;

    println!("\nDone in {:.1?}.", clock.elapsed());
    Ok(())
}

fn section(title: &str) {
    println!();
    println!("━━━ {title} {}", "━".repeat(60usize.saturating_sub(title.len() + 5)));
}

// ── 1. The frequentist table ──────────────────────────────────────────────────

/// Counting the deck: every answer here is a fraction of equally likely deals.
fn frequentist_table(board: &Board) -> Result<(), PKError> {
    section("1. The frequentist table: counting the deck");
    preflop_exact_versus_sampled()?;
    exact_by_street(board)?;
    river_pot_odds(board)?;
    Ok(())
}

/// Heads-up preflop has C(48,5) = 1,712,304 boards. The embedded chart holds
/// the exact count; Monte Carlo estimates it with an error bar that shrinks
/// as 1/sqrt(n).
fn preflop_exact_versus_sampled() -> Result<(), PKError> {
    let exact = HUPResult::lookup(&DANIEL, &GUS)?;
    // The chart is from the higher hand's side; Gus's equity is his wins plus half the ties.
    let total = exact.odds.total() as f64;
    let gus_exact = (exact.odds.losses as f64 + exact.odds.draws as f64 / 2.0) / total;
    println!(
        "\nPreflop, 55 vs 66, every one of {} runouts counted:",
        exact.odds.total()
    );
    println!("  {exact}");
    println!(
        "  Gus's equity is {:.2}% with no error bar: nothing was estimated.",
        gus_exact * 100.0
    );

    println!("\nThe same question by sampling. Error falls as 1/sqrt(n):");
    println!(
        "  {:>9}  {:>12}  {:>9}  {:>10}  {:>10}",
        "samples", "seed", "estimate", "actual err", "std error"
    );
    // The engine derives each sample's seed as `seed ^ index`, so seeds that
    // differ only in their low bits draw almost the same samples. Spread them.
    for samples in [2_500_u64, 25_000, 100_000] {
        for seed in [7_u64, 7 << 24, 7 << 48] {
            let request = EquityRequest {
                players: vec![PlayerSpec::Exact(GUS), PlayerSpec::Exact(DANIEL)],
                board: Board::default(),
                opts: EquityOptions {
                    max_samples: samples,
                    seed: Some(seed),
                    ..Default::default()
                },
            };
            let report = request.compute()?;
            debug_assert_eq!(Method::MonteCarlo, report.method);
            let estimate = report.players[0].equity;
            println!(
                "  {:>9}  {:>12}  {:>8.2}%  {:>+9.2}pp  {:>9.2}pp",
                report.samples,
                seed,
                estimate * 100.0,
                (estimate - gus_exact) * 100.0,
                standard_error(estimate, report.samples) * 100.0,
            );
        }
    }
    println!("  A 95% interval is about ±2 standard errors. The default 25,000");
    println!("  samples is honest to the whole percent, and no further.");
    Ok(())
}

/// Once the board starts, the runout space is small enough to enumerate.
fn exact_by_street(board: &Board) -> Result<(), PKError> {
    println!("\nStreet by street the space shrinks and the count becomes exact:");
    println!(
        "  {:<7}  {:<16}  {:>8}  {:>10}  {:>8}  {:>8}",
        "street", "board", "runouts", "method", "Gus", "Daniel"
    );
    let streets = [
        ("flop", Board::from_str("9♣ 6♦ 5♥")?),
        ("turn", Board::from_str("9♣ 6♦ 5♥ 5♠")?),
        ("river", *board),
    ];
    for (name, partial) in streets {
        let request = EquityRequest {
            players: vec![PlayerSpec::Exact(GUS), PlayerSpec::Exact(DANIEL)],
            board: partial,
            opts: EquityOptions::default(),
        };
        let report = request.compute()?;
        println!(
            "  {:<7}  {:<16}  {:>8}  {:>10}  {:>7.1}%  {:>7.1}%",
            name,
            partial.cards().to_string(),
            report.samples,
            format!("{:?}", report.method),
            report.players[0].equity_pct(),
            report.players[1].equity_pct(),
        );
    }
    Ok(())
}

/// Pot odds and EV only mean something over many repetitions of the same spot.
fn river_pot_odds(board: &Board) -> Result<(), PKError> {
    let pot_after_bet = POT_AT_RIVER + DANIEL_RIVER_BET;
    let pot_odds = PotOdds::new(pot_after_bet, DANIEL_RIVER_BET);
    let versus = Versus::new_with_board(GUS, Combos::from_str("66")?, *board);
    let odds = versus.combined_odds_at_river()?;
    let ev = Ev::new(odds, pot_odds);

    println!("\nGus faces Daniel's {DANIEL_RIVER_BET} bet into {POT_AT_RIVER}:");
    println!("  {pot_odds}");
    println!(
        "  Against the {} live combos of 66 the river has one runout each: wins={}, losses={}, draws={}.",
        odds.total(),
        odds.wins,
        odds.losses,
        odds.draws
    );
    println!(
        "  EV of calling: {:+.0} chips. A single hand has an outcome; the EV is",
        ev.as_chips()
    );
    println!("  the average over every time this spot repeats.");
    Ok(())
}

// ── 2. The Bayesian table ─────────────────────────────────────────────────────

/// Daniel's belief about Gus's cards, street by street. The cards are fixed
/// and unknown; probability here is degree of belief, not frequency.
fn bayesian_table(board: &Board) -> Result<(WeightedCombos, WeightedCombos), PKError> {
    section("2. The Bayesian table: what does Gus hold?");

    let prior = priors();
    let dead = Cards::from(vec![DANIEL.first(), DANIEL.second()]);
    println!("\nBlockers are conditioning. Daniel's own {DANIEL} removes hands from the prior:");
    println!(
        "  {} hands in the chart, {} once Daniel's cards are dead.",
        Twos::from(&prior.to_combos()).len(),
        prior
            .to_combos()
            .iter()
            .map(|combo| live_hands(*combo, &dead).len())
            .sum::<usize>()
    );
    report_belief("Prior (Gus opens from early position)", &prior, &dead);

    let flop = Cards::from_str("9♣ 6♦ 5♥")?;
    let turn = Cards::from_str("9♣ 6♦ 5♥ 5♠")?;
    let river = board.cards();

    // Each step is Bayes' rule: new weight = old weight × P(action | hand).
    // The likelihoods are a hand-written model of how an aggressive player
    // acts; the quality of the posterior is capped by the quality of this model.
    let posterior = update(&prior, &dead, |two| {
        if two.is_pair() {
            0.9
        } else if two.is_suited() {
            0.6
        } else {
            0.4
        }
    });
    report_belief(
        "After Gus calls a 3-bet: P(call | pair)=0.9, suited=0.6, offsuit=0.4",
        &posterior,
        &dead,
    );

    let dead = with_board(&dead, &flop);
    let posterior = update(&posterior, &dead, |two| likelihood(bucket(*two, &flop), 0.8, 0.3, 0.15));
    report_belief(
        "After Gus check-raises 9♣ 6♦ 5♥: P(x/r | monster)=0.8, pair=0.3, air=0.15",
        &posterior,
        &dead,
    );

    let dead = with_board(&dead, &turn);
    let posterior = update(&posterior, &dead, |two| {
        likelihood(bucket(*two, &turn), 0.9, 0.35, 0.15)
    });
    report_belief(
        "After Gus bets the 5♠ turn: P(bet | monster)=0.9, pair=0.35, air=0.15",
        &posterior,
        &dead,
    );
    let after_turn = posterior.clone();

    let dead = with_board(&dead, &river);
    let posterior = update(&posterior, &dead, |two| likelihood(bucket(*two, &river), 0.5, 0.6, 0.6));
    report_belief(
        "After Gus checks the 8♠ river: P(check | monster)=0.5, pair=0.6, air=0.6",
        &posterior,
        &dead,
    );

    solver_update(board, &after_turn, &posterior, &dead);
    let before_shove = posterior.clone();

    let posterior = update(&posterior, &dead, |two| {
        likelihood(bucket(*two, &river), 0.95, 0.05, 0.1)
    });
    report_belief(
        "After Gus check-raises all in: P(shove | monster)=0.95, pair=0.05, air=0.1",
        &posterior,
        &dead,
    );

    Ok((before_shove, posterior))
}

/// Two charts, two priors. The library's GTO chart for an early open does not
/// contain 55 at all, and a zero prior stays zero under any evidence
/// (Cromwell's rule). Gus's reputation says loose-aggressive, so that chart is
/// the prior this example uses.
fn priors() -> WeightedCombos {
    let gto = PositionRanges::gto_nine_max();
    let gto_open = gto.for_position(Position::UTG).for_action("open_raise");
    let gto_open = gto_open.map(weighted_combos_from_range).unwrap_or_default();
    println!("\nA position chart is a prior.");
    println!("  GTO 9-max UTG open:        {}", gto_open.to_range_str());
    println!(
        "  P(55) under that prior:    {:.0}%. Zero stays zero no matter what Gus does.",
        gto_open.frequency(&Combo::COMBO_55).unwrap_or_default() * 100.0
    );

    let lag = PositionRanges::loose_aggressive_six_max();
    let lag_open = lag.for_position(Position::LJ).for_action("open_raise");
    let lag_open = lag_open.map(weighted_combos_from_range).unwrap_or_default();
    println!("  Loose-aggressive open:     {}", lag_open.to_range_str());
    println!("  Gus's reputation picks the second chart. That choice is itself a prior.");
    lag_open
}

/// The hand-made update was one likelihood table. The solver's equilibrium
/// is another: `P(action | hand)` for every hand at every node. Feeding Gus's
/// range through [`WeightedCombos::after_action`] at the river root is the
/// same Bayes numerator, with the strategy profile as the likelihood.
fn solver_update(board: &Board, after_turn: &WeightedCombos, hand_made: &WeightedCombos, dead: &Cards) {
    let gus_range = after_turn.to_combos();
    let daniel_range = PositionRanges::gto_nine_max()
        .for_position(Position::UTGP1)
        .for_action("three_bet")
        .and_then(|range| Combos::from_str(&weighted_combos_from_range(range).to_range_str()).ok())
        .unwrap_or_default();

    println!("\nThe library's own update. A river CFR solve gives P(check | hand) for Gus:");
    println!("  Gus (out of position): {gus_range}");
    println!("  Daniel's 3-bet chart:  {daniel_range}");

    let config =
        SolverConfig::new(gus_range, daniel_range, *board, STACK_AT_RIVER, POT_AT_RIVER).with_max_iterations(300);
    let clock = Instant::now();
    let mut solver = Solver::new(config);
    let result = solver.solve();
    println!(
        "  {} iterations, exploitability {:.0} chips ({:.1}% of the pot), {:.1?}",
        result.iterations,
        result.exploitability,
        result.exploitability / POT_AT_RIVER as f64 * 100.0,
        clock.elapsed()
    );
    println!("  Not converged, and the solver iterates hash maps, so this profile and the");
    println!("  column below move from run to run. A seeded sampler replays; this does not.");

    // The solver keeps its tree private; rebuilding from the same config is deterministic.
    let tree = GameTree::build_river(&solver.config);
    let root = solver.root_id();
    let Some(Node::Action(node)) = tree.get(root) else {
        println!("  (river tree has no action root; skipping)");
        return;
    };
    let Some(check) = node.actions.iter().position(|action| matches!(action, Action::Check)) else {
        println!("  (no check action at the root; skipping)");
        return;
    };

    if let Some(freqs) = result.equilibrium.get(root, &GUS) {
        println!(
            "  Equilibrium for Gus's actual {GUS} at the root: {}",
            node.actions
                .iter()
                .zip(freqs.as_slice())
                .map(|(action, freq)| format!("{action} {:.0}%", freq * 100.0))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    let from_solver = after_turn.after_action(&result.equilibrium, root, check);
    println!("\n  Posterior after the river check, two likelihood models side by side:");
    println!("  {:<6}  {:>10}  {:>10}", "combo", "hand-made", "solver");
    let hand_rows = mass_share(hand_made, dead);
    for (combo, solver_share) in mass_share(&from_solver, dead) {
        let hand_share = hand_rows.iter().find(|(c, _)| *c == combo).map_or(0.0, |(_, p)| *p);
        println!(
            "  {:<6}  {:>9.1}%  {:>9.1}%",
            combo.to_string(),
            hand_share * 100.0,
            solver_share * 100.0
        );
    }
    println!("  Same prior, same evidence, different likelihood: different belief.");
}

// ── 3. Where they meet ────────────────────────────────────────────────────────

/// Daniel's decision needs both tables: a frequency (how often 66 beats each
/// hand, counted) and a belief (how much weight each hand carries).
fn where_they_meet(board: &Board, before_shove: &WeightedCombos, posterior: &WeightedCombos) -> Result<(), PKError> {
    section("3. Where they meet: Daniel's call");
    let dead = with_board(&Cards::from(vec![DANIEL.first(), DANIEL.second()]), &board.cards());

    // Frequentist half: each live hand in Gus's range is counted against 66.
    let odds = odds_versus_daniel(posterior, *board, &dead);
    // Bayesian half: the counts are weighted by the posterior.
    let weighted = posterior.weighted_win_probability(&odds);

    // The equity engine samples a range uniformly: the weights are dropped
    // (gap 3 in the essay), so it answers a different question.
    let request = EquityRequest {
        players: vec![PlayerSpec::Exact(DANIEL), PlayerSpec::Range(posterior.to_combos())],
        board: *board,
        opts: EquityOptions {
            seed: Some(7),
            ..Default::default()
        },
    };
    let uniform = request.compute()?;

    println!("\nDaniel's {DANIEL} is sixes full. Against Gus's posterior range:");
    println!("  P(Daniel wins), weights honoured:  {:>5.1}%", weighted * 100.0);
    println!(
        "  P(Daniel wins), weights dropped:   {:>5.1}%  ({:?}, every combo at full weight)",
        uniform.players[0].equity_pct(),
        uniform.method
    );

    let pot_before_call = POT_AT_RIVER + DANIEL_RIVER_BET + STACK_AT_RIVER;
    let to_call = STACK_AT_RIVER - DANIEL_RIVER_BET;
    let pot_odds = PotOdds::new(pot_before_call, to_call);
    println!("\nGus is all in. {pot_odds}");
    println!(
        "  EV of calling under the belief above: {:+.0} chips",
        belief_ev(weighted, pot_odds).as_chips()
    );

    // A posterior is only as good as its likelihood. Read Gus as a player who
    // shoves air half the time, apply that to the same pre-shove belief, and
    // the same evidence points the other way.
    let river = board.cards();
    let bluffy = update(before_shove, &dead, |two| {
        likelihood(bucket(*two, &river), 0.95, 0.3, 0.5)
    });
    let bluffy_win = bluffy.weighted_win_probability(&odds_versus_daniel(&bluffy, *board, &dead));
    println!("\nThe same evidence under Daniel's read, 'Gus shoves air half the time':");
    println!(
        "  P(Daniel wins): {:>5.1}%   EV of calling: {:+.0} chips",
        bluffy_win * 100.0,
        belief_ev(bluffy_win, pot_odds).as_chips()
    );
    println!("  Daniel called. The frequencies were never in doubt; the likelihood was.");
    Ok(())
}

/// One river runout per hand, so each [`WinLoseDraw`] is a count of 1.
fn odds_versus_daniel(range: &WeightedCombos, board: Board, dead: &Cards) -> HashMap<Two, WinLoseDraw> {
    let daniel = Seven::from_case_and_board(&DANIEL, &board).hand_rank();
    let mut odds = HashMap::new();
    for combo in range.to_combos().iter() {
        for two in live_hands(*combo, dead) {
            let gus = Seven::from_case_and_board(&two, &board).hand_rank();
            let result = match daniel.cmp(&gus) {
                Ordering::Greater => WinLoseDraw {
                    wins: 1,
                    losses: 0,
                    draws: 0,
                },
                Ordering::Less => WinLoseDraw {
                    wins: 0,
                    losses: 1,
                    draws: 0,
                },
                Ordering::Equal => WinLoseDraw {
                    wins: 0,
                    losses: 0,
                    draws: 1,
                },
            };
            odds.insert(two, result);
        }
    }
    odds
}

/// [`Ev`] wants counts, so a belief is scaled to pseudo-counts.
fn belief_ev(win_probability: f64, pot_odds: PotOdds) -> Ev {
    let wins = (win_probability * BELIEF_SCALE).round() as u64;
    let losses = BELIEF_SCALE as u64 - wins;
    Ev::new(WinLoseDraw { wins, losses, draws: 0 }, pot_odds)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Standard error of a sampled proportion: sqrt(p(1-p)/n). Zero when nothing
/// was sampled, because an exact count has no sampling error.
fn standard_error(equity: f64, samples: u64) -> f64 {
    if samples == 0 {
        return 0.0;
    }
    (equity * (1.0 - equity) / samples as f64).sqrt()
}

/// The specific hands of `combo` that do not collide with `dead`.
fn live_hands(combo: Combo, dead: &Cards) -> Vec<Two> {
    Twos::from(combo)
        .to_vec()
        .into_iter()
        .filter(|two| !dead.contains(&two.first()) && !dead.contains(&two.second()))
        .collect()
}

/// Each combo's share of the belief: weight × live hands, normalised to 1.
fn mass_share(range: &WeightedCombos, dead: &Cards) -> Vec<(Combo, f64)> {
    let mut rows: Vec<(Combo, f64)> = range
        .to_combos()
        .iter()
        .map(|combo| {
            let weight = range.frequency(combo).unwrap_or_default();
            (*combo, weight * live_hands(*combo, dead).len() as f64)
        })
        .collect();
    let total: f64 = rows.iter().map(|(_, mass)| mass).sum();
    if total > 0.0 {
        for row in &mut rows {
            row.1 /= total;
        }
    }
    rows.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(Ordering::Equal)
            .then_with(|| right.0.cmp(&left.0))
    });
    rows
}

/// Bayes' rule, unnormalised: `new weight = old weight × mean P(action | hand)`
/// over the combo's live hands, which is what [`WeightedCombos::after_action`]
/// does with a strategy profile. Weights are then rescaled so the heaviest
/// combo sits at 1.0; only the ratios matter, and [`WeightedCombos`] stores
/// whole percents, so rescaling keeps small weights from rounding to zero.
fn update(prior: &WeightedCombos, dead: &Cards, likelihood: impl Fn(&Two) -> f64) -> WeightedCombos {
    let mut scaled: Vec<(Combo, f64)> = Vec::new();
    for combo in prior.to_combos().iter() {
        let hands = live_hands(*combo, dead);
        if hands.is_empty() {
            continue;
        }
        let mean = hands.iter().map(&likelihood).sum::<f64>() / hands.len() as f64;
        let weight = prior.frequency(combo).unwrap_or_default() * mean;
        if weight > 0.0 {
            scaled.push((*combo, weight));
        }
    }
    let heaviest = scaled.iter().map(|(_, w)| *w).fold(0.0_f64, f64::max);
    let mut posterior = WeightedCombos::default();
    for (combo, weight) in scaled {
        posterior.insert(combo, weight / heaviest);
    }
    posterior
}

/// A chart entry such as `"JJ+"` at frequency `f` becomes each of its single
/// combos (`JJ`, `QQ`, `KK`, `AA`) at `f`. The chart's `JJ+` is one [`Combo`]
/// with a plus flag; expanding it to hands and regrouping by [`Combo`] splits
/// it, so 55 can be tracked apart from 66 and 99.
fn weighted_combos_from_range(range: &WeightedRange) -> WeightedCombos {
    let mut weighted = WeightedCombos::default();
    for entry in range.combos() {
        if let Ok(combos) = Combos::from_str(&entry.range) {
            for two in Twos::from(combos).to_vec() {
                weighted.insert(Combo::from(two), entry.frequency);
            }
        }
    }
    weighted
}

/// Coarse hand strength against the board so far. Trips or better is a
/// monster; one or two pair (on a paired board, everyone has two pair) is a
/// pair; nothing is air.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Bucket {
    Monster,
    Pair,
    Air,
}

fn bucket(two: Two, board: &Cards) -> Bucket {
    let mut cards = board.to_vec();
    cards.push(two.first());
    cards.push(two.second());
    let cards = Cards::from(cards);
    let name = match cards.len() {
        5 => Five::try_from(cards).map(|hand| hand.hand_rank().name),
        6 => Six::try_from(cards).map(|hand| hand.hand_rank().name),
        _ => Seven::try_from(cards).map(|hand| hand.hand_rank().name),
    };
    match name.unwrap_or(HandRankName::HighCard) {
        HandRankName::Pair | HandRankName::TwoPair => Bucket::Pair,
        HandRankName::HighCard | HandRankName::RazzLow => Bucket::Air,
        _ => Bucket::Monster,
    }
}

fn likelihood(bucket: Bucket, monster: f64, pair: f64, air: f64) -> f64 {
    match bucket {
        Bucket::Monster => monster,
        Bucket::Pair => pair,
        Bucket::Air => air,
    }
}

fn with_board(dead: &Cards, board: &Cards) -> Cards {
    let mut cards = dead.to_vec();
    for card in board.to_vec() {
        if !cards.contains(&card) {
            cards.push(card);
        }
    }
    Cards::from(cards)
}

fn report_belief(title: &str, range: &WeightedCombos, dead: &Cards) {
    println!("\n{title}");
    let rows = mass_share(range, dead);
    let line = rows
        .iter()
        .map(|(combo, share)| format!("{combo} {:.0}%", share * 100.0))
        .collect::<Vec<_>>()
        .join("  ");
    println!("  {line}");
    let gus = rows
        .iter()
        .find(|(combo, _)| *combo == Combo::COMBO_55)
        .map_or(0.0, |(_, p)| *p);
    println!("  P(Gus holds 55) = {:.1}%", gus * 100.0);
}

#[cfg(test)]
#[allow(non_snake_case)]
mod examples__two_tables_tests {
    use super::*;
    use pkcore::analysis::gto::combo::Combo;
    use pkcore::analysis::gto::weighted_combos::WeightedCombos;
    use pkcore::arrays::two::Two;
    use pkcore::bot::weighted_range::WeightedRange;
    use pkcore::cards::Cards;
    use std::str::FromStr;

    fn close(left: f64, right: f64) -> bool {
        (left - right).abs() < 1e-3
    }

    #[test]
    fn standard_error_matches_the_essay_at_default_samples() {
        // The essay: equity near 50% at n = 25,000 has a standard error of ~0.32 points.
        let se = standard_error(0.5, 25_000);
        assert!(close(se * 100.0, 0.316), "got {se}");
    }

    #[test]
    fn standard_error_is_zero_for_an_exact_answer() {
        assert_eq!(0.0, standard_error(0.5, 0));
    }

    #[test]
    fn live_hands_removes_combos_blocked_by_dead_cards() {
        let dead = Cards::from_str("5♥ 5♠").unwrap_or_default();
        assert_eq!(1, live_hands(Combo::COMBO_55, &dead).len());

        let dead = Cards::from_str("6♠ 6♥ 6♦").unwrap_or_default();
        assert!(live_hands(Combo::COMBO_66, &dead).is_empty());

        assert_eq!(6, live_hands(Combo::COMBO_AA, &Cards::default()).len());
    }

    #[test]
    fn mass_share_weighs_a_combo_by_weight_times_live_hands() {
        // The essay's worked case: 6 combos of AA against 16 of AK is 27%.
        let mut prior = WeightedCombos::default();
        prior.insert(Combo::COMBO_AA, 1.0);
        prior.insert(Combo::COMBO_AKs, 1.0);
        prior.insert(Combo::COMBO_AKo, 1.0);

        let share = mass_share(&prior, &Cards::default());
        let aces = share
            .iter()
            .find(|(combo, _)| *combo == Combo::COMBO_AA)
            .map(|(_, p)| *p);
        assert!(close(aces.unwrap_or_default(), 6.0 / 22.0));
    }

    #[test]
    fn update_multiplies_weights_by_the_likelihood_of_the_action() {
        // The essay's worked case: aces 4-bet always, ace-king a quarter of the
        // time, so after the 4-bet aces go from 27% to 60% of the range.
        let mut prior = WeightedCombos::default();
        prior.insert(Combo::COMBO_AA, 1.0);
        prior.insert(Combo::COMBO_AKs, 1.0);
        prior.insert(Combo::COMBO_AKo, 1.0);

        let posterior = update(&prior, &Cards::default(), |two| if two.is_pair() { 1.0 } else { 0.25 });

        assert!(close(posterior.frequency(&Combo::COMBO_AA).unwrap_or_default(), 1.0));
        assert!(close(posterior.frequency(&Combo::COMBO_AKo).unwrap_or_default(), 0.25));

        let share = mass_share(&posterior, &Cards::default());
        let aces = share
            .iter()
            .find(|(combo, _)| *combo == Combo::COMBO_AA)
            .map(|(_, p)| *p);
        assert!(close(aces.unwrap_or_default(), 0.6));
    }

    #[test]
    fn update_drops_a_combo_with_zero_likelihood() {
        let mut prior = WeightedCombos::default();
        prior.insert(Combo::COMBO_AA, 1.0);
        prior.insert(Combo::COMBO_KK, 1.0);

        let posterior = update(&prior, &Cards::default(), |two| {
            if two.is_pair() && two.first().get_rank() == pkcore::rank::Rank::ACE {
                1.0
            } else {
                0.0
            }
        });

        assert_eq!(None, posterior.frequency(&Combo::COMBO_KK));
    }

    #[test]
    fn weighted_combos_from_range_keeps_each_entry_weight() {
        let mut range = WeightedRange::new();
        range.push("JJ+", 1.0);
        range.push("AKs", 0.5);

        let weighted = weighted_combos_from_range(&range);

        assert!(close(weighted.frequency(&Combo::COMBO_AA).unwrap_or_default(), 1.0));
        assert!(close(weighted.frequency(&Combo::COMBO_JJ).unwrap_or_default(), 1.0));
        assert!(close(weighted.frequency(&Combo::COMBO_AKs).unwrap_or_default(), 0.5));
        assert_eq!(None, weighted.frequency(&Combo::COMBO_TT));
    }

    #[test]
    fn bucket_classifies_a_hand_against_the_board_so_far() {
        let flop = Cards::from_str("9♣ 6♦ 5♥").unwrap_or_default();
        assert_eq!(Bucket::Monster, bucket(Two::HAND_5D_5C, &flop));
        assert_eq!(Bucket::Monster, bucket(Two::HAND_6S_6H, &flop), "sixes flop a set");
        assert_eq!(Bucket::Pair, bucket(Two::HAND_TS_TD, &flop), "tens are an overpair");
        assert_eq!(Bucket::Air, bucket(Two::HAND_AS_KS, &flop));

        let river = Cards::from_str("9♣ 6♦ 5♥ 5♠ 8♠").unwrap_or_default();
        assert_eq!(Bucket::Monster, bucket(Two::HAND_6S_6H, &river));
    }
}
