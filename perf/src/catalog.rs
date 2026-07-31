//! pkcore's measurable workloads.
//!
//! Phase 1 covers the nano band only: the Cactus-Kev evaluator and the parsing
//! path. All of it builds under pkcore's `--no-default-features` pure kernel,
//! so these numbers are the publishable kernel headline.

use crate::workload::{Band, HotFn, PerfError, Workload};
use itertools::Itertools;
use pkcore::analysis::eval::Eval;
use pkcore::arrays::HandRanker;
use pkcore::prelude::{Card, Deck, Five, FromStr, Seven};

/// How many distinct hands each workload cycles through. A power of two, so
/// the modulo in the hot loop compiles to a mask.
const SAMPLE_HANDS: usize = 1_024;

/// Stride through the combination space, so the sample spans a wide range of
/// hand types instead of the first N lexicographic hands (which are all
/// low-card garbage). Coprime with nothing in particular — just large enough
/// to spread out and small enough to stay cheap.
const STRIDE: usize = 97;

/// Index mask for the hot loops. `i & MASK` is a single `and` instruction,
/// where `i % hands.len()` is a real division the compiler cannot eliminate
/// because the length is a runtime value. At ~20-40 cycles on this host that
/// division was a meaningful fraction of a nano-band measurement.
///
/// Correct only because `SAMPLE_HANDS` is a power of two and every sample
/// builder returns exactly that many hands — both asserted in the tests.
const MASK: usize = SAMPLE_HANDS - 1;

/// Builds a deterministic spread of five-card hands.
fn five_sample() -> Result<Vec<Five>, PerfError> {
    let hands: Vec<Five> = Deck::as_vec()
        .into_iter()
        .combinations(5)
        .step_by(STRIDE)
        .take(SAMPLE_HANDS)
        .map(Five::try_from)
        .collect::<Result<Vec<Five>, _>>()
        .map_err(|e| PerfError::Setup(format!("building five-card sample: {e:?}")))?;

    if hands.len() < SAMPLE_HANDS {
        return Err(PerfError::Setup(format!(
            "expected {SAMPLE_HANDS} hands, built {}",
            hands.len()
        )));
    }
    Ok(hands)
}

/// Builds a deterministic spread of seven-card hands.
fn seven_sample() -> Result<Vec<Seven>, PerfError> {
    let hands: Vec<Seven> = Deck::as_vec()
        .into_iter()
        .combinations(7)
        .step_by(STRIDE)
        .take(SAMPLE_HANDS)
        .map(|cards| {
            <[Card; 7]>::try_from(cards)
                .map(Seven::from)
                .map_err(|v| PerfError::Setup(format!("expected 7 cards, got {}", v.len())))
        })
        .collect::<Result<Vec<Seven>, PerfError>>()?;

    if hands.len() < SAMPLE_HANDS {
        return Err(PerfError::Setup(format!(
            "expected {SAMPLE_HANDS} hands, built {}",
            hands.len()
        )));
    }
    Ok(hands)
}

fn make_five_hand_rank_value() -> Result<HotFn, PerfError> {
    let hands = five_sample()?;
    Ok(Box::new(move |iters: u32| {
        let mut acc: u64 = 0;
        for i in 0..iters as usize {
            acc = acc.wrapping_add(u64::from(hands[i & MASK].hand_rank_value()));
        }
        acc
    }))
}

fn make_seven_hand_rank_value() -> Result<HotFn, PerfError> {
    let hands = seven_sample()?;
    Ok(Box::new(move |iters: u32| {
        let mut acc: u64 = 0;
        for i in 0..iters as usize {
            acc = acc.wrapping_add(u64::from(hands[i & MASK].hand_rank_value()));
        }
        acc
    }))
}

/// `Eval::from(Seven)` — the call the equity engine makes per showdown, which
/// goes through `hand_rank_value_and_hand` rather than the rank-only fast path
/// that `eval.seven.hand_rank_value` measures. The gap between the two
/// workloads is the cost of also materialising the winning five-card hand.
fn make_seven_eval() -> Result<HotFn, PerfError> {
    let hands = seven_sample()?;
    Ok(Box::new(move |iters: u32| {
        let mut acc: u64 = 0;
        for i in 0..iters as usize {
            acc = acc.wrapping_add(u64::from(Eval::from(hands[i & MASK]).hand_rank.value));
        }
        acc
    }))
}

fn make_five_or_rank_bits() -> Result<HotFn, PerfError> {
    let hands = five_sample()?;
    Ok(Box::new(move |iters: u32| {
        let mut acc: u64 = 0;
        for i in 0..iters as usize {
            acc = acc.wrapping_add(u64::from(hands[i & MASK].or_rank_bits()));
        }
        acc
    }))
}

fn make_five_from_str() -> Result<HotFn, PerfError> {
    let hands = five_sample()?;
    let texts: Vec<String> = hands.iter().map(ToString::to_string).collect();

    // Validate the round-trip at setup time. Without this, a Display/FromStr
    // mismatch would make every parse in the hot loop fail — and because the
    // error arm still folds a non-zero value into the checksum, the harness's
    // dead-code guard would pass while timing the error path instead of the
    // parser.
    for (text, expected) in texts.iter().zip(hands.iter()) {
        match Five::from_str(text) {
            Ok(parsed) if parsed.or_rank_bits() == expected.or_rank_bits() => {}
            Ok(_) => {
                return Err(PerfError::Setup(format!(
                    "Five::from_str({text:?}) round-tripped to a different hand"
                )));
            }
            Err(e) => {
                return Err(PerfError::Setup(format!("Five::from_str({text:?}) failed: {e:?}")));
            }
        }
    }

    Ok(Box::new(move |iters: u32| {
        let mut acc: u64 = 0;
        for i in 0..iters as usize {
            let parsed = Five::from_str(&texts[i & MASK]);
            acc = acc.wrapping_add(match parsed {
                Ok(five) => u64::from(five.or_rank_bits()),
                Err(_) => 1,
            });
        }
        acc
    }))
}

