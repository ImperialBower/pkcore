#![no_main]

//! Card strings (`"AS KD 9C"`) must never panic, whatever the input. A string
//! that parses as five or seven cards is also ranked, so the evaluator sees
//! every card set the parser lets through.

use libfuzzer_sys::fuzz_target;
use pkcore::arrays::HandRanker;
use pkcore::prelude::*;

struct CardStrings;

impl CardStrings {
    fn check(input: &str) {
        let _ = Cards::from_str(input);
        let _ = Board::from_str(input);
        let _ = HoleCards::from_str(input);
        if let Ok(five) = Five::from_str(input) {
            let _ = five.hand_rank_value();
        }
        if let Ok(seven) = Seven::from_str(input) {
            let _ = seven.hand_rank_value();
        }
    }
}

fuzz_target!(|input: &str| CardStrings::check(input));
