//! Equity-engine workloads. Requires the `equity` feature.
//!
//! Checksums fold the integer `wins` and `ties` counts, never the `f64` equity
//! figure: rayon's reduction order varies between runs and float addition is
//! not associative, so a float checksum would report spurious mismatches (see
//! design Section 3).

use crate::workload::{Band, HotFn, PerfError, Workload};
use pkcore::analysis::equity::{EquityOptions, EquityRequest, PlayerSpec};
use pkcore::arrays::two::Two;
use pkcore::play::board::Board;
use pkcore::prelude::FromStr;

/// A dry, unpaired flop that gives neither hand a draw worth speaking of, so
/// the measured work is enumeration rather than an unusual board texture.
const DRY_FLOP: &str = "2♣ 7♦ 9♠";

/// Parses [`DRY_FLOP`], mapping the error into a `PerfError` so setup failures
/// are reported rather than panicking.
fn dry_flop() -> Result<Board, PerfError> {
    Board::from_str(DRY_FLOP).map_err(|e| PerfError::Setup(format!("parsing flop {DRY_FLOP:?}: {e:?}")))
}

/// AA versus KK on [`DRY_FLOP`] — 990 runouts, enumerated exactly.
fn hu_flop_request() -> EquityRequest {
    EquityRequest {
        players: vec![PlayerSpec::Exact(Two::HAND_AS_AH), PlayerSpec::Exact(Two::HAND_KS_KH)],
        board: dry_flop().unwrap_or_default(),
        opts: EquityOptions {
            exact_threshold: 100_000,
            max_samples: 100_000,
            seed: Some(42),
        },
    }
}

/// AA versus KK pre-flop, forced to enumerate exactly. `C(48,5)` is about
/// 1.7M runouts — `exact_threshold` is raised well above that count so this
/// is true enumeration, not the Monte Carlo fallback most pre-flop requests
/// take. That much real work is minutes, not micro-to-seconds, on this host:
/// [`Band::Macro`], not [`Band::Micro`] like its siblings. Because it is
/// `Macro`, the smoke tests below only prove it sets up; they do not run it.
fn hu_preflop_request() -> EquityRequest {
    EquityRequest {
        players: vec![PlayerSpec::Exact(Two::HAND_AS_AH), PlayerSpec::Exact(Two::HAND_KS_KH)],
        board: Board::default(),
        opts: EquityOptions {
            exact_threshold: 2_000_000,
            max_samples: 100_000,
            seed: Some(42),
        },
    }
}

/// Three-way seeded Monte Carlo, the shape a live table actually asks for.
///
/// The third seat is `A♦K♦`, not `A♠K♠`: `Two::HAND_AS_KS` shares the ace of
/// spades with the first seat's `Two::HAND_AS_AH`, which `compute` correctly
/// rejects as `PKError::DuplicateCard`. `A♦K♦` keeps the same AKs shape
/// without colliding with either pocket pair.
fn three_way_request() -> EquityRequest {
    EquityRequest {
        players: vec![
            PlayerSpec::Exact(Two::HAND_AS_AH),
            PlayerSpec::Exact(Two::HAND_KS_KH),
            PlayerSpec::Exact(Two::HAND_AD_KD),
        ],
        board: Board::default(),
        opts: EquityOptions {
            exact_threshold: 0,
            max_samples: 20_000,
            seed: Some(42),
        },
    }
}

/// Builds the timed closure that folds the integer win and tie counts across
/// every seat, for one already-built request.
fn hot_closure(request: EquityRequest) -> HotFn {
    Box::new(move |iters: u32| {
        let mut acc: u64 = 0;
        for _ in 0..iters {
            if let Ok(report) = request.compute() {
                for player in &report.players {
                    acc = acc.wrapping_add(player.wins).wrapping_add(player.ties);
                }
            }
        }
        acc
    })
}

/// Wraps a request builder into a hot closure that folds the integer win and
/// tie counts across every seat.
///
/// Validates by computing once before the timed region, so a bad fixture is a
/// setup error rather than a mysteriously fast measurement. That validation
/// call is itself real work — cheap for every workload in this module except
/// [`hu_preflop_request`], whose `make` bypasses this helper for exactly that
/// reason; see [`make_hu_preflop`].
fn make_from_request(build: fn() -> EquityRequest) -> Result<HotFn, PerfError> {
    let request = build();
    request
        .compute()
        .map_err(|e| PerfError::Setup(format!("equity request failed: {e:?}")))?;
    Ok(hot_closure(request))
}

fn make_hu_flop() -> Result<HotFn, PerfError> {
    make_from_request(hu_flop_request)
}

/// Deliberately skips `make_from_request`'s eager validate-by-computing:
/// for this one request, that validation *is* the ~1.7M-runout enumeration
/// that earns it [`Band::Macro`] in the first place, so paying for it during
/// setup would make every smoke test that proves setup succeeds just as slow
/// as actually running the workload. The two seats are hardcoded, known
/// non-overlapping exact hands (unlike `equity.mc.three_way`'s three seats,
/// where the same class of duplicate-card mistake this validation exists to
/// catch was actually found), and the exact-enumeration code path itself is
/// already exercised by `equity.exact.hu_flop`, so the residual risk this
/// skips checking is small.
fn make_hu_preflop() -> Result<HotFn, PerfError> {
    Ok(hot_closure(hu_preflop_request()))
}

