//! Scratch diagnostic (not part of the catalog, safe to delete).
//!
//! Decomposes `Five::hand_rank_value` to find where its ~102 ns goes, by
//! timing each component against the same 1024-hand sample the catalog uses.

use itertools::Itertools;
use pkcore::arrays::HandRanker;
use pkcore::prelude::{Deck, Five, Pile};
use std::time::Instant;

const SAMPLE_HANDS: usize = 1_024;
const STRIDE: usize = 97;
const ITERS: usize = 200_000;
const TRIALS: usize = 15;

fn sample() -> Vec<Five> {
    Deck::as_vec()
        .into_iter()
        .combinations(5)
        .step_by(STRIDE)
        .take(SAMPLE_HANDS)
        .map(Five::try_from)
        .collect::<Result<Vec<Five>, _>>()
        .expect("sample builds")
}

/// Times `f` over the sample, reporting the minimum ns/op across trials.
fn time(label: &str, hands: &[Five], f: impl Fn(&Five) -> u64) {
    let mut best = f64::MAX;
    for _ in 0..TRIALS {
        let start = Instant::now();
        let mut acc: u64 = 0;
        for i in 0..ITERS {
            acc = acc.wrapping_add(f(&hands[i % hands.len()]));
        }
        let ns = start.elapsed().as_nanos() as f64 / ITERS as f64;
        std::hint::black_box(acc);
        best = best.min(ns);
    }
    println!("{label:<34} {best:>8.2} ns/op");
}

fn main() {
    let hands = sample();

    // Baseline: how many of the sample are paired-or-better, i.e. take the
    // not_unique() binary-search path rather than the flat unique_rank lookup.
    let paired = hands
        .iter()
        .filter(|h| !h.is_flush() && Five::unique_rank(h.or_rank_bits() as usize) == 0)
        .count();
    println!(
        "sample: {} hands, {paired} paired-or-better ({:.1}%)\n",
        hands.len(),
        100.0 * paired as f64 / hands.len() as f64
    );

    time("or_rank_bits (floor)", &hands, |h| u64::from(h.or_rank_bits()));
    time("is_dealt", &hands, |h| u64::from(h.is_dealt()));
    time("  .are_unique", &hands, |h| u64::from(h.are_unique()));
    time("  .contains_blank", &hands, |h| u64::from(h.contains_blank()));
    time("is_flush", &hands, |h| u64::from(h.is_flush()));
    time("unique_rank(or_rank_bits)", &hands, |h| {
        u64::from(Five::unique_rank(h.or_rank_bits() as usize))
    });
    time("not_unique (binary search)", &hands, |h| {
        u64::from(h.not_unique())
    });
    time("hand_rank_value (total)", &hands, |h| {
        u64::from(h.hand_rank_value())
    });
}
