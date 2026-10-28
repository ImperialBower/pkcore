#![no_main]

//! Range strings (`"66+, AJo"`, `"AA:0.5"`) must never panic, whatever the
//! input. A range that parses is also expanded to its exact two-card hands.

use libfuzzer_sys::fuzz_target;
use pkcore::analysis::gto::twos::Twos;
use pkcore::analysis::gto::weighted_combos::WeightedCombos;
use pkcore::prelude::*;

struct RangeStrings;

impl RangeStrings {
    fn check(input: &str) {
        let _ = WeightedCombos::from_str(input);
        if let Ok(combos) = Combos::from_str(input) {
            let _ = Twos::from(&combos);
        }
    }
}

fuzz_target!(|input: &str| RangeStrings::check(input));
