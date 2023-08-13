use pkcore::arrays::matchups::masked::{Masked, MASKED_DISTINCT};
use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use std::collections::HashSet;

/// I'm going to use this file for a test harness validating are Masked distinct values.
fn main() {
    let distorg = SortedHeadsUp::distinct().unwrap();
    let distinct = MASKED_DISTINCT.clone();
    let mut diff: HashSet<SortedHeadsUp> = HashSet::new();

    for masked in distinct {
        if !distorg.contains(&masked.shu) {
            println!("Missing: {masked}");
            diff.insert(masked.shu);
        }
    }

    SortedHeadsUp::generate_csv("data/distinct_diff.csv", diff).expect("TODO: panic message");
}
