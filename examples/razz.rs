use itertools::Itertools;
use pkcore::ranks::Ranks;
use std::str::FromStr;

fn main() {
    // Ace is 1, then 2-8
    let ranks = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
    let mut hands: Vec<Vec<u8>> = ranks
        .iter()
        .combinations(5)
        .map(|c| c.into_iter().copied().collect())
        .collect();

    // Sort by lowest to highest
    hands.sort();

    // Print top 100 hands
    for hand in hands.iter().take(6000) {
        // Map 1->A, 11->J, 12->Q, 13->K for display
        let display: Vec<String> = hand
            .iter()
            .map(|&r| match r {
                1 => "A".to_string(),
                10 => "T".to_string(),
                11 => "J".to_string(),
                12 => "Q".to_string(),
                13 => "K".to_string(),
                n => n.to_string(),
            })
            .collect();

        let ranks = Ranks::from_str(display.join(" ").as_str()).unwrap();
        let sum = ranks.sum_or();

        // println!("LOW_{} = 0b{:013b} {}", display.join(""), sum, sum);
        // println!("LOW_{} = 0b{:013b},", display.join(""), sum);
        println!("0b{:013b} => RazzHandRank::LOW_{},", sum, display.join(""));
    }
}
