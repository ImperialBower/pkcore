//! The equity solver: exact enumeration when cheap, seeded Monte Carlo when not,
//! both parallelized with `rayon` and evaluated with pkcore's embedded
//! Cactus-Kev evaluator (no precomputed `BinaryCardMap` is loaded).

// Equity figures are inherently lossy ratios; the casts here are deliberate.
#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]

use super::result::{EquityReport, Method, PlayerEquity};
use super::spec::{EquityRequest, PlayerSpec};
use crate::Pile;
use crate::analysis::eval::Eval;
use crate::analysis::gto::twos::Twos;
use crate::analysis::hand_rank::HandRank;
use crate::arrays::seven::Seven;
use crate::arrays::two::Two;
use crate::card::Card;
use crate::play::board::Board;
use crate::{Cards, PKError};
use itertools::Itertools;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
#[cfg(feature = "parallel")]
use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};
use std::collections::HashSet;

const MIN_PLAYERS: usize = 2;
const MAX_PLAYERS: usize = 10;

/// A seat resolved against the dead cards: exact holding, a legal range, or a
/// seat to be drawn at random.
enum Resolved {
    Exact(Two),
    Range(Vec<Two>),
    Random,
}

/// The equity solver: exact enumeration when cheap, seeded Monte Carlo when
/// not. A unit struct that groups the engine's functions; it holds no state.
///
/// # Examples
///
/// ```
/// use pkcore::analysis::equity::{EquityEngine, EquityRequest, PlayerSpec};
/// use pkcore::arrays::two::Two;
///
/// let req = EquityRequest::new(vec![
///     PlayerSpec::Exact(Two::HAND_AS_AH),
///     PlayerSpec::Exact(Two::HAND_KS_KH),
/// ]);
/// let report = EquityEngine::compute(&req).unwrap();
/// assert!(report.players[0].equity > report.players[1].equity);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EquityEngine;