fn make_three_way() -> Result<HotFn, PerfError> {
    make_from_request(three_way_request)
}

/// Ported from the `benches/preflop_odds.rs` heads-up case that Phase 5
/// deletes. Kept distinct from `equity.exact.hu_preflop` because it fixes the
/// sample count rather than the threshold.
fn make_dealeval_hu() -> Result<HotFn, PerfError> {
    make_from_request(|| EquityRequest {
        players: vec![PlayerSpec::Exact(Two::HAND_AS_KS), PlayerSpec::Random],
        board: Board::default(),
        opts: EquityOptions {
            exact_threshold: 0,
            max_samples: 10_000,
            seed: Some(7),
        },
    })
}

/// Ported from the `benches/preflop_odds.rs` three-way case. One known hand
/// against two unknowns is the shape a hand-history replayer asks for, and the
/// extra seat roughly doubles the per-sample showdown work relative to
/// [`make_dealeval_hu`].
fn make_dealeval_three_way() -> Result<HotFn, PerfError> {
    make_from_request(|| EquityRequest {
        players: vec![
            PlayerSpec::Exact(Two::HAND_AS_KS),
            PlayerSpec::Random,
            PlayerSpec::Random,
        ],
        board: Board::default(),
        opts: EquityOptions {
            exact_threshold: 0,
            max_samples: 10_000,
            seed: Some(7),
        },
    })
}

/// Every equity-engine workload.
///
/// # Examples
///
/// ```
/// use pkcore_perf::catalog_equity::equity_workloads;
///
/// assert!(equity_workloads().iter().all(|w| w.features.contains(&"equity")));
/// ```
#[must_use]
pub fn equity_workloads() -> Vec<Workload> {
    vec![
        Workload {
            name: "equity.exact.hu_flop",
            band: Band::Micro,
            inner_iters: 1,
            features: &["equity"],
            make: make_hu_flop,
        },
        Workload {
            name: "equity.exact.hu_preflop",
            band: Band::Macro,
            inner_iters: 1,
            features: &["equity"],
            make: make_hu_preflop,
        },
        Workload {
            name: "equity.mc.three_way",
            band: Band::Micro,
            inner_iters: 1,
            features: &["equity"],
            make: make_three_way,
        },
        Workload {
            name: "dealeval.hu",
            band: Band::Micro,
            inner_iters: 1,
            features: &["equity"],
            make: make_dealeval_hu,
        },
        Workload {
            name: "dealeval.three_way",
            band: Band::Micro,
            inner_iters: 1,
            features: &["equity"],
            make: make_dealeval_three_way,
        },
    ]
}

#[cfg(test)]
#[allow(non_snake_case)]
mod perf__catalog_equity_tests {
    use super::*;
    use crate::runner::{Status, measure};
    use crate::workload::Band;

    /// Every equity workload declares the `equity` feature; `hu_preflop`
    /// alone is `Macro` (true 1.7M-runout enumeration), its four siblings
    /// `Micro`.
    #[test]
    fn every_equity_workload_declares_the_equity_feature() {
        for workload in equity_workloads() {
            let expected_band = if workload.name == "equity.exact.hu_preflop" {
                Band::Macro
            } else {
                Band::Micro
            };
            assert_eq!(workload.band, expected_band, "{}", workload.name);
            assert!(
                workload.features.contains(&"equity"),
                "{} must declare the equity feature",
                workload.name
            );
        }
    }

    /// Exact enumeration is deterministic, and seeded Monte Carlo must be too —
    /// an unseeded RNG here would surface as `Status::Nondeterministic`.
    ///
    /// Macro-band workloads (currently only `hu_preflop`, a true 1.7M-runout
    /// enumeration) are excluded from the timed run: minutes per trial would
    /// make this smoke test unusable, and — because `make_hu_preflop` skips
    /// the eager validate-by-computing that the cheaper workloads get (see
    /// its doc comment) — calling `(workload.make)()` here is deliberately
    /// cheap rather than a real proof of correctness. `hu_preflop` is instead
    /// exercised by explicitly running it through the release binary
    /// (`perf run equity.exact.hu_preflop`), where paying for the real
    /// enumeration once is affordable.
    #[test]
    fn every_equity_workload_is_deterministic_and_does_real_work() {
        for workload in equity_workloads() {
            if workload.band == Band::Macro {
                let _ = (workload.make)().unwrap_or_else(|e| panic!("{} setup failed: {e}", workload.name));
                continue;
            }
            let sample = measure(&workload, 0, 2, 1);
            assert_eq!(
                sample.status,
                Status::Ok,
                "{} was not Ok: {:?}",
                workload.name,
                sample.message
            );
            assert_ne!(sample.checksum, Some(0), "{} produced a zero checksum", workload.name);
        }
    }

    /// AA versus KK on a dry, unpaired, rainbow-ish flop: aces are a heavy
    /// favourite. A checksum alone cannot catch a workload that computes the
    /// wrong thing quickly; this does.
    #[test]
    fn hu_flop_puts_aces_far_ahead() {
        let report = hu_flop_request().compute().expect("computes");
        assert_eq!(report.players.len(), 2);
        assert!(
            report.players[0].equity > 0.75,
            "AA equity was {}",
            report.players[0].equity
        );
    }
}
