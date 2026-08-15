//! The nano-band workloads under [Divan](https://github.com/nvzqz/divan),
//! for side-by-side comparison with the custom harness and Criterion.
//!
//! ```text
//! cd perf && cargo bench --bench divan
//! ```
//!
//! See `benches/criterion.rs` for what the three harnesses do differently
//! and why their figures will not match exactly. Divan's contribution to the
//! comparison: the leanest API of the three — an attribute per function, and
//! any value returned from the benched closure is black-boxed automatically,
//! so there is no explicit `black_box` and no checksum discipline.
//!
//! Function names mirror the catalog's dotted workload names
//! (`eval.five.hand_rank_value` → `five_hand_rank_value`).

use pkcore::arrays::HandRanker;
use pkcore::prelude::{Five, FromStr};
use pkcore_perf::catalog::{five_sample, seven_sample};

fn main() {
    divan::main();
}

#[divan::bench]
fn five_hand_rank_value(bencher: divan::Bencher) {
    let hands = five_sample().expect("sample builds");
    let mask = hands.len() - 1;
    let mut i = 0_usize;
    bencher.bench_local(move || {
        i = (i + 1) & mask;
        hands[i].hand_rank_value()
    });
}

#[divan::bench]
fn seven_hand_rank_value(bencher: divan::Bencher) {
    let hands = seven_sample().expect("sample builds");
    let mask = hands.len() - 1;
    let mut i = 0_usize;
    bencher.bench_local(move || {
        i = (i + 1) & mask;
        hands[i].hand_rank_value()
    });
}

#[divan::bench]
fn five_or_rank_bits(bencher: divan::Bencher) {
    let hands = five_sample().expect("sample builds");
    let mask = hands.len() - 1;
    let mut i = 0_usize;
    bencher.bench_local(move || {
        i = (i + 1) & mask;
        hands[i].or_rank_bits()
    });
}

#[divan::bench]
fn five_from_str(bencher: divan::Bencher) {
    let hands = five_sample().expect("sample builds");
    let texts: Vec<String> = hands.iter().map(ToString::to_string).collect();
    let mask = texts.len() - 1;
    let mut i = 0_usize;
    bencher.bench_local(move || {
        i = (i + 1) & mask;
        Five::from_str(&texts[i])
    });
}
