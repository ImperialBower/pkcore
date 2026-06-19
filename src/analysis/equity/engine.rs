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

/// Computes per-seat equity for an [`EquityRequest`].
///
/// Picks **exact enumeration** of all board runouts when every seat has known
/// cards and the runout count is within
/// [`EquityOptions::exact_threshold`](super::spec::EquityOptions::exact_threshold);
/// otherwise it runs **Monte Carlo** sampling. Both paths use a bounded `rayon`
/// pool and the on-the-fly Cactus-Kev evaluator.
///
/// # Errors
///
/// - [`PKError::NotEnoughHands`] if there are fewer than two or more than ten seats.
/// - [`PKError::InvalidCardCount`] if the board already holds more than five cards.
/// - [`PKError::DuplicateCard`] if any known card (hole or board) is repeated.
/// - [`PKError::InvalidHand`] if a range seat has no holding compatible with the dead cards.
///
/// # Examples
///
/// ```
/// use pkcore::analysis::equity::{EquityRequest, PlayerSpec, compute};
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
/// let report = compute(&req).unwrap();
/// assert!(report.players[0].equity > report.players[1].equity);
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

    // Dead cards = every known hole card plus the board; reject duplicates.
    let mut dead = board_vec.clone();
    for player in &req.players {
        if let PlayerSpec::Exact(two) = player {
            dead.push(two.first());
            dead.push(two.second());
        }
    }
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
    let runouts = n_choose_k(remaining.len() as u64, unknown_board as u64);

    let (tally, method) = if all_exact && runouts <= req.opts.exact_threshold {
        let fixed: Vec<Two> = resolved
            .iter()
            .filter_map(|r| match r {
                Resolved::Exact(two) => Some(*two),
                _ => None,
            })
            .collect();
        (
            exact_enumerate(&fixed, &board_vec, &remaining, unknown_board, n),
            Method::Exact,
        )
    } else {
        let seed = req.opts.seed.unwrap_or_else(rand::random);
        (
            monte_carlo(
                &resolved,
                &board_vec,
                &remaining,
                unknown_board,
                n,
                req.opts.max_samples,
                seed,
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
fn exact_enumerate(fixed: &[Two], board_vec: &[Card], remaining: &[Card], b: usize, n: usize) -> Tally {
    remaining
        .iter()
        .copied()
        .combinations(b)
        .par_bridge()
        .map(|runout| {
            let board = build_board(board_vec, &runout);
            let ranks: Vec<HandRank> = fixed
                .iter()
                .map(|two| Eval::from(Seven::from_case_and_board(two, &board)).hand_rank)
                .collect();
            tally_from_ranks(&ranks, n)
        })
        .reduce(|| Tally::zeroed(n), Tally::combine)
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
    (0..max_samples)
        .into_par_iter()
        .filter_map(|i| sample_once(resolved, board_vec, remaining, b, n, seed ^ i))
        .reduce(|| Tally::zeroed(n), Tally::combine)
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
                let two = pick_range(candidates, &taken, &mut rng)?;
                taken.insert(two.first());
                taken.insert(two.second());
                hands.push(two);
            }
            Resolved::Random => {
                let first = draw(remaining, &taken, &mut rng)?;
                taken.insert(first);
                let second = draw(remaining, &taken, &mut rng)?;
                taken.insert(second);
                hands.push(Two::new(first, second).ok()?);
            }
        }
    }

    let mut runout = Vec::with_capacity(b);
    for _ in 0..b {
        let card = draw(remaining, &taken, &mut rng)?;
        taken.insert(card);
        runout.push(card);
    }

    let board = build_board(board_vec, &runout);
    let ranks: Vec<HandRank> = hands
        .iter()
        .map(|two| Eval::from(Seven::from_case_and_board(two, &board)).hand_rank)
        .collect();
    Some(tally_from_ranks(&ranks, n))
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
    fn n_choose_k_basics() {
        assert_eq!(n_choose_k(47, 2), 1_081);
        assert_eq!(n_choose_k(48, 5), 1_712_304);
        assert_eq!(n_choose_k(5, 0), 1);
        assert_eq!(n_choose_k(2, 5), 0);
    }

    #[test]
    fn compute__rejects_single_player() {
        let req = EquityRequest::new(vec![exact(Two::HAND_AS_AH)]);
        assert_eq!(compute(&req).unwrap_err(), PKError::NotEnoughHands);
    }

    #[test]
    fn compute__rejects_duplicate_cards() {
        let req = EquityRequest::new(vec![exact(Two::HAND_AS_AH), exact(Two::HAND_AS_KS)]);
        assert_eq!(compute(&req).unwrap_err(), PKError::DuplicateCard);
    }

    #[test]
    fn compute__exact_river_is_deterministic_showdown() {
        // Full board: only one case, the winner is exact.
        let board = Board::from_str("9♣ 6♦ 5♥ 5♠ 8♠").unwrap();
        let mut req = EquityRequest::new(vec![exact(Two::HAND_6S_6H), exact(Two::HAND_5D_5C)]);
        req.board = board;
        let report = compute(&req).unwrap();
        assert_eq!(report.method, Method::Exact);
        assert_eq!(report.samples, 1);
        // Quad fives beats sixes-full-of-fives.
        assert_eq!(report.players[1].equity, 1.0);
        assert_eq!(report.players[0].equity, 0.0);
    }

    /// Exact enumeration on a dry flop is cheap (`C(45,2)` = 990 runouts) and
    /// uses the default options, so it exercises the exact path without the
    /// 1.7M-runout pre-flop blow-up.
    #[test]
    fn compute__exact_on_dry_flop() {
        let board = Board::from_str("7♦ 8♣ 2♠").unwrap();
        let mut req = EquityRequest::new(vec![exact(Two::HAND_AS_AH), exact(Two::HAND_KS_KH)]);
        req.board = board;
        let report = compute(&req).unwrap();
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
            opts,
        };
        let report = compute(&req).unwrap();
        assert_eq!(report.method, Method::MonteCarlo);
        assert!((report.players[0].equity - 0.82).abs() < 0.02);
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
            opts,
        };
        let req_b = EquityRequest {
            players: players(),
            board: Board::default(),
            opts,
        };
        let a = compute(&req_a).unwrap();
        let b = compute(&req_b).unwrap();
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
        let exact_report = compute(&exact_req).unwrap();
        assert_eq!(exact_report.method, Method::Exact);

        let mc_req = EquityRequest {
            players: vec![exact(Two::HAND_AS_AH), exact(Two::HAND_KS_KH)],
            board,
            opts: EquityOptions {
                exact_threshold: 0,
                max_samples: 10_000,
                seed: Some(7),
            },
        };
        let mc_report = compute(&mc_req).unwrap();
        assert_eq!(mc_report.method, Method::MonteCarlo);
        assert!((mc_report.players[0].equity - exact_report.players[0].equity).abs() < 0.02);
    }

    #[test]
    fn compute__random_opponents() {
        // One known hand against two random seats; just exercises the path.
        let req = EquityRequest {
            players: vec![exact(Two::HAND_AS_AH), PlayerSpec::Random, PlayerSpec::Random],
            board: Board::default(),
            opts: EquityOptions {
                exact_threshold: 0,
                max_samples: 5_000,
                seed: Some(1),
            },
        };
        let report = compute(&req).unwrap();
        assert_eq!(report.players.len(), 3);
        let total: f64 = report.players.iter().map(|p| p.equity).sum();
        assert!((total - 1.0).abs() < 1e-6);
        // AA against two random hands still wins more than a third of the pot.
        assert!(report.players[0].equity > 0.34);
    }

    #[test]
    fn compute__range_player() {
        let kk_plus = Combos::from_str("KK+").unwrap();
        let req = EquityRequest {
            players: vec![exact(Two::HAND_AS_KS), PlayerSpec::Range(kk_plus)],
            board: Board::default(),
            opts: EquityOptions {
                exact_threshold: 0,
                max_samples: 5_000,
                seed: Some(3),
            },
        };
        let report = compute(&req).unwrap();
        assert_eq!(report.players.len(), 2);
        // AKs is a clear dog to a {KK, AA} range.
        assert!(report.players[1].equity > report.players[0].equity);
    }
}