/// Every workload pkcore currently exposes for measurement.
///
/// Phase 1 returns four nano-band workloads, all pure kernel. Phase 2 adds
/// `eval.seven.eval`, the equity engine's real showdown call.
///
/// # Examples
///
/// ```
/// use pkcore_perf::catalog::catalog;
///
/// assert!(catalog().iter().any(|w| w.name == "eval.seven.eval"));
/// ```
#[must_use]
pub fn catalog() -> Vec<Workload> {
    vec![
        Workload {
            name: "eval.five.hand_rank_value",
            band: Band::Nano,
            inner_iters: 100_000,
            features: &[],
            make: make_five_hand_rank_value,
        },
        Workload {
            name: "eval.seven.hand_rank_value",
            band: Band::Nano,
            inner_iters: 10_000,
            features: &[],
            make: make_seven_hand_rank_value,
        },
        Workload {
            name: "eval.seven.eval",
            band: Band::Nano,
            inner_iters: 10_000,
            features: &[],
            make: make_seven_eval,
        },
        Workload {
            name: "eval.five.or_rank_bits",
            band: Band::Nano,
            inner_iters: 100_000,
            features: &[],
            make: make_five_or_rank_bits,
        },
        Workload {
            name: "parse.five.from_str",
            band: Band::Nano,
            inner_iters: 10_000,
            features: &[],
            make: make_five_from_str,
        },
    ]
}

#[cfg(test)]
#[allow(non_snake_case)]
mod perf__catalog_tests {
    use super::*;
    use crate::runner::{Status, measure};

    #[test]
    fn catalog_contains_the_nano_workloads_in_order() {
        let nano: Vec<&str> = catalog()
            .iter()
            .filter(|w| w.band == Band::Nano)
            .map(|w| w.name)
            .collect();
        assert_eq!(
            nano,
            vec![
                "eval.five.hand_rank_value",
                "eval.seven.hand_rank_value",
                "eval.seven.eval",
                "eval.five.or_rank_bits",
                "parse.five.from_str",
            ]
        );
    }

    #[test]
    fn catalog_includes_the_equity_engines_real_eval_path() {
        let names: Vec<&str> = catalog().iter().map(|w| w.name).collect();
        assert!(
            names.contains(&"eval.seven.eval"),
            "eval.seven.eval missing; got {names:?}"
        );
    }

    /// `Eval::from(Seven)` must agree with `Seven::hand_rank_value` on the same
    /// hands — if it did not, the two workloads would not be comparable and the
    /// ratio between them would be meaningless.
    #[test]
    fn seven_eval_agrees_with_seven_hand_rank_value() {
        use pkcore::analysis::eval::Eval;
        use pkcore::arrays::HandRanker;

        for hand in seven_sample().expect("sample builds").iter().take(64) {
            assert_eq!(
                Eval::from(*hand).hand_rank.value,
                hand.hand_rank_value(),
                "disagreement on {hand}"
            );
        }
    }

    #[test]
    fn every_workload_is_pure_kernel_and_nano_band() {
        for workload in catalog() {
            assert_eq!(workload.band, Band::Nano, "{}", workload.name);
            assert!(
                workload.features.is_empty(),
                "{} should need no features",
                workload.name
            );
        }
    }

    /// The hot loops index with `& MASK` rather than `% len`, which is only
    /// correct if the sample length is exactly `MASK + 1` and a power of two.
    #[test]
    fn sample_length_is_the_power_of_two_the_mask_assumes() {
        assert!(
            SAMPLE_HANDS.is_power_of_two(),
            "SAMPLE_HANDS must be a power of two for the mask index"
        );
        assert_eq!(MASK, SAMPLE_HANDS - 1);
        assert_eq!(five_sample().expect("sample builds").len(), SAMPLE_HANDS);
        assert_eq!(seven_sample().expect("sample builds").len(), SAMPLE_HANDS);
    }

    /// Smoke test: every workload's setup succeeds and one iteration runs.
    #[test]
    fn every_workload_sets_up_and_runs() {
        for workload in catalog() {
            let hot = (workload.make)().unwrap_or_else(|e| panic!("{} setup failed: {e}", workload.name));
            let _ = hot(1);
        }
    }

    /// The dead-code-elimination guard. If the optimizer deleted the work, the
    /// checksum would be a constant 0; if the work were unstable, trials would
    /// disagree. Both show up here.
    #[test]
    fn every_workload_is_deterministic_and_does_real_work() {
        for workload in catalog() {
            let sample = measure(&workload, 1, 3, 512);
            assert_eq!(
                sample.status,
                Status::Ok,
                "{} was not Ok: {:?}",
                workload.name,
                sample.message
            );
            assert_ne!(
                sample.checksum,
                Some(0),
                "{} produced a zero checksum — suspect dead-code elimination",
                workload.name
            );
        }
    }
}
