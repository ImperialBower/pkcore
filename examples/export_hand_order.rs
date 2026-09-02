//! Generates `src/bot/hand_order_table.rs` — the 169 canonical starting-hand
//! classes, strongest first, each with its exact equity against a uniformly
//! random opposing hand.
//!
//! The numbers come from `generated/hups.bin`, but that chart is 15.8 MB and
//! this table is the only thing the reads path wants from it. Precomputing the
//! 169 values keeps the chart out of binaries that do not otherwise need it —
//! a WASM build in particular.
//!
//! ```bash
//! cargo run --release --example export_hand_order > src/bot/hand_order_table.rs
//! ```
//!
//! `bot__hand_order_tests::table_matches_the_chart` re-derives the table and
//! fails if the checked-in file has drifted.

use pkcore::analysis::gto::combo::Combo;
use pkcore::bot::hand_order::derive_hand_ordering;
use std::str::FromStr;

fn main() {
    let ordering = derive_hand_ordering();
    assert_eq!(169, ordering.len(), "expected 169 canonical classes");

    println!("//! The precomputed starting-hand ordering (EPIC-39 Phase 3a).");
    println!("//!");
    println!("//! Generated — do not edit by hand:");
    println!("//!");
    println!("//! ```bash");
    println!("//! cargo run --release --example export_hand_order > src/bot/hand_order_table.rs");
    println!("//! ```");
    println!("//!");
    println!("//! Each entry is a canonical class and its exact equity against a uniformly");
    println!("//! random opposing hand, strongest first, derived from `generated/hups.bin`.");
    println!("//! Shipping the 169 numbers rather than reading the 15.8 MB chart at runtime");
    println!("//! is what keeps that chart out of a linked binary — see");
    println!("//! [`crate::bot::hand_order::derive_hand_ordering`].");
    println!();
    println!("/// The 169 canonical classes with their equity, strongest first.");
    println!("pub(crate) static HAND_ORDER: &[(&str, f64)] = &[");
    for (combo, equity) in &ordering {
        let notation = combo.to_string();
        assert_eq!(
            Ok(*combo),
            Combo::from_str(&notation),
            "{notation} must round-trip through FromStr"
        );
        println!("    (\"{notation}\", {equity:?}),");
    }
    println!("];");
}
