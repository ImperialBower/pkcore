//! Scratch diagnostic (not part of the catalog, safe to delete).
//!
//! Decomposes the gap between `eval.seven.hand_rank_value` (734 ns) and
//! `eval.seven.eval` (4132 ns) — the ~3400 ns that `Seven::hand_rank_value_and_hand`
//! (`src/arrays/seven.rs:197`) pays for `best_hand.sort().clean()` on the
//! winning five-card hand, once per seven-card evaluation.
//!
//! Times each link of the chain against the same 1024-hand sample the catalog
//! uses:
//!
//!   Seven::hand_rank_value_and_hand
//!     -> Five::sort              (arrays/mod.rs -> five.rs:246)
//!        -> Five::sort_in_place  (five.rs:253)
//!           -> Five::is_wheel    (five.rs:100)
//!           -> Pile::cards       (lib.rs:798 — Vec + IndexSet)
//!           -> Cards::frequency_weighted (cards.rs:360)
//!              -> Cards::map_by_rank     (cards.rs:535 — HashMap over 13 ranks)
//!           -> Five::try_from(Cards)
//!           -> Five::clean       (five.rs:323 — 5x Card::clean, one AND each)
//!     -> Five::clean             (the redundant second clean at seven.rs:197)

use itertools::Itertools;
use pkcore::analysis::eval::Eval;
use pkcore::arrays::HandRanker;
use pkcore::prelude::{Card, Deck, Five, Pile, Seven};
use std::time::Instant;

const SAMPLE_HANDS: usize = 1_024;
const STRIDE: usize = 97;
const ITERS: usize = 20_000;
const TRIALS: usize = 15;

fn seven_sample() -> Vec<Seven> {
    Deck::as_vec()
        .into_iter()
        .combinations(7)
        .step_by(STRIDE)
        .take(SAMPLE_HANDS)
        .map(|cards| Seven::from(<[Card; 7]>::try_from(cards).expect("7 cards")))
        .collect()
}

/// The winning `Five` for each `Seven`, taken **before** `sort()` touches it.
///
/// Replicates `Seven::hand_rank_value_and_hand`'s permutation loop
/// (`seven.rs:188-196`) minus its trailing `.sort().clean()`, so the sample is
/// the raw permutation-order hand that `sort()` actually receives in
/// production. Timing `sort()` on an already-sorted hand would measure a
/// different input distribution.
fn winner_sample(sevens: &[Seven]) -> Vec<Five> {
    sevens
        .iter()
        .map(|seven| {
            let mut best_hrv = u16::MAX;
            let mut best_hand = Five::default();
            for perm in Seven::FIVE_CARD_PERMUTATIONS {
                let hand = seven.five_from_permutation(perm);
                let hrv = hand.hand_rank_value();
                if (best_hrv == 0) || hrv != 0 && hrv < best_hrv {
                    best_hrv = hrv;
                    best_hand = hand;
                }
            }
            best_hand
        })
        .collect()
}

/// Times `f` over `items`, reporting the minimum ns/op across trials.
fn time<T>(label: &str, items: &[T], f: impl Fn(&T) -> u64) {
    let mut best = f64::MAX;
    for _ in 0..TRIALS {
        let start = Instant::now();
        let mut acc: u64 = 0;
        for i in 0..ITERS {
            acc = acc.wrapping_add(f(&items[i % items.len()]));
        }
        let ns = start.elapsed().as_nanos() as f64 / ITERS as f64;
        std::hint::black_box(acc);
        best = best.min(ns);
    }
    println!("{label:<44} {best:>10.2} ns/op");
}

fn main() {
    let sevens = seven_sample();
    let winners = winner_sample(&sevens);

    let wheels = winners.iter().filter(|five| five.is_wheel()).count();
    println!(
        "sample: {} sevens, {} winning fives, {wheels} wheels ({:.1}% take the cheap sort path)\n",
        sevens.len(),
        winners.len(),
        100.0 * wheels as f64 / winners.len() as f64
    );

    println!("-- whole-Seven paths --");
    time("Seven::hand_rank_value (no sort)", &sevens, |s| {
        u64::from(s.hand_rank_value())
    });
    time("Seven::hand_rank_value_and_hand", &sevens, |s| {
        u64::from(s.hand_rank_value_and_hand().0)
    });
    time("Eval::from(Seven)", &sevens, |s| {
        u64::from(Eval::from(*s).hand_rank.value)
    });
    // The proposed replacement at engine.rs:171 and :238, which want only
    // `.hand_rank` and discard the `Eval`'s sorted `.hand`.
    time("Seven::hand_rank()  (same HandRank, no hand)", &sevens, |s| {
        u64::from(s.hand_rank().value)
    });

    println!("\n-- the tail, on the winning Five --");
    time("Five::sort().clean()  (what seven.rs:197 does)", &winners, |f| {
        u64::from(f.sort().clean().or_rank_bits())
    });
    time("Five::sort()", &winners, |f| u64::from(f.sort().or_rank_bits()));
    time("Five::clean()  (5x bitwise AND)", &winners, |f| {
        u64::from(f.clean().or_rank_bits())
    });

    println!("\n-- inside Five::sort_in_place (five.rs:253) --");
    time("Five::is_wheel()  (the branch)", &winners, |f| u64::from(f.is_wheel()));
    time("Pile::cards()  (Vec + IndexSet)", &winners, |f| f.cards().len() as u64);
    time("  .frequency_weighted()  (HashMap x13)", &winners, |f| {
        f.cards().frequency_weighted().len() as u64
    });
    time(
        "  .. + Five::try_from  (full non-wheel body)",
        &winners,
        |f| match Five::try_from(f.cards().frequency_weighted()) {
            Ok(five) => u64::from(five.or_rank_bits()),
            Err(_) => 1,
        },
    );
    time("[u8;5] sort_unstable  (what a sort costs)", &winners, |f| {
        let mut arr = f.to_arr();
        arr.sort_unstable();
        u64::from(Five::from(arr).or_rank_bits())
    });
}