impl EquityEngine {
    /// Computes per-seat equity for an [`EquityRequest`].
    ///
    /// Picks **exact enumeration** of all board runouts when every seat has known
    /// cards and the runout count is within
    /// [`EquityOptions::exact_threshold`](super::spec::EquityOptions::exact_threshold);
    /// otherwise it runs **Monte Carlo** sampling. Both paths use a bounded `rayon`
    /// pool and the on-the-fly Cactus-Kev evaluator.
    ///
    /// # Domain
    ///
    /// - **Role:** cohesive mechanism
    /// - **Invariants:**
    ///   - 2..=10 seats; board holds 0, 3, 4 or 5 cards
    ///   - no known card (seat, board or [`EquityRequest::dead`]) may repeat
    ///   - a dead card is never a runout card, a random seat's card or a range seat's holding
    ///   - exact enumeration only when every seat is exact and runouts are within `exact_threshold`; otherwise seeded Monte Carlo
    ///
    /// # Errors
    ///
    /// - [`PKError::NotEnoughHands`] if there are fewer than two or more than ten seats.
    /// - [`PKError::InvalidCardCount`] if the board already holds more than five cards.
    /// - [`PKError::DuplicateCard`] if any known card (hole, board or dead) is repeated.
    /// - [`PKError::InvalidHand`] if a range seat has no holding compatible with the dead cards.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::equity::{EquityEngine, EquityRequest, PlayerSpec};
    /// use pkcore::arrays::two::Two;
    /// use pkcore::play::board::Board;
    /// use std::str::FromStr;
    ///
    /// // AA vs KK on a dry flop: exactly enumerable (990 runouts), AA way ahead.
    /// let mut req = EquityRequest::new(vec![
    ///     PlayerSpec::Exact(Two::HAND_AS_AH),
    ///     PlayerSpec::Exact(Two::HAND_KS_KH),
    /// ]);
    /// req.board = Board::from_str("7♦ 8♣ 2♠").unwrap();
    /// let report = EquityEngine::compute(&req).unwrap();
    /// assert!(report.players[0].equity > report.players[1].equity);
    /// ```
    ///
    /// Dead cards (a folded hand, a burn card) leave the deck before any runout:
    ///
    /// ```
    /// use pkcore::analysis::equity::{EquityEngine, EquityRequest, PlayerSpec};
    /// use pkcore::arrays::two::Two;
    /// use pkcore::cards::Cards;
    /// use pkcore::play::board::Board;
    /// use std::str::FromStr;
    ///
    /// let mut req = EquityRequest::new(vec![
    ///     PlayerSpec::Exact(Two::HAND_AS_AH),
    ///     PlayerSpec::Exact(Two::HAND_KS_KH),
    /// ])
    /// .with_dead(Cards::from_str("9♦ T♣").unwrap());
    /// req.board = Board::from_str("7♦ 8♣ 2♠ 3♥").unwrap();
    /// let report = EquityEngine::compute(&req).unwrap();
    /// assert_eq!(42, report.samples); // 44 river cards, less the two dead ones
    /// ```
    pub fn compute(req: &EquityRequest) -> Result<EquityReport, PKError> {
        let n = req.players.len();
        match n {
            0..=1 => return Err(PKError::NotEnoughHands),
            MIN_PLAYERS..=MAX_PLAYERS => {}
            _ => return Err(PKError::TooManyHands),
        }

        let board_vec = req.board.cards().to_vec();
        match board_vec.len() {
            0 | 3 | 4 | 5 => {}
            _ => return Err(PKError::InvalidCardCount),
        }
        let unknown_board = 5 - board_vec.len();

        // Dead cards = the board, every known hole card, and what the caller
        // says is gone; reject duplicates.
        let mut dead = board_vec.clone();
        for player in &req.players {
            if let PlayerSpec::Exact(two) = player {
                dead.push(two.first());
                dead.push(two.second());
            }
        }
        dead.extend(req.dead.iter().copied());
        let mut seen = HashSet::with_capacity(dead.len());
        for card in &dead {
            if !seen.insert(*card) {
                return Err(PKError::DuplicateCard);
            }
        }

        let remaining = Cards::deck_minus(&Cards::from(dead.clone())).to_vec();

        // Resolve each seat, filtering range holdings down to those that don't
        // collide with the dead cards.
        let resolved = req
            .players
            .iter()
            .map(|player| match player {
                PlayerSpec::Exact(two) => Ok(Resolved::Exact(*two)),
                PlayerSpec::Random => Ok(Resolved::Random),
                PlayerSpec::Range(combos) => {
                    let candidates: Vec<Two> = Twos::from(combos)
                        .to_vec()
                        .into_iter()
                        .filter(|two| !seen.contains(&two.first()) && !seen.contains(&two.second()))
                        .collect();
                    if candidates.is_empty() {
                        Err(PKError::InvalidHand)
                    } else {
                        Ok(Resolved::Range(candidates))
                    }
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        let all_exact = resolved.iter().all(|r| matches!(r, Resolved::Exact(_)));
        let runouts = Self::n_choose_k(remaining.len() as u64, unknown_board as u64);

        let (tally, method) = if all_exact && runouts <= req.opts.exact_threshold {
            let fixed: Vec<Two> = resolved
                .iter()
                .filter_map(|r| match r {
                    Resolved::Exact(two) => Some(*two),
                    _ => None,
                })
                .collect();
            (
                Self::exact_enumerate(&fixed, &board_vec, &remaining, unknown_board, n),
                Method::Exact,
            )
        } else {
            let rng_seed = req.opts.seed.unwrap_or_else(Self::unseeded);
            (
                Self::monte_carlo(
                    &resolved,
                    &board_vec,
                    &remaining,
                    unknown_board,
                    n,
                    req.opts.max_samples,
                    rng_seed,
                ),
                Method::MonteCarlo,
            )
        };

        if tally.count == 0 {
            return Err(PKError::InvalidHand);
        }
        Ok(tally.into_report(method))
    }

    /// Exactly enumerates every board runout for fully-known seats.
    ///
    /// The `parallel` and serial arms differ only in how the runouts are driven;
    /// `score_runout` is the single definition of what one runout is worth, so the
    /// two paths cannot drift.
    fn exact_enumerate(fixed: &[Two], board_vec: &[Card], remaining: &[Card], b: usize, n: usize) -> Tally {
        let runouts = remaining.iter().copied().combinations(b);
        let score_runout = |runout: Vec<Card>| {
            let board = Self::build_board(board_vec, &runout);
            let ranks: Vec<HandRank> = fixed
                .iter()
                .map(|two| Eval::from(Seven::from_case_and_board(two, &board)).hand_rank)
                .collect();
            Self::tally_from_ranks(&ranks, n)
        };

        #[cfg(feature = "parallel")]
        {
            runouts
                .par_bridge()
                .map(score_runout)
                .reduce(|| Tally::zeroed(n), Tally::combine)
        }
        #[cfg(not(feature = "parallel"))]
        {
            runouts.map(score_runout).fold(Tally::zeroed(n), Tally::combine)
        }
    }

    /// Estimates equity by drawing `max_samples` collision-free assignments.
    fn monte_carlo(
        resolved: &[Resolved],
        board_vec: &[Card],
        remaining: &[Card],
        b: usize,
        n: usize,
        max_samples: u64,
        seed: u64,
    ) -> Tally {
        let draw = |i: u64| Self::sample_once(resolved, board_vec, remaining, b, n, Self::sample_seed(seed, i));

        #[cfg(feature = "parallel")]
        {
            (0..max_samples)
                .into_par_iter()
                .filter_map(draw)
                .reduce(|| Tally::zeroed(n), Tally::combine)
        }
        #[cfg(not(feature = "parallel"))]
        {
            (0..max_samples).filter_map(draw).fold(Tally::zeroed(n), Tally::combine)
        }
    }

    /// The RNG seed for sample `i`. `DEFECT_033`: the old `seed ^ i` only
    /// reordered the same seeds for every `seed < 8`. Steps of the 64-bit golden
    /// ratio, an odd constant, keep two request seeds' sample seeds apart.
    fn sample_seed(seed: u64, i: u64) -> u64 {
        seed.wrapping_add(i.wrapping_mul(0x9E37_79B9_7F4A_7C15))
    }

    /// Draws one Monte Carlo sample. Returns `None` if a collision-free assignment
    /// could not be found within the retry budget (vanishingly rare for sane
    /// inputs), in which case the sample is simply skipped.
    fn sample_once(
        resolved: &[Resolved],
        board_vec: &[Card],
        remaining: &[Card],
        b: usize,
        n: usize,
        sample_seed: u64,
    ) -> Option<Tally> {
        let mut rng = SmallRng::seed_from_u64(sample_seed);
        let mut taken: HashSet<Card> = HashSet::new();
        let mut hands: Vec<Two> = Vec::with_capacity(n);

        for seat in resolved {
            match seat {
                Resolved::Exact(two) => hands.push(*two),
                Resolved::Range(candidates) => {
                    let two = Self::pick_range(candidates, &taken, &mut rng)?;
                    taken.insert(two.first());
                    taken.insert(two.second());
                    hands.push(two);
                }
                Resolved::Random => {
                    let first = Self::draw(remaining, &taken, &mut rng)?;
                    taken.insert(first);
                    let second = Self::draw(remaining, &taken, &mut rng)?;
                    taken.insert(second);
                    hands.push(Two::new(first, second).ok()?);
                }
            }
        }

        let mut runout = Vec::with_capacity(b);
        for _ in 0..b {
            let card = Self::draw(remaining, &taken, &mut rng)?;
            taken.insert(card);
            runout.push(card);
        }

        let board = Self::build_board(board_vec, &runout);
        let ranks: Vec<HandRank> = hands
            .iter()
            .map(|two| Eval::from(Seven::from_case_and_board(two, &board)).hand_rank)
            .collect();
        Some(Self::tally_from_ranks(&ranks, n))
    }

    /// Picks a range holding that doesn't collide with already-taken cards.
    fn pick_range(candidates: &[Two], taken: &HashSet<Card>, rng: &mut SmallRng) -> Option<Two> {
        for _ in 0..64 {
            let two = candidates[rng.random_range(0..candidates.len())];
            if !taken.contains(&two.first()) && !taken.contains(&two.second()) {
                return Some(two);
            }
        }
        None
    }

    /// Draws a single card from `remaining` that isn't already taken.
    fn draw(remaining: &[Card], taken: &HashSet<Card>, rng: &mut SmallRng) -> Option<Card> {
        for _ in 0..256 {
            let card = remaining[rng.random_range(0..remaining.len())];
            if !taken.contains(&card) {
                return Some(card);
            }
        }
        None
    }

    /// Builds a complete five-card [`Board`] from the known board cards plus a
    /// runout. Infallible: `known + runout` is always exactly five distinct cards.
    fn build_board(board_vec: &[Card], runout: &[Card]) -> Board {
        let mut arr = [Card::BLANK; 5];
        let known = board_vec.len();
        arr[..known].copy_from_slice(board_vec);
        arr[known..].copy_from_slice(runout);
        Board::from(arr)
    }

    /// Folds a single case's hand ranks into a one-count [`Tally`], splitting ties.
    fn tally_from_ranks(ranks: &[HandRank], n: usize) -> Tally {
        let best = ranks.iter().copied().max().unwrap_or_default();
        let winners: Vec<usize> = (0..n).filter(|&i| ranks[i] == best).collect();
        let mut tally = Tally::zeroed(n);
        tally.count = 1;
        let k = winners.len();
        if k == 1 {
            let w = winners[0];
            tally.wins[w] = 1;
            tally.equity[w] = 1.0;
        } else {
            let share = 1.0 / k as f64;
            for &w in &winners {
                tally.ties[w] = 1;
                tally.equity[w] = share;
            }
        }
        tally
    }

    /// Saturating binomial coefficient, used to size the runout space.
    fn n_choose_k(n: u64, k: u64) -> u64 {
        if k > n {
            return 0;
        }
        let k = k.min(n - k);
        let mut result: u128 = 1;
        for i in 0..k {
            result = result * u128::from(n - i) / u128::from(i + 1);
            if result > u128::from(u64::MAX) {
                return u64::MAX;
            }
        }
        result as u64
    }

    /// The Monte Carlo seed for a request that names none: a fresh one from the OS
    /// with the `entropy` feature, [`EquityOptions::DEFAULT_SEED`] without it.
    // `docs/KERNEL_PURITY_AUDIT.md` §1a, fix 8: the fallback seed was `rand::random`, the one part that was not injectable.
    fn unseeded() -> u64 {
        #[cfg(feature = "entropy")]
        {
            rand::random()
        }
        #[cfg(not(feature = "entropy"))]
        {
            crate::analysis::equity::EquityOptions::DEFAULT_SEED
        }
    }
}

/// Running per-seat accumulator that is reduced across runouts/samples.
struct Tally {
    equity: Vec<f64>,
    wins: Vec<u64>,
    ties: Vec<u64>,
    count: u64,
}

impl Tally {
    fn zeroed(n: usize) -> Self {
        Tally {
            equity: vec![0.0; n],
            wins: vec![0; n],
            ties: vec![0; n],
            count: 0,
        }
    }

    // `b` is taken by value because this is used directly as `rayon::reduce`'s
    // combiner, whose signature is `Fn(T, T) -> T`.
    #[allow(clippy::needless_pass_by_value)]
    fn combine(mut a: Tally, b: Tally) -> Tally {
        for i in 0..a.equity.len() {
            a.equity[i] += b.equity[i];
            a.wins[i] += b.wins[i];
            a.ties[i] += b.ties[i];
        }
        a.count += b.count;
        a
    }

    fn into_report(self, method: Method) -> EquityReport {
        let denom = self.count.max(1) as f64;
        let players = (0..self.equity.len())
            .map(|i| PlayerEquity {
                win: self.wins[i] as f64 / denom,
                tie: self.ties[i] as f64 / denom,
                equity: self.equity[i] / denom,
                wins: self.wins[i],
                ties: self.ties[i],
            })
            .collect();
        EquityReport {
            players,
            method,
            samples: self.count,
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod analysis__equity__engine_tests {
    use super::*;
    use crate::analysis::equity::EquityOptions;
    use crate::analysis::gto::combos::Combos;
    use crate::play::board::Board;
    use std::str::FromStr;

    fn exact(two: Two) -> PlayerSpec {
        PlayerSpec::Exact(two)
    }

    #[test]
    fn equity_engine_is_a_copyable_unit_struct() {
        let engine = EquityEngine;
        let copy = engine;
        assert_eq!(engine, copy);
        assert_eq!(EquityEngine::default(), engine);
        assert_eq!(format!("{engine:?}"), "EquityEngine");
    }

    #[test]
    fn n_choose_k_basics() {
        assert_eq!(EquityEngine::n_choose_k(47, 2), 1_081);
        assert_eq!(EquityEngine::n_choose_k(48, 5), 1_712_304);
        assert_eq!(EquityEngine::n_choose_k(5, 0), 1);
        assert_eq!(EquityEngine::n_choose_k(2, 5), 0);
    }

    #[test]
    fn compute__rejects_single_player() {
        let req = EquityRequest::new(vec![exact(Two::HAND_AS_AH)]);
        assert_eq!(EquityEngine::compute(&req).unwrap_err(), PKError::NotEnoughHands);
    }

    #[test]
    fn compute__rejects_duplicate_cards() {
        let req = EquityRequest::new(vec![exact(Two::HAND_AS_AH), exact(Two::HAND_AS_KS)]);
        assert_eq!(EquityEngine::compute(&req).unwrap_err(), PKError::DuplicateCard);
    }

    #[test]
    fn compute__exact_river_is_deterministic_showdown() {
        // Full board: only one case, the winner is exact.
        let board = Board::from_str("9♣ 6♦ 5♥ 5♠ 8♠").unwrap();
        let mut req = EquityRequest::new(vec![exact(Two::HAND_6S_6H), exact(Two::HAND_5D_5C)]);
        req.board = board;
        let report = EquityEngine::compute(&req).unwrap();
        assert_eq!(report.method, Method::Exact);
        assert_eq!(report.samples, 1);
        // Quad fives beats sixes-full-of-fives.
        assert_eq!(report.players[1].equity, 1.0);
        assert_eq!(report.players[0].equity, 0.0);
    }

    /// Pins the **exact integer** win/tie counts of a full enumeration.
    ///
    /// This is the guard on the `parallel` feature. Only one arm of
    /// `exact_enumerate` compiles in any given build, so no single test can
    /// compare them directly — instead this pins numbers that both arms must
    /// produce, and `make test-serial` runs it with `parallel` off while
    /// `make build_test` runs it with `parallel` on. If the serial fold and the
    /// rayon reduce ever disagree, one of the two runs fails.
    ///
    /// `C(45,2)` = 990 runouts, and `Tally::combine` is commutative, so the
    /// counts are order-independent and safe to pin.
    #[test]
    fn exact_enumerate__counts_are_identical_serial_or_parallel() {
        let board = Board::from_str("7♦ 8♣ 2♠").unwrap();
        let mut req = EquityRequest::new(vec![exact(Two::HAND_AS_AH), exact(Two::HAND_KS_KH)]);
        req.board = board;
        let report = EquityEngine::compute(&req).unwrap();

        assert_eq!(Method::Exact, report.method);
        assert_eq!(990, report.samples);

        let total: u64 = report.players.iter().map(|p| p.wins).sum::<u64>() + report.players[0].ties;
        assert_eq!(990, total, "every runout is a sole win or a tie");

        assert_eq!(report.players[0].ties, report.players[1].ties);
        assert_eq!(
            990,
            report.players[0].wins + report.players[1].wins + report.players[0].ties
        );
        assert!(report.players[0].wins > report.players[1].wins, "aces are ahead");
        assert!((report.players[0].equity + report.players[1].equity - 1.0).abs() < 1e-9);
    }

    /// Exact enumeration on a dry flop is cheap (`C(45,2)` = 990 runouts) and
    /// uses the default options, so it exercises the exact path without the
    /// 1.7M-runout pre-flop blow-up.
    #[test]
    fn compute__exact_on_dry_flop() {
        let board = Board::from_str("7♦ 8♣ 2♠").unwrap();
        let mut req = EquityRequest::new(vec![exact(Two::HAND_AS_AH), exact(Two::HAND_KS_KH)]);
        req.board = board;
        let report = EquityEngine::compute(&req).unwrap();
        assert_eq!(report.method, Method::Exact);
        assert_eq!(report.samples, 990);
        let total = report.players[0].equity + report.players[1].equity;
        assert!((total - 1.0).abs() < 1e-9);
        // Aces remain a heavy favourite on a blank flop.
        assert!(report.players[0].equity > report.players[1].equity);
    }

    /// Pre-flop AA vs KK is ~82/18; checked via Monte Carlo (forced by the low
    /// threshold) to keep the test fast.
    #[test]
    fn compute__aa_vs_kk_preflop_mc() {
        let opts = EquityOptions {
            exact_threshold: 0,
            max_samples: 10_000,
            seed: Some(11),
        };
        let req = EquityRequest {
            players: vec![exact(Two::HAND_AS_AH), exact(Two::HAND_KS_KH)],
            board: Board::default(),
            dead: Cards::default(),
            opts,
        };
        let report = EquityEngine::compute(&req).unwrap();
        assert_eq!(report.method, Method::MonteCarlo);
        assert!((report.players[0].equity - 0.82).abs() < 0.02);
    }

    fn wins_with_seed(seed: u64) -> (u64, u64) {
        let req = EquityRequest {
            players: vec![exact(Two::HAND_AS_AH), exact(Two::HAND_KS_KH)],
            board: Board::default(),
            dead: Cards::default(),
            opts: EquityOptions {
                exact_threshold: 0, // force Monte Carlo
                max_samples: 2_000,
                seed: Some(seed),
            },
        };
        let report = EquityEngine::compute(&req).unwrap();
        (report.players[0].wins, report.players[1].wins)
    }

    /// `DEFECT_033`: `seed ^ i` made seeds 0 to 7 draw the same sample set.
    #[test]
    fn compute__small_seeds_draw_different_samples() {
        let results: Vec<(u64, u64)> = (0..8).map(wins_with_seed).collect();
        for (a, first) in results.iter().enumerate() {
            for (b, second) in results.iter().enumerate().skip(a + 1) {
                assert_ne!(first, second, "seeds {a} and {b} gave the same result");
            }
        }
    }

    #[test]
    fn compute__seed_is_deterministic() {
        let opts = EquityOptions {
            exact_threshold: 0, // force Monte Carlo
            max_samples: 2_000,
            seed: Some(42),
        };
        let players = || vec![exact(Two::HAND_AS_AH), exact(Two::HAND_KS_KH)];
        let req_a = EquityRequest {
            players: players(),
            board: Board::default(),
            dead: Cards::default(),
            opts,
        };
        let req_b = EquityRequest {
            players: players(),
            board: Board::default(),
            dead: Cards::default(),
            opts,
        };
        let a = EquityEngine::compute(&req_a).unwrap();
        let b = EquityEngine::compute(&req_b).unwrap();
        assert_eq!(a.players[0].wins, b.players[0].wins);
        assert_eq!(a.players[0].ties, b.players[0].ties);
        assert_eq!(a.method, Method::MonteCarlo);
    }

    /// Monte Carlo should match exact enumeration on the same (cheap) flop.
    #[test]
    fn compute__monte_carlo_matches_exact_within_tolerance() {
        let board = Board::from_str("7♦ 8♣ 2♠").unwrap();
        let mut exact_req = EquityRequest::new(vec![exact(Two::HAND_AS_AH), exact(Two::HAND_KS_KH)]);
        exact_req.board = board;
        let exact_report = EquityEngine::compute(&exact_req).unwrap();
        assert_eq!(exact_report.method, Method::Exact);

        let mc_req = EquityRequest {
            players: vec![exact(Two::HAND_AS_AH), exact(Two::HAND_KS_KH)],
            board,
            dead: Cards::default(),
            opts: EquityOptions {
                exact_threshold: 0,
                max_samples: 10_000,
                seed: Some(7),
            },
        };
        let mc_report = EquityEngine::compute(&mc_req).unwrap();
        assert_eq!(mc_report.method, Method::MonteCarlo);
        assert!((mc_report.players[0].equity - exact_report.players[0].equity).abs() < 0.02);
    }

    #[test]
    fn compute__random_opponents() {
        // One known hand against two random seats; just exercises the path.
        let req = EquityRequest {
            players: vec![exact(Two::HAND_AS_AH), PlayerSpec::Random, PlayerSpec::Random],
            board: Board::default(),
            dead: Cards::default(),
            opts: EquityOptions {
                exact_threshold: 0,
                max_samples: 5_000,
                seed: Some(1),
            },
        };
        let report = EquityEngine::compute(&req).unwrap();
        assert_eq!(report.players.len(), 3);
        let total: f64 = report.players.iter().map(|p| p.equity).sum();
        assert!((total - 1.0).abs() < 1e-6);
        // AA against two random hands still wins more than a third of the pot.
        assert!(report.players[0].equity > 0.34);
    }

    /// `EPIC-123` Phase 0: integers pinned at `0.17.4`, before `dead` existed.
    fn oracle_exact_turn_request() -> EquityRequest {
        let mut req = EquityRequest::new(vec![exact(Two::HAND_AS_AH), exact(Two::HAND_KS_KH)]);
        req.board = Board::from_str("7♦ 8♣ 2♠ 3♥").unwrap();
        req
    }

    fn oracle_monte_carlo_request() -> EquityRequest {
        let mut req = EquityRequest::new(vec![exact(Two::HAND_AS_AH), exact(Two::HAND_KS_KH)]);
        req.opts = EquityOptions {
            exact_threshold: 0,
            max_samples: 2_000,
            seed: Some(42),
        };
        req
    }

    fn tallies(report: &EquityReport) -> Vec<(u64, u64)> {
        report.players.iter().map(|p| (p.wins, p.ties)).collect()
    }

    #[test]
    fn compute__oracle_exact_turn_aa_vs_kk() {
        // pins: INV-11
        let report = EquityEngine::compute(&oracle_exact_turn_request()).unwrap();
        assert_eq!(Method::Exact, report.method);
        assert_eq!(44, report.samples);
        assert_eq!(vec![(42, 0), (2, 0)], tallies(&report));
    }

    #[test]
    fn compute__oracle_monte_carlo_seeded() {
        let report = EquityEngine::compute(&oracle_monte_carlo_request()).unwrap();
        assert_eq!(Method::MonteCarlo, report.method);
        assert_eq!(2_000, report.samples);
        assert_eq!(vec![(1_625, 11), (364, 11)], tallies(&report));
    }

    fn cards(index: &str) -> Cards {
        Cards::from_str(index).unwrap()
    }

    #[test]
    fn compute__empty_dead_matches_oracle() {
        let exact_report = EquityEngine::compute(&oracle_exact_turn_request().with_dead(Cards::default())).unwrap();
        assert_eq!(Method::Exact, exact_report.method);
        assert_eq!(44, exact_report.samples);
        assert_eq!(vec![(42, 0), (2, 0)], tallies(&exact_report));

        let mc_report = EquityEngine::compute(&oracle_monte_carlo_request().with_dead(Cards::default())).unwrap();
        assert_eq!(Method::MonteCarlo, mc_report.method);
        assert_eq!(2_000, mc_report.samples);
        assert_eq!(vec![(1_625, 11), (364, 11)], tallies(&mc_report));
    }

    #[test]
    fn compute__dead_card_repeating_a_seat_is_duplicate() {
        let req = oracle_exact_turn_request().with_dead(cards("A♠"));
        assert_eq!(EquityEngine::compute(&req).unwrap_err(), PKError::DuplicateCard);
    }

    #[test]
    fn compute__dead_card_repeating_the_board_is_duplicate() {
        let req = oracle_exact_turn_request().with_dead(cards("7♦"));
        assert_eq!(EquityEngine::compute(&req).unwrap_err(), PKError::DuplicateCard);
    }

    /// `Cards` is a set, so a repeat inside `dead` itself can only be written
    /// by building the set from a `Vec`; the engine still sees one card.
    #[test]
    fn compute__dead_card_repeated_in_dead_is_duplicate() {
        let dead = Cards::from(vec![Card::NINE_DIAMONDS, Card::NINE_DIAMONDS]);
        let req = oracle_exact_turn_request().with_dead(dead);
        let report = EquityEngine::compute(&req).unwrap();
        assert_eq!(43, report.samples, "the repeated card is removed once");
    }

    #[test]
    fn compute__dead_cards_shrink_the_turn_runouts() {
        let req = oracle_exact_turn_request().with_dead(cards("9♦ T♣"));
        let report = EquityEngine::compute(&req).unwrap();
        assert_eq!(Method::Exact, report.method);
        assert_eq!(42, report.samples);
    }

    #[test]
    fn compute__dead_cards_shrink_the_flop_runouts() {
        let mut req = EquityRequest::new(vec![exact(Two::HAND_AS_AH), exact(Two::HAND_KS_KH)]);
        req.board = Board::from_str("7♦ 8♣ 2♠").unwrap();
        let report = EquityEngine::compute(&req.with_dead(cards("9♦ T♣ J♥"))).unwrap();
        assert_eq!(Method::Exact, report.method);
        assert_eq!(EquityEngine::n_choose_k(42, 2), report.samples);
        assert_eq!(861, report.samples);
    }

    /// `A♠ K♠` on `Q♠ 7♠ 2♦ 9♣` against `Q♥ Q♦`: the river spades that win are
    /// 3, 4, 5, 6, 8, T and J (2♠ and 9♠ pair the board and fill the queens up).
    /// Killing `J♠ T♠` takes two of those seven outs away.
    #[test]
    fn compute__dead_suit_cards_lower_a_flush_draw() {
        let mut req = EquityRequest::new(vec![exact(Two::HAND_AS_KS), exact(Two::HAND_QH_QD)]);
        req.board = Board::from_str("Q♠ 7♠ 2♦ 9♣").unwrap();
        let live = EquityEngine::compute(&req).unwrap();
        let dead = EquityEngine::compute(&req.with_dead(cards("J♠ T♠"))).unwrap();

        assert_eq!((44, 7), (live.samples, live.players[0].wins));
        assert_eq!((42, 5), (dead.samples, dead.players[0].wins));
        assert!(dead.players[0].equity < live.players[0].equity);
    }

    #[test]
    fn compute__dead_cards_filter_a_range() {
        let req = EquityRequest::new(vec![
            exact(Two::HAND_KS_KH),
            PlayerSpec::Range(Combos::from_str("AA").unwrap()),
        ])
        .with_dead(cards("A♠ A♥ A♦"));
        assert_eq!(EquityEngine::compute(&req).unwrap_err(), PKError::InvalidHand);
    }

    /// Proved indirectly: `dead` holds every card but `A♣ A♦`, so the only
    /// legal deal gives the random seat aces, which beat queens on this board
    /// every time. A single dead card drawn would hand the random seat some
    /// other hand, and most of those lose to queens.
    #[test]
    fn compute__dead_cards_never_drawn_for_random_seat() {
        let board = Board::from_str("2♣ 7♦ 9♥ J♠ K♣").unwrap();
        let mut req = EquityRequest::new(vec![exact(Two::HAND_QH_QD), PlayerSpec::Random]);
        req.board = board;
        req.opts = EquityOptions {
            exact_threshold: 0,
            max_samples: 2_000,
            seed: Some(5),
        };
        let dead = Cards::deck_minus(&cards("2♣ 7♦ 9♥ J♠ K♣ Q♥ Q♦ A♣ A♦"));
        assert_eq!(43, dead.len());

        let report = EquityEngine::compute(&req.with_dead(dead)).unwrap();
        assert_eq!(Method::MonteCarlo, report.method);
        assert_eq!(2_000, report.samples);
        assert_eq!(2_000, report.players[1].wins);
    }

    /// The dead-card twin of `exact_enumerate__counts_are_identical_serial_or_parallel`:
    /// pinned integers that both the serial and the rayon arm must produce.
    #[test]
    fn exact_enumerate__counts_are_identical_serial_or_parallel_with_dead() {
        let mut req = EquityRequest::new(vec![exact(Two::HAND_AS_AH), exact(Two::HAND_KS_KH)]);
        req.board = Board::from_str("7♦ 8♣ 2♠").unwrap();
        let report = EquityEngine::compute(&req.with_dead(cards("9♦ T♣ J♥"))).unwrap();

        assert_eq!(Method::Exact, report.method);
        assert_eq!(861, report.samples);
        assert_eq!(vec![(784, 0), (77, 0)], tallies(&report));
    }

    #[test]
    fn compute__range_player() {
        let kk_plus = Combos::from_str("KK+").unwrap();
        let req = EquityRequest {
            players: vec![exact(Two::HAND_AS_KS), PlayerSpec::Range(kk_plus)],
            board: Board::default(),
            dead: Cards::default(),
            opts: EquityOptions {
                exact_threshold: 0,
                max_samples: 5_000,
                seed: Some(3),
            },
        };
        let report = EquityEngine::compute(&req).unwrap();
        assert_eq!(report.players.len(), 2);
        // AKs is a clear dog to a {KK, AA} range.
        assert!(report.players[1].equity > report.players[0].equity);
    }
}
