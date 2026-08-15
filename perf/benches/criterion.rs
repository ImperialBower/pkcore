//! The nano-band workloads under [Criterion](https://github.com/bheisler/criterion.rs),
//! for side-by-side comparison with the custom harness and Divan.
//!
//! ```text
//! cd perf && cargo bench --bench criterion
//! ```
//!
//! The three harnesses measure the same operations over the same hand
//! samples ([`five_sample`]/[`seven_sample`]), but they do not measure the
//! same way, and their figures will not match exactly:
//!
//! - **The custom harness** (`perf run`) times a whole inner loop of N
//!   operations per trial and divides by N. Simple, portable to wasm, and it
//!   folds an integer checksum both to defeat dead-code elimination and to
//!   prove cross-target determinism.
//! - **Criterion** samples many short timing windows, fits a linear model,
//!   and reports a confidence interval. Statistically richer; heavier; no
//!   checksum, so it relies on `black_box` to keep the work alive.
//! - **Divan** (see `benches/divan.rs`) times per-call with a lighter
//!   protocol and black-boxes returned values automatically.
//!
//! Bench IDs reuse the catalog's dotted workload names so a row here can be
//! matched directly against `docs/perf/RESULTS.md`.

use criterion::{Criterion, criterion_group, criterion_main};
use pkcore::arrays::HandRanker;
use pkcore::prelude::{Five, FromStr};
use pkcore_perf::catalog::{five_sample, seven_sample};
use std::hint::black_box;

fn five_hand_rank_value(c: &mut Criterion) {
    let hands = five_sample().expect("sample builds");
    let mask = hands.len() - 1;
    let mut i = 0_usize;
    c.bench_function("eval.five.hand_rank_value", |b| {
        b.iter(|| {
            i = (i + 1) & mask;
            black_box(hands[i].hand_rank_value())
        });
    });
}

fn seven_hand_rank_value(c: &mut Criterion) {
    let hands = seven_sample().expect("sample builds");
    let mask = hands.len() - 1;
    let mut i = 0_usize;
    c.bench_function("eval.seven.hand_rank_value", |b| {
        b.iter(|| {
            i = (i + 1) & mask;
            black_box(hands[i].hand_rank_value())
        });
    });
}

fn five_or_rank_bits(c: &mut Criterion) {
    let hands = five_sample().expect("sample builds");
    let mask = hands.len() - 1;
    let mut i = 0_usize;
    c.bench_function("eval.five.or_rank_bits", |b| {
        b.iter(|| {
            i = (i + 1) & mask;
            black_box(hands[i].or_rank_bits())
        });
    });
}

fn five_from_str(c: &mut Criterion) {
    let hands = five_sample().expect("sample builds");
    let texts: Vec<String> = hands.iter().map(ToString::to_string).collect();
    let mask = texts.len() - 1;
    let mut i = 0_usize;
    c.bench_function("parse.five.from_str", |b| {
        b.iter(|| {
            i = (i + 1) & mask;
            black_box(Five::from_str(&texts[i]))
        });
    });
}

criterion_group!(
    nano,
    five_hand_rank_value,
    seven_hand_rank_value,
    five_or_rank_bits,
    five_from_str
);
criterion_main!(nano);
